use std::rc::Rc;

use crate::{lexer::token::Token, env_modifier::EnvModifier};

use super::type_env::TypeEnv;

#[derive(Debug, Clone)]
pub struct TypeNode {
    pub token: Token,
    pub delta: Rc<dyn EnvModifier<TypeEnv>>
}