use cranelift::prelude::*;
use cranelift::prelude::isa::TargetFrontendConfig;
use cranelift::prelude::FunctionBuilderContext;
use cranelift_module::{Module, DataContext, Linkage, DataId, FuncId, FuncOrDataId};

use crate::error::{Error, error};
use crate::lexer::sig_lexer::lex_signature;
use crate::parser::signature_parser::TypedSignature;
use crate::parser::node::Node;

use super::function_translator::{FunctionTranslator, FunctionOptions};

pub struct CodeGen<M: Module> {
    builder_context: FunctionBuilderContext,

    data_ctx: DataContext,

    pub module: M,

    naming_idx: u32
}

impl<M: Module> CodeGen<M> {
    pub fn new(module: M) -> Self {
        CodeGen {
            builder_context: FunctionBuilderContext::new(),
            data_ctx: DataContext::new(),
            module,
            naming_idx: 0
        }
    }

    pub fn translate(&mut self, nodes: Vec<Node>, options: FunctionOptions) -> Result<FunctionTranslator<M>, Error> {
        let mut tran = FunctionTranslator::new(
            self, 
            &mut self.builder_context,
            options
        );
        
        for node in nodes {
            tran.translate_node(node)?;
        }

        Ok(tran)
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

    pub fn gen_name(&mut self, prefix: &str) -> String {
        self.naming_idx += 1;
        format!("{}{}", prefix, self.naming_idx)
    }

    pub fn declare_external_func(&mut self, name: &str, sig_src: &str) -> Result<FuncId, Error> {
        let sig = self.parse_cranelift_signature(sig_src)?;
    
        let func_id = self.module
            .declare_function(name, Linkage::Import, &sig)
            .expect("problem declaring function");
    
        return Ok(func_id)
    }

    pub fn target_config(&self) -> TargetFrontendConfig {
        self.module.target_config()
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