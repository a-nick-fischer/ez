use std::collections::HashMap;

use cranelift::{prelude::{FunctionBuilder, Value, Variable, EntityRef, InstBuilder, FunctionBuilderContext, isa::{CallConv, TargetFrontendConfig}}, codegen::Context};
use cranelift_module::{Module, Linkage, FuncId};

use crate::{parser::{node::{Node, Literal}, types::{types::Type, self}, signature_parser::TypedSignature}, error::{Error, error}};

use super::{codegen::CodeGen, pointer_type};

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

pub struct FunctionTranslator<'a, M: Module> {
    builder: FunctionBuilder<'a>,

    codegen: &'a mut CodeGen<M>,

    pub context: Context,

    options: FunctionOptions,

    variables: HashMap<String, Variable>,

    stack: Vec<Value>
}

impl<'a, M: Module> FunctionTranslator<'a, M> {
    pub fn new(
        codegen: &'a mut CodeGen<M>, 
        builder_context: &'a mut FunctionBuilderContext,
        options: FunctionOptions
    ) -> Self {
        let mut context = Context::new();
        context.set_disasm(true); // TODO This is computed even when not needed

        let builder = FunctionBuilder::new(
            &mut context.func, 
            builder_context
        );

        FunctionTranslator { 
            builder, 
            codegen,
            context,
            options,
            variables: HashMap::new(),
            stack: Vec::new()
        }
    }

    pub fn to_func(&mut self, name: &str, sig: TypedSignature) -> Result<FuncId, Error> {
        let mut sig = self.codegen.build_cranelift_signature(sig)?;
        sig.call_conv = self.options.call_conv;

        let id = self
            .codegen
            .module
            .declare_function(name, self.options.linkage, &sig)?;

        self
            .codegen
            .module
            .define_function(id, &mut self.context)?;

        Ok(id)
    }

    pub fn to_anon_func(&mut self, sig: TypedSignature) -> Result<FuncId, Error> {
        let mut sig = self.codegen.build_cranelift_signature(sig)?;
        sig.call_conv = self.options.call_conv;

        let id = self
            .codegen
            .module
            .declare_anonymous_function(&sig)?;

        self
            .codegen
            .module
            .define_function(id, &mut self.context)?;

        Ok(id)
    }

    pub fn translate_node(&mut self, node: Node) -> Result<(), Error> {
        match node {
            Node::Assigment { name, .. } => {
                let var = Variable::new(self.variables.len());
                self.variables.insert(name, var);

                let content = self.stack.pop().unwrap();
                self.builder.def_var(var, content);
            },
    
            Node::Variable { name, .. } => {
                let var = self.variables.get(&name)
                    .ok_or(error(format!("Variable {name} not found - yes this is a compiler bug")))?;

                let val = self.builder.use_var(*var);
                self.stack.push(val);
            },
    
            Node::Call { name, arguments, .. } => 
                self.ins_call(name, arguments.len())?,
    
            Node::Literal { typ, value, .. } => {
                let val = self.build_literal(typ, value)?;
                self.stack.push(val);
            }
        }

        Ok(())
    }

    fn build_literal(&mut self, typ: Type, literal: Literal) -> Result<Value, Error> {
        if let Type::Kind(typ_name, _type_vars) = typ {
            match (typ_name.as_str(), literal) {
                (types::QUOTE_TYPE_NAME, Literal::Quote(value)) => {
                    let id = self.codegen.create_data(
                        value.as_bytes().to_vec()
                    )?;

                    let local_id = self
                        .codegen
                        .module
                        .declare_data_in_func(id, self.builder.func);

                    let pointer = pointer_type();
                    Ok(self.builder.ins().symbol_value(pointer, local_id))
                },
    
                (types::NUMBER_TYPE_NAME, Literal::Number(value)) => {
                    Ok(self.builder.ins().f64const(value))
                },

                (types::LIST_TYPE_NAME, Literal::List(_ast)) => {
                    todo!() // TODO No fcking clue how to translate this one..
                },
    
                (types::FUNC_TYPE_NAME, Literal::Function(sig, ast)) => {
                    let func = self.codegen.translate(ast, FunctionOptions::internal())?;

                    let id = func.to_anon_func(sig)?;

                    let local_callee = self
                        .codegen
                        .module
                        .declare_func_in_func(id, self.builder.func);

                    Ok(self.builder.ins().func_addr(pointer_type(), local_callee))
                },
    
                _ => unreachable!()
            }
    
        }
        else { unreachable!() }
    }

    fn ins_call<S: AsRef<str>>(&mut self, name: S, args_len: usize) -> Result<(), Error> {
        let func_id = self.codegen.get_func_by_name(name.as_ref())?;
        
        let local_callee = self
            .codegen
            .module
            .declare_func_in_func(func_id, &mut self.builder.func);

        let slice = &self.stack[args_len..];
        let call = self.builder.ins().call(local_callee, slice);

        let results = self.builder.inst_results(call);
        self.stack.extend_from_slice(results);

        Ok(())
    }
}