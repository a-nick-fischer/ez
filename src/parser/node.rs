use crate::{error::Error, lexer::token::Token};

use super::{type_env::TypeEnv, typelist::TypeList, types::typ::Type, signature_parser::TypedSignature};

#[derive(Clone, Debug)]
pub struct FunctionDefinition {
    sig: TypedSignature, 
    body: Vec<Node>
}

#[derive(Clone, Debug)]
pub enum Node {
    FunctionDefinition {
        name: String,
        assigment_token: Token,
        function_token: Token,
        definition: FunctionDefinition
    },

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
        value: Literal,
        token: Token
    }
}

#[derive(Clone, Debug)]
pub enum Literal {
    Quote(String),

    Number(f64),

    List(Vec<Node>),

    Function(FunctionDefinition)
}

impl Node {
    pub fn apply(&self, env: &mut TypeEnv) -> Result<(), Error> {
        match self {
            Node::FunctionDefinition { name, assigment_token, definition, .. } => {
                if env.bindings.contains_key(name){
                    return Err(Error::Reassigment { token: assigment_token.clone() });
                }

                env.bindings.insert(name.clone(), definition.sig.into() );
                Ok(())
            },

            Node::Assigment { name, typ, token, .. } => {
                if env.bindings.contains_key(name){
                    return Err(Error::Reassigment { token: token.clone() });
                }

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

    pub fn new_marker_call(name: &str) -> Node {
        Node::Call {
            name: name.to_string(), 
            token: Token::Newline, // TODO It does not matter, but this is hacky anyways
            arguments: TypeList::new(),
            returns: TypeList::new()
        }
    }
}
