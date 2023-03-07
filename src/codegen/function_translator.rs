use std::collections::HashMap;

use cranelift::{prelude::{FunctionBuilder, Value, InstBuilder, FunctionBuilderContext, isa::{CallConv, TargetFrontendConfig}, MemFlags}, codegen::Context};
use cranelift_module::{Module, Linkage, FuncId};

use crate::{parser::{node::{Node, Literal}, types::{typ::Type, self}, signature_parser::TypedSignature}, error::{Error, error}};

use super::{pointer_type, codegen_module::CodeGenModule};

// Layout of Types:
// num - just a f64
// str - pointer to a struct: <len:i64><content:&[u8]><0:u8>
// 

pub struct FunctionOptions {
    call_conv: CallConv,
    linkage: Linkage
}

impl FunctionOptions {
    pub fn external(config: &TargetFrontendConfig) -> Self {
        FunctionOptions { 
            call_conv: config.default_call_conv,
            linkage: Linkage::Export
        }
    }

    pub fn internal() -> Self {
        FunctionOptions { 
            call_conv: CallConv::Fast,
            linkage: Linkage::Local
        }
    }
}

pub struct TranslatedFunction<'a, M: Module> {
    codegen: &'a mut CodeGenModule<M>,

    pub context: Context
}

impl<'a, M: Module> TranslatedFunction<'a, M> {
    pub fn finish_func(mut self, name: &str, options: FunctionOptions) -> Result<(FuncId, Context), Error> {
        let mut sig = &mut self.context.func.signature;
        sig.call_conv = options.call_conv;

        let id = self
            .codegen
            .module
            .declare_function(name, options.linkage, sig)?;

        self
            .codegen
            .module
            .define_function(id, &mut self.context)?;

        Ok((id, self.context))
    }

    pub fn finish_anon_func(mut self, options: FunctionOptions) -> Result<(FuncId, Context), Error> {
        let mut sig = &mut self.context.func.signature;
        sig.call_conv = options.call_conv;

        let id = self
            .codegen
            .module
            .declare_anonymous_function(sig)?;

        self
            .codegen
            .module
            .define_function(id, &mut self.context)?;

        Ok((id, self.context))
    }
}

pub struct FunctionTranslator<'a, M: Module> {
    pub codegen: &'a mut CodeGenModule<M>,

    pub signature: TypedSignature,

    pub variables: HashMap<String, Value>,

    pub stack: Vec<Value>
}

impl<'a, M: Module> FunctionTranslator<'a, M> {
    pub fn new(
        codegen: &'a mut CodeGenModule<M>
    ) -> Self {
        FunctionTranslator { 
            codegen,
            signature: TypedSignature::default(),
            variables: HashMap::new(),
            stack: Vec::new()
        }
    }

    pub fn with_signature(mut self, sig: TypedSignature) -> FunctionTranslator<'a, M> {
        self.signature = sig;
        self
    }

    pub fn with_body(self, nodes: Vec<Node>) -> Result<TranslatedFunction<'a, M>, Error> {
        self.with_body_generator(|translator, builder| {
            translator.translate_nodes(nodes, builder)?;

            Ok(())
        })
    }

    pub fn with_body_generator<F>(mut self, gen: F) -> Result<TranslatedFunction<'a, M>, Error>
        where F: FnOnce(&mut FunctionTranslator<'a, M>, &mut FunctionBuilder) -> Result<(), Error>
    {
        let mut build_ctx = FunctionBuilderContext::new();

        let mut context = Context::new();
        let sig = self.codegen.build_cranelift_signature(&self.signature)?;
        context.func.signature = sig;

        // TODO This is computed even when not needed
        context.set_disasm(true);

        let mut builder = FunctionBuilder::new(
            &mut context.func,
            &mut build_ctx
        );

        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);

        let vals = builder.block_params(entry);
        self.stack.extend(vals);

        builder.seal_block(entry);

        gen(&mut self, &mut builder)?;

        let len = self.stack.len() - self.signature.returns().len();
        builder.ins().return_(&self.stack[len..]);

        builder.seal_all_blocks();
        builder.finalize();

        Ok(TranslatedFunction { 
            codegen: self.codegen, 
            context
        })
    }

    pub fn translate_nodes(&mut self, mut nodes: Vec<Node>, builder: &mut FunctionBuilder) -> Result<(), Error> {
        let transforms = self.codegen.transformations.clone();

        'outer: while !nodes.is_empty() {
            for transform in transforms.iter() {
                // Try to apply one of the known transforms
                if transform.try_apply(&mut nodes, self, builder)? {
                    continue 'outer;
                }
            }

            // Fall back to "normal" translations (TODO should be integrated with transforms)
            let top = nodes.remove(0);
            self.translate_single_node(top, builder)?;
        }

        Ok(())
    }

    fn translate_single_node(&mut self, node: Node, builder: &mut FunctionBuilder) -> Result<(), Error> {
        match node {
            Node::Assigment { name, .. } => {
                let node = self.pop_node();
                self.variables.insert(name, node);
            },
    
            Node::Variable { name, .. } => {
                let val = self.variables.get(&name)
                    .cloned()
                    .or_else(|| self.codegen.get_func_by_name(&name)
                        .ok()
                        .map(|func_id| {
                            let callee = self.codegen
                                .module
                                .declare_func_in_func(func_id, builder.func);

                            builder.ins().func_addr(pointer_type(), callee)
                        })
                    )
                    .ok_or_else(|| error(format!("Variable {name} not found - yes this is a compiler bug")))?;

                self.push_node(val);
            },
    
            Node::Call { name, arguments, .. } => 
                self.ins_call(name, arguments.len(), builder)?,
    
            Node::Literal { typ, value, .. } => {
                let val = self.build_literal(typ, value, builder)?;
                self.stack.push(val);
            }
        }

        Ok(())
    }

    fn build_literal(&mut self, typ: Type, literal: Literal, builder: &mut FunctionBuilder) -> Result<Value, Error> {
        if let Type::Kind(typ_name, _type_vars) = typ {
            match (typ_name.as_str(), literal) {
                (types::QUOTE_TYPE_NAME, Literal::Quote(value)) => {
                    let content = value.as_bytes().to_vec();
                    let len_buf = content.len().to_le_bytes();

                    let mut buffer: Vec<u8> = Vec::new();
                    buffer.extend(len_buf);  // Save str len
                    buffer.extend(content);  // Save content
                    buffer.push(0);          // Save 0 byte

                    let id = self.codegen.create_data(buffer)?;

                    let local_id = self
                        .codegen
                        .module
                        .declare_data_in_func(id, builder.func);

                    let pointer = pointer_type();
                    Ok(builder.ins().symbol_value(pointer, local_id))
                },
    
                (types::NUMBER_TYPE_NAME, Literal::Number(value)) => {
                    Ok(builder.ins().f64const(value))
                },

                (types::LIST_TYPE_NAME, Literal::List(ast)) => {
                    let stack_size_before = self.stack.len();

                    // Inline the list as if it is were function
                    self.translate_nodes(ast, builder)?;

                    // Collect the new values pushed by the list
                    let vals: Vec<Value> = self.stack.drain(stack_size_before..)
                        .rev()
                        .collect();

                    // Allocate the list on the heap
                    let len = builder.ins().iconst(cranelift::prelude::types::I64, vals.len() as i64);
                    let size = builder.ins().imul_imm(len, 8);
                    
                    self.push_node(size);
                    self.ins_call("malloc", 1, builder)?;

                    // Top-of-stack should be the address returned by malloc
                    let address = self.pop_node();

                    let flags = MemFlags::new();

                    // Write the list length
                    builder.ins().store(flags, len, address, 0);

                    // Write the list content
                    for (i, val) in vals.iter().enumerate(){
                        builder.ins().store(flags, *val, address, (i as i32 + 1) * 8);
                    }

                    Ok(address)
                },
    
                (types::FUNC_TYPE_NAME, Literal::Function(sig, ast)) => {
                    let (id, _) = FunctionTranslator::new(self.codegen)
                        .with_signature(sig)
                        .with_body(ast)?
                        .finish_anon_func(FunctionOptions::internal())?;

                    let local_callee = self
                        .codegen
                        .module
                        .declare_func_in_func(id, builder.func);

                    Ok(builder.ins().func_addr(pointer_type(), local_callee))
                },
    
                _ => unreachable!()
            }
    
        }
        else { unreachable!() }
    }

    pub fn ins_call<S: AsRef<str>>(&mut self, name: S, args_len: usize, builder: &mut FunctionBuilder) -> Result<(), Error> {
        let func_id = self.codegen.get_func_by_name(name.as_ref())?;
        
        let local_callee = self
            .codegen
            .module
            .declare_func_in_func(func_id, builder.func);

        let range = (self.stack.len() - args_len)..;

        let slice: Vec<Value> = self.stack.drain(range).collect();
        let call = builder.ins().call(local_callee, &slice[..]);

        let results = builder.inst_results(call);
        self.stack.extend_from_slice(results);

        Ok(())
    }

    pub fn pop_node(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    pub fn push_node(&mut self, val: Value) {
        self.stack.push(val)
    }
}