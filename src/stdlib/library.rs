use cranelift_module::Module;

use crate::parser::types::type_env::TypeBindings;

use super::functions::EzFun;

pub struct Library<M: Module> {
    bindings: TypeBindings,

    functions: Vec<Box<dyn EzFun<M>>>
}

