use cranelift_module::Module;

use crate::parser::types::type_env::{TypeBindings, TypeEnv};

use super::functions::CodeTransformation;

pub type Transformations<M> = Vec<Box<dyn CodeTransformation<M>>>;

pub struct Library<M: Module> {
    pub bindings: TypeBindings,

    pub transformations: Transformations<M>
}

impl<M: Module> Library<M> {
    pub fn new() -> Self {
        Self {
            bindings: TypeBindings::new(),

            transformations: Vec::new()
        }
    }

    pub fn type_env(&self) -> TypeEnv {
        TypeEnv::new(&self.bindings)
    }
}