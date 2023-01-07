use std::{cell::RefCell, rc::Rc, collections::HashMap};

use crate::env_modifier::EnvModifier;

use super::types::Type;

pub type TypeEnvModifier = Rc<dyn EnvModifier<TypeEnv>>;

#[derive(Clone, Debug)]
pub struct TypeEnv {
    pub var_counter: u32,
    pub stack: Vec<Type>,
    pub bindings: HashMap<String, Type>
}

impl TypeEnv {
    pub fn new(bindings: &HashMap<String, Type>) -> Self {
        Self {
            var_counter: 0,
            stack: vec![],
            bindings: bindings.clone()
        }
    }

    pub fn new_var(&mut self, name: String, val: Option<Type>) -> Type {
        let name = format!("{name}{}", self.var_counter);
        self.var_counter += 1;

        Type::Variable(name, Rc::new(RefCell::new(val)))
    }
}