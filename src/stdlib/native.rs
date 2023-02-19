use cranelift_module::Module;

use crate::{parser::signature_parser::TypedSignature, codegen::{function_translator::FunctionTranslator, codegen::CodeGen}, error::Error};

trait StdFn<'a> {
    fn name(&self) -> &'a str;

    fn sig(&self) -> TypedSignature;

    fn try_apply<'b, M: Module>(&self, translator: &mut FunctionTranslator<'b, M>) -> bool;
}