use std::{collections::HashMap, sync::{Mutex, Arc}};

use super::{typelist::TypeList, typ::Type};

pub type TypeBindings = HashMap<String, Type>;

#[derive(Clone, Debug)]
pub struct TypeEnv {
    pub var_counter: u32,
    pub stack: TypeList,
    pub bindings: TypeBindings
}

impl TypeEnv {
    pub fn new(bindings: &TypeBindings) -> Self {
        Self {
            var_counter: 0,
            stack: TypeList::new(),
            bindings: bindings.clone()
        }
    }

    pub fn new_var(&mut self, name: String, val: Option<Type>) -> Type {
        let name = format!("{name}{}", self.var_counter);
        self.var_counter += 1;

        Type::Variable(name, Arc::new(Mutex::new(val)))
    }
}