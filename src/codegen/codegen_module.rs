use cranelift::prelude::*;
use cranelift_module::{Module, DataContext, DataId, FuncId, FuncOrDataId};

use crate::error::{Error, error};
use crate::parser::signature_parser::TypedSignature;
use crate::parser::node::Node;
use crate::stdlib::library::Transformations;

use super::function_translator::{FunctionTranslator, TranslatedFunction};

pub struct CodeGenModule<M: Module> {
    data_ctx: DataContext,

    pub module: M,

    pub transformations: Transformations<M>
}

impl<M: Module> CodeGenModule<M> {
    pub fn new(module: M) -> Self {
        CodeGenModule {
            data_ctx: DataContext::new(),
            transformations: Vec::new(),
            module
        }
    }

    pub fn translate_ast(&mut self, sig: TypedSignature, nodes: Vec<Node>) -> Result<TranslatedFunction<M>, Error> {
        FunctionTranslator::new(self)
            .with_signature(sig)
            .with_body(nodes)
    }

    pub fn create_data(&mut self, content: Vec<u8>) -> Result<DataId, Error> {
        self.data_ctx.define(content.into_boxed_slice());

        let id = self
            .module
            .declare_anonymous_data(false, false)?;

        self.module
            .define_data(id, &self.data_ctx)?;

        // Has to be cleared so we can overwrite it later
        self.data_ctx.clear();

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

    pub fn build_cranelift_signature(&self, sig: &TypedSignature) -> Result<Signature, Error> {
        let mut cranelift_cig = self.module.make_signature();
    
        let params: Vec<AbiParam> = sig.arguments().clone().into();
        cranelift_cig.params.extend(params);
    
        let returns: Vec<AbiParam> = sig.returns().clone().into();
        cranelift_cig.returns.extend(returns);

        Ok(cranelift_cig)
    }
}