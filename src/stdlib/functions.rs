use cranelift::prelude::FunctionBuilder;
use cranelift_module::{Module, Linkage};

use crate::{parser::{signature_parser::TypedSignature, node::Node}, codegen::{function_translator::{FunctionTranslator}, codegen_module::CodeGenModule}, error::Error, match_nodes};

pub trait CodeTransformation<M: Module> {
    fn try_apply<'b>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error>;
}

pub trait EzFun<M: Module>: CodeTransformation<M> {
    fn init(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error>;

    fn name(&self) -> &str;

    fn signature(&self) -> TypedSignature;
}

// Black magic f*ckery
impl<M: Module, T: EzFun<M>> CodeTransformation<M> for T {
    fn try_apply<'b>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error> {

        match_nodes!(
            nodes(1): [Node::Call { name, .. }] if name == self.name() => {
                translator.ins_call(name, self.signature().arguments().len(), builder)?;
            }
        )
    }
}

pub struct NativeFun<'a> {
    name: &'a str,

    sig: TypedSignature
}

impl<'a> NativeFun<'a> {
    pub fn new(name: &'a str, sig: &str) -> Result<Self, Error> {
        Ok(Self {
            name,

            sig: sig.parse()?
        })
    }
}

impl<'a, M: Module> EzFun<M> for NativeFun<'a> {
    fn init(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        let mut sig = codegen.build_cranelift_signature(&self.sig)?;
        sig.call_conv = codegen.module.target_config().default_call_conv;
    
        codegen.module
            .declare_function(self.name, Linkage::Import, &sig)?;

        Ok(())
    }

    fn name(&self) -> &str {
        self.name
    }

    fn signature(&self) -> TypedSignature {
        self.sig.clone()
    }
}
