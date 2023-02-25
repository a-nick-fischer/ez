use cranelift_module::Module;

use crate::parser::types::type_env::TypeBindings;

use super::functions::CodeTransformation;

pub struct Library<M: Module> {
    pub bindings: TypeBindings,

    pub transformations: Vec<Box<dyn CodeTransformation<M>>>
}

impl<M: Module> Library<M> {
    pub fn new() -> Self {
        Self {
            bindings: TypeBindings::new(),

            transformations: Vec::new()
        }
    }
}