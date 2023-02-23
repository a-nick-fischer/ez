use cranelift::prelude::FunctionBuilder;
use cranelift_module::{Module, Linkage};

use crate::{parser::{signature_parser::TypedSignature, node::Node}, codegen::{function_translator::FunctionTranslator, codegen_module::CodeGenModule}, error::Error};

trait StdFun {
    fn init<M: Module>(&self, module: &mut CodeGenModule<M>) -> Result<(), Error>;

    fn try_apply<'b, M: Module>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error>;
}

struct NativeFun {
    name: String,

    sig: TypedSignature
}

impl NativeFun {
    fn new(name: &str, sig: &str) -> Result<Self, Error> {
        Ok(Self {
            name: name.to_string(),

            sig: sig.parse()?
        })
    }
}

impl StdFun for NativeFun {
    fn init<M: Module>(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        let mut sig = codegen.build_cranelift_signature(&self.sig)?;
        sig.call_conv = codegen.module.target_config().default_call_conv;
    
        codegen.module
            .declare_function(self.name.as_str(), Linkage::Import, &sig)?;

        Ok(())
    }

    fn try_apply<'b, M: Module>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error> {

        match nodes.last() {
            Some(Node::Call { name, .. }) if name == &self.name => {
                translator.ins_call(name, self.sig.arguments().len(), builder)?;

                nodes.pop().expect("Beeing able to pop an element of the stack");
                Ok(true)
            },

            _ => Ok(false)
        }        
    }
}