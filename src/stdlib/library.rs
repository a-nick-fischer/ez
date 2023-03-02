use std::rc::Rc;

use cranelift_module::Module;

use crate::{parser::types::type_env::{TypeBindings, TypeEnv}, codegen::codegen_module::CodeGenModule, error::Error};

use super::functions::{CodeTransformation, EzFun};

pub type Transformations<M> = Vec<Rc<dyn CodeTransformation<M>>>;
pub type Functions<M> = Vec<Rc<dyn EzFun<M>>>;

pub struct Library<M: Module> {
    pub bindings: TypeBindings,

    pub functions: Functions<M>,

    pub transformations: Transformations<M>
}

impl<M: Module> Library<M> {
    pub fn new() -> Self {
        Self {
            bindings: TypeBindings::new(),

            functions: Vec::new(),

            transformations: Vec::new()
        }
    }

    pub fn init_with(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        for func in &self.functions {
            func.init(codegen)?;
        }

        Ok(())
    }

    pub fn type_env(&self) -> TypeEnv {
        TypeEnv::new(&self.bindings)
    }
}