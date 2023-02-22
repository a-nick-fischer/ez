use cranelift::prelude::*;
use cranelift::prelude::isa::TargetFrontendConfig;
use cranelift_module::{Module, DataContext, Linkage, DataId, FuncId, FuncOrDataId};

use crate::error::{Error, error};
use crate::lexer::sig_lexer::lex_signature;
use crate::parser::signature_parser::TypedSignature;
use crate::parser::node::Node;

use super::function_translator::{FunctionTranslator, TranslatedFunction};

pub struct CodeGenModule<M: Module> {
    data_ctx: DataContext,

    pub module: M,
}

impl<M: Module> CodeGenModule<M> {
    pub fn new(module: M) -> Self {
        CodeGenModule {
            data_ctx: DataContext::new(),
            module
        }
    }

    pub fn translate_ast(&mut self, nodes: Vec<Node>) -> Result<TranslatedFunction<M>, Error> {
        FunctionTranslator::new(self).with_body(nodes)
    }


    pub fn create_data(&mut self, content: Vec<u8>) -> Result<DataId, Error> {
        self.data_ctx.define(content.into_boxed_slice());

        let id = self
            .module
            .declare_anonymous_data(false, false)?;

        self.module
            .define_data(id, &self.data_ctx)?;

        // self.data_ctx.clear(); // TODO Needed?

        Ok(id)
    }

    pub fn get_func_by_name(&self, name: &str) -> Result<FuncId, Error> {
        let maybe_func = self
            .module
            .declarations()
            .get_name(name);

        match maybe_func {
            Some(FuncOrDataId::Func(id)) => Ok(id),

            _ => Err(error(format!("{name} not a function - yes this is a compiler bug"))),
        }
    }

    pub fn declare_external_func(&mut self, name: &str, sig_src: &str) -> Result<FuncId, Error> {
        let mut sig = self.parse_cranelift_signature(sig_src)?;
        sig.call_conv = self.module.target_config().default_call_conv;
    
        let func_id = self.module
            .declare_function(name, Linkage::Import, &sig)
            .expect("problem declaring external function");
    
        Ok(func_id)
    }

    pub fn declare_internal_func(&mut self, name: &str, typed_sig: TypedSignature) -> Result<FuncId, Error> {
        let sig = self.build_cranelift_signature(typed_sig)?;

        let func_id = self.module
            .declare_function(name, Linkage::Local, &sig)
            .expect("problem declaring internal function");

        Ok(func_id)
    }

    pub fn parse_cranelift_signature(&self, sig_src: &str) -> Result<Signature, Error> {
        let lexed_sig = lex_signature(sig_src)?;
    
        self.build_cranelift_signature(lexed_sig.into())
    }

    pub fn build_cranelift_signature(&self, sig: TypedSignature) -> Result<Signature, Error> {
        let mut cranelift_cig = self.module.make_signature();
    
        let params: Vec<AbiParam> = sig.arguments().clone().into();
        cranelift_cig.params.extend(params);
    
        let returns: Vec<AbiParam> = sig.returns().clone().into();
        cranelift_cig.returns.extend(returns);

        Ok(cranelift_cig)
    }
}