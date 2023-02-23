use std::collections::HashMap;

use cranelift::{prelude::{FunctionBuilder, Value, Variable, EntityRef, InstBuilder, FunctionBuilderContext, isa::{CallConv, TargetFrontendConfig}}, codegen::Context};
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

    signature: TypedSignature,

    pub context: Context
}

impl<'a, M: Module> TranslatedFunction<'a, M> {
    pub fn finish_func(mut self, name: &str, options: FunctionOptions) -> Result<(FuncId, Context), Error> {
        let mut sig = self.codegen.build_cranelift_signature(self.signature)?;
        sig.call_conv = options.call_conv;

        let id = self
            .codegen
            .module
            .declare_function(name, options.linkage, &sig)?;

        self
            .codegen
            .module
            .define_function(id, &mut self.context)?;

        Ok((id, self.context))
    }

    pub fn finish_anon_func(mut self, options: FunctionOptions) -> Result<(FuncId, Context), Error> {
        let mut sig = self.codegen.build_cranelift_signature(self.signature)?;
        sig.call_conv = options.call_conv;

        let id = self
            .codegen
            .module
            .declare_anonymous_function(&sig)?;

        self
            .codegen
            .module
            .define_function(id, &mut self.context)?;

        Ok((id, self.context))
    }
}

pub struct FunctionTranslator<'a, M: Module> {
    codegen: &'a mut CodeGenModule<M>,

    signature: TypedSignature,

    variables: HashMap<String, Variable>,

    stack: Vec<Value>
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

    pub fn with_body(mut self, nodes: Vec<Node>) -> Result<TranslatedFunction<'a, M>, Error> {
        let mut build_ctx = FunctionBuilderContext::new();

        let mut context = Context::new();
        context.set_disasm(true); // TODO This is computed even when not needed

        let mut builder = FunctionBuilder::new(
            &mut context.func,
            &mut build_ctx
        );

        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        for node in nodes {
            self.translate_node(node, &mut builder)?;
        }

        let len = self.stack.len() - self.signature.returns().len();
        builder.ins().return_(&self.stack[len..]);

        builder.seal_all_blocks();
        builder.finalize();

        Ok(TranslatedFunction { 
            codegen: self.codegen, 
            context, 
            signature: self.signature 
        })
    }

    fn translate_node(&mut self, node: Node, builder: &mut FunctionBuilder) -> Result<(), Error> {
        match node {
            Node::Assigment { name, .. } => {
                let var = Variable::new(self.variables.len());
                self.variables.insert(name, var);

                let content = self.stack.pop().unwrap();
                builder.def_var(var, content);
            },
    
            Node::Variable { name, .. } => {
                let var = self.variables.get(&name)
                    .ok_or_else(|| error(format!("Variable {name} not found - yes this is a compiler bug")))?;

                let val = builder.use_var(*var);
                self.stack.push(val);
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

                (types::LIST_TYPE_NAME, Literal::List(_ast)) => {
                    todo!()
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

    fn ins_call<S: AsRef<str>>(&mut self, name: S, args_len: usize, builder: &mut FunctionBuilder) -> Result<(), Error> {
        let func_id = self.codegen.get_func_by_name(name.as_ref())?;
        
        let local_callee = self
            .codegen
            .module
            .declare_func_in_func(func_id, builder.func);

        let range = (self.stack.len() - args_len) .. self.stack.len();

        let slice: Vec<Value> = self.stack.drain(range).collect();
        let call = builder.ins().call(local_callee, &slice[..]);

        let results = builder.inst_results(call);
        self.stack.extend_from_slice(results);

        Ok(())
    }
}