use crate::{env_modifier::EnvModifier, error::TErr};

use super::type_env::TypeEnv;

#[derive(Debug, Clone)]
pub struct Assigment {
    var_name: String
}

impl Assigment {
    pub fn new(var_name: String) -> Self {
        Assigment { var_name }
    }
}

impl EnvModifier<TypeEnv> for Assigment {
    fn apply(&self, env: &mut TypeEnv) -> Result<(), TErr> {
        if let Some(typ) = env.stack.pop() {
            env.bindings.insert(self.var_name.to_string(), typ);

            Ok(())
        } else {
            panic!("Empty stack, no value to assign to {}", self.var_name)
        }
    }
}