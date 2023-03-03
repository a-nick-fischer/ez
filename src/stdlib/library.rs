use cranelift_module::Module;

use crate::{parser::types::type_env::{TypeBindings, TypeEnv}, codegen::codegen_module::CodeGenModule, error::Error};

use super::functions::{CodeTransformation, EzFun, FuncCodeTransformation};

pub type Transformations = Vec<Box<dyn CodeTransformation>>;
pub type Functions = Vec<Box<dyn EzFun>>;

#[derive(Default)]
pub struct Library {
    pub bindings: TypeBindings,

    pub functions: Functions,

    pub transformations: Transformations
}

impl Library {
    pub fn init_codegen<M: Module>(self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        for func in self.functions {
            func.init(codegen)?;

            let transform = Box::new(FuncCodeTransformation { inner: func });
            codegen.transformations.push(transform);
        }

        codegen.transformations.extend(self.transformations);

        Ok(())
    }

    pub fn type_env(&self) -> TypeEnv {
        TypeEnv::new(&self.bindings)
    }
}