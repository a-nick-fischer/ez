use crate::{error::Error, lexer::token::Token};

use super::{type_env::TypeEnv, typelist::TypeList, types::types::Type, signature_parser::TypedSignature};

#[derive(Clone, Debug)]
pub enum Node {
    Assigment {
        name: String,
        token: Token,
        typ: Type,
        stack_size: usize
    },

    Variable {
        name: String,
        token: Token,
        typ: Type,
        stack_size: usize
    },

    Call {
        name: String,
        token: Token,
        arguments: TypeList,
        returns: TypeList,
        stack_size: usize
    },

    Literal {
        typ: Type,
        value: Literal,
        token: Token,
        stack_size: usize
    }
}

#[derive(Clone, Debug)]
pub enum Literal {
    Quote(String),

    Number(f64),

    List(Vec<Node>),

    Function(TypedSignature, Vec<Node>)
}

impl Node {
    pub fn stack_size_before(&self) -> usize {
        match self {
            Node::Assigment { stack_size, .. } |
            Node::Variable { stack_size, .. } |
            Node::Call { stack_size, .. } |
            Node::Literal { stack_size, .. } => *stack_size,
        }
    }

    pub fn stack_size_after(&self) -> usize {
        match self {
            Node::Assigment { stack_size, .. } => stack_size - 1,

            Node::Variable { stack_size, .. } => stack_size + 1,
            
            Node::Call { stack_size, arguments, returns, .. } => stack_size + arguments.len() - returns.len(),
            
            Node::Literal { stack_size, .. } => stack_size + 1,
        }
    }

    pub fn apply(&self, env: &mut TypeEnv) -> Result<(), Error> {
        match self {
            Node::Assigment { name, typ, .. } => {
                env.bindings.insert(name.clone(), typ.clone());
                Ok(())
            },

            Node::Variable { typ, .. } => {
                env.stack.push(typ.clone());
                Ok(())
            },

            Node::Call { name, arguments, returns, token, .. } => {            
                let arg_len = arguments.len();
                let stack_len = env.stack.len();
            
                if arg_len > stack_len {
                    return Err(Error::WrongArguments { 
                        fname: name.clone(), token: token.clone(), expected: arguments.clone(), got: env.stack.clone() 
                    })
                }

                let args = arguments.vec();
            
                let mut tenv = env.clone();
            
                for i in (0..arg_len).rev() {
                    let stack_args = &tenv.stack.pop().unwrap().refresh_vars(&mut tenv);
            
                    args[i].unify(stack_args)
                        .map_err(|_| Error::WrongArguments { 
                            fname: name.clone(), token: token.clone(), expected: arguments.clone(), got: env.stack.clone() 
                        })?;
                }
            
                tenv.stack.extend(returns.concretize());
            
                arguments.clear_vars();
                returns.clear_vars();
            
                *env = tenv;
            
                Ok(())
            },

            Node::Literal { typ, .. } => {
                env.stack.push(typ.clone());
                Ok(())
            },
        }
    }
}
