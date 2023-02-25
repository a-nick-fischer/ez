use cranelift::prelude::FunctionBuilder;
use cranelift_module::{Module, Linkage};

use crate::{parser::{signature_parser::TypedSignature, node::Node}, codegen::{function_translator::{FunctionTranslator, FunGenerator, FunctionOptions}, codegen_module::CodeGenModule, self}, error::Error, match_nodes};

pub trait EzFun<M: Module> {
    fn init(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error>;

    fn try_apply<'b>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error>;
}

pub struct NativeFun<'a> {
    name: &'a str,

    sig: TypedSignature
}

impl<'a> NativeFun<'a> {
    fn new(name: &'a str, sig: &str) -> Result<Self, Error> {
        Ok(Self {
            name,

            sig: sig.parse()?
        })
    }
}

impl<M: Module> EzFun<M> for NativeFun<'_> {
    fn init(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        let mut sig = codegen.build_cranelift_signature(&self.sig)?;
        sig.call_conv = codegen.module.target_config().default_call_conv;
    
        codegen.module
            .declare_function(self.name, Linkage::Import, &sig)?;

        Ok(())
    }

    fn try_apply<'b>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error> {

        match_nodes!(
            nodes(1): [Node::Call { name, .. }] if name == &self.name => {
                translator.ins_call(name, self.sig.arguments().len(), builder)?;
            }
        )
    }
}


struct SimpleFun<'b, M: Module>  {
    name: &'b str,

    sig: TypedSignature,

    generator: FunGenerator<'b, M>
}

impl<'b, M: Module> SimpleFun<'b, M> {
    fn new(name: &'b str, sig: &str, generator: FunGenerator<'b, M>) -> Result<Self, Error> {
        Ok(Self {
            name,

            sig: sig.parse()?,

            generator
        })
    }
}

impl<M: Module> EzFun<M> for SimpleFun<'_, M> {
    fn init(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        FunctionTranslator::new(codegen)
            .generate_body(self.generator)?
            .finish_func(self.name, FunctionOptions::internal())?;

        Ok(())
    }

    fn try_apply<'b>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error> {

        match_nodes!(
            nodes(1): [Node::Call { name, .. }] if name == self.name => {
                translator.ins_call(name, self.sig.arguments().len(), builder)?;
            }
        )
    }
}