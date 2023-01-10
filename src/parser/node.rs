use crate::{error::TErr, lexer::token::Token};

use super::{type_env::TypeEnv, typelist::TypeList, types::types::Type};

#[derive(Clone, Debug)]
pub enum Node {
    Assigment {
        name: String,
        token: Token,
        typ: Type
    },

    Variable {
        name: String,
        token: Token,
        typ: Type
    },

    Call {
        name: String,
        token: Token,
        arguments: TypeList,
        returns: TypeList
    },

    Literal {
        typ: Type,
        token: Token,
    }
}

impl Node {
    pub fn apply(&self, env: &mut TypeEnv) -> Result<(), TErr> {
        match self {
            Node::Assigment { name, token, typ } => todo!(),
            Node::Variable { name, token, typ } => todo!(),
            Node::Call { name, token, arguments, returns } => todo!(),
            Node::Literal { typ, token } => todo!(),
        }
    }
}

fn call(arguments: &TypeList, returns: &TypeList, env: &mut TypeEnv) -> Result<(), TErr>  {
    let args = arguments.vec();
    let res = returns.vec();

    let arg_len = args.len();
    let stack_len = env.stack.len();

    if arg_len > stack_len {
        panic!("Expected {} elem on the stack, got {}", arg_len, stack_len) // TODO Error handling
    }

    let mut tenv = env.clone();

    for i in (0..arg_len).rev() {
        let stack_args = &tenv.stack.pop().unwrap().refresh_vars(&mut tenv);

        args[i].unify(stack_args).unwrap(); // TODO Error handling
    }

    tenv.stack.extend(returns.concretize());

    arguments.clear_vars();
    returns.clear_vars();

    *env = tenv;

    Ok(())
}
