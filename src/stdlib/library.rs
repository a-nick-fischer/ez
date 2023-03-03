use cranelift_module::Module;

use crate::{parser::types::type_env::{TypeBindings, TypeEnv}, codegen::codegen_module::CodeGenModule, error::Error};

use super::functions::{CodeTransformation, EzFun, FuncCodeTransformation};

pub type Transformations<M> = Vec<Box<dyn CodeTransformation<M>>>;
pub type Functions<M> = Vec<Box<dyn EzFun<M>>>;

pub struct Library<M: Module> {
    pub bindings: TypeBindings,

    pub functions: Functions<M>,

    pub transformations: Transformations<M>
}

impl<M: Module + 'static> Library<M> {
    pub fn new() -> Self {
        Self {
            bindings: TypeBindings::new(),

            functions: Functions::new(),

            transformations: Transformations::new()
        }
    }


    pub fn init_codegen(self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
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