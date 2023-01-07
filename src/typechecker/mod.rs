pub mod assigment;
pub mod signature;
pub mod type_env;
pub mod type_node;
pub mod types;

use std::{rc::Rc, cell::RefCell};

use crate::{error::TErr, lexer::token::Token};

use self::{types::Type, type_env::{TypeEnv, TypeEnvModifier}, type_node::TypeNode};

pub fn string_type() -> Type {
    Type::Kind("str".to_string(), vec![])
}

pub fn number_type() -> Type {
    Type::Kind("num".to_string(), vec![])
}

pub fn func_type(args: Vec<Type>, result: Vec<Type>) -> Type {
    Type::Kind("fun".to_string(), vec![
        Type::Kind("arg".to_string(), args),
        Type::Kind("ret".to_string(), result)
    ])
}

pub fn var_type(name: &str) -> Type {
    Type::Variable(name.to_string(), Rc::new(RefCell::new(None)))
}

pub fn typecheck(tokens: Vec<Token>, init_env: TypeEnv) -> Result<(TypeEnv, Vec<TypeNode>), TErr> {
    let token_to_node = |acc: (TypeEnv, Vec<TypeNode>), token: Token| {
        let (current_env, mut buffer) = acc;
        let maybe_sig = TypeEnvModifier::try_from(token);

        match maybe_sig.clone().and_then(|sig| sig.apply(&current_env)) {
            Ok(env) => {
                buffer.push(TypeNode {
                    token,
                    delta: maybe_sig.unwrap()
                });

                Ok((env, buffer))
            },

            Err(msg) => Err(token.to_error(msg))
        }
    };

    let (tenv, nodes) = tokens
        .into_iter()
        .try_fold((init_env, vec![]), token_to_node)?;

    Ok((tenv, nodes))
}