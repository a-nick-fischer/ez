use cranelift::prelude::*;
use cranelift::{codegen, prelude::{FunctionBuilderContext}};
use cranelift_module::{Module, DataContext, Linkage, DataId, FuncId, FuncOrDataId};

use crate::error::{Error, error};
use crate::lexer::sig_lexer::lex_signature;
use crate::parser::signature_parser::parse_signature;
use crate::parser::node::Node;

use super::function_translator::FunctionTranslator;

pub struct CodeGen<M: Module> {
    pub builder_context: FunctionBuilderContext,

    pub ctx: codegen::Context,

    pub data_ctx: DataContext,

    pub module: M,

    pub naming_idx: u32
}

impl<M: Module> CodeGen<M> {
    pub fn translate(&mut self, name: Option<&str>, nodes: Vec<Node>) -> Result<FuncId, Error> {
        let mut tran = FunctionTranslator::new(self);
        
        for node in nodes {
            tran.translate_node(node)?;
        }

        let sig = self.module.make_signature();
        let id = self.create_func(name, sig)?;

        Ok(id)
    }

    pub fn create_data(&mut self, name: String, content: Vec<u8>) -> Result<DataId, Error> {
        self.data_ctx.define(content.into_boxed_slice());

        let id = self
            .module
            .declare_data(&name, Linkage::Export, true, false)?;

        self.module
            .define_data(id, &self.data_ctx)?;

        // self.data_ctx.clear(); // TODO Needed?

        Ok(id)
    }

    pub fn create_func(&mut self, maybe_name: Option<&str>, sig: Signature) -> Result<FuncId, Error> {
        let id = 
            if let Some(name) = maybe_name {
                self
                    .module
                    .declare_function(name, Linkage::Export, &sig)?
            }
            else {
                self
                    .module
                    .declare_anonymous_function(&sig)?
            };
        
        self.module
            .define_function(id, &mut self.ctx)?;

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
        let lexed_sig = lex_signature(sig_src)?;
        let (parsed_args, parsed_returns) = parse_signature(lexed_sig);
    
        let mut sig = self.module.make_signature();
    
        let params: Vec<AbiParam> = parsed_args.into();
        sig.params.extend(params);
    
        let returns: Vec<AbiParam> = parsed_returns.into();
        sig.returns.extend(returns);
    
        let func_id = self.module
            .declare_function(name, Linkage::Import, &sig)
            .expect("problem declaring function");
    
        return Ok(func_id)
    }
}