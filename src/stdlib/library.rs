use cranelift_module::Module;

use crate::{parser::types::type_env::{TypeBindings, TypeEnv}, codegen::codegen_module::CodeGenModule, error::Error};

use super::functions::{CodeTransformation, EzFun};

pub type Transformations<M> = Vec<Box<dyn CodeTransformation<M>>>;

pub struct Library<M: Module> {
    bindings: TypeBindings,

    functions: Vec<Box<dyn EzFun<M>>>,

    transformations: Transformations<M>
}

impl<M: Module> Library<M> {
    pub fn new() -> Self {
        Self {
            bindings: TypeBindings::new(),

            functions: Vec::new(),

            transformations: Vec::new()
        }
    }

    pub fn init_in(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        for func in self.functions {
            func.init(&mut codegen)?;
        }

        Ok(())
    }

    pub fn type_env(&self) -> TypeEnv {
        TypeEnv::new(&self.bindings)
    }

    pub fn into_transformations(self) -> Transformations<M> {
        self.transformations.extend(self.functions);
        self.transformations
    }
}