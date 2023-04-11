pub mod signature_parser;
pub mod node;
pub mod types;

use crate::{lexer::token::Token, error::Error};

use self::{node::{Node, Literal, FunctionDefinition}, types::{*, type_env::TypeEnv, typelist::TypeList, typ::Type}, signature_parser::TypedSignature};

pub fn parse(mut tokens: Vec<Token>, type_env: &mut TypeEnv) -> Result<Vec<Node>, Error> {
    let mut typed_stack = Vec::new();
    
    while !tokens.is_empty() {
        let token = &tokens.pop().unwrap();

        let node = match token.clone() {
            Token::Number { value, .. } =>
                Node::Literal { 
                    typ: number_type(), 
                    token: token.clone(), 
                    value: Literal::Number(value)
                },
            
            Token::Quote { value, .. } =>
                Node::Literal { 
                    typ: quote_type(), 
                    token: token.clone(),
                    value: Literal::Quote(value)
                },
            
            Token::Ident { ref value, .. } => {
                let typ = type_env.bindings.get(value)
                    .ok_or_else(|| Error::VariableNotFound { token: token.clone() })?;

                if let Some((args, ret)) = typ.extract_function() {
                    Node::Call {
                        name: value.clone(), 
                        token: token.clone(),
                        arguments: args,
                        returns: ret
                    }
                }
                else {
                    Node::Variable { 
                        name: value.clone(), 
                        token: token.clone(), 
                        typ: typ.clone()
                    }
                }
            },
            
            Token::GetIdent { ref value, .. } => {
                let typ = type_env.bindings.get(value)
                    .ok_or_else(|| Error::VariableNotFound { token: token.clone() })?;

                Node::Variable { 
                    name: value.clone(), 
                    token: token.clone(), 
                    typ: typ.clone()
                }
            },

            Token::Assigment { ref value, .. } => {
                if let Some(val) = type_env.stack.pop() {
                    Node::Assigment { 
                        name: value.clone(), 
                        token: token.clone(),
                        typ: val
                    }
                }
                else {
                    return Err(Error::AssigmentEmptyStack { token: token.clone() })
                }
            },

            Token::List { ref value, .. } if value.is_empty() =>
                Node::Literal { 
                    typ: var_type("a", None),
                    value: Literal::List(Vec::new()),
                    token: token.clone()
                },

            Token::List { ref value, .. } => {
                let mut new_env = type_env.clone();
                new_env.stack.clear();

                let ast = parse(value.clone(), &mut new_env)?;
                
                match typecheck_list(&new_env.stack) {
                    Ok(typ) => 
                        Node::Literal { 
                            typ: list_type(typ), 
                            token: token.clone(),
                            value: Literal::List(ast)
                        },

                    Err((expected, got)) => {
                        return Err(Error::WrongTypeInList { token: token.clone(), expected, got });
                    },
                }                
            },

            Token::Function { sig: sig_src, body, .. } => {
                let sig: TypedSignature = sig_src.into();

                let mut new_env = type_env.clone();
                new_env.stack = sig.arguments().clone();

                // Typecheck args
                let ast = parse(body, &mut new_env)?;

                // Typecheck return
                typecheck_func_return(token, sig.returns().clone(), &mut new_env)?;

                let definition = FunctionDefinition { sig, body: ast };

                let next_token = tokens.first().cloned();
                match next_token {
                    Some(Token::Assigment { ref value, .. }) => {
                        tokens.pop();

                        Node::FunctionDefinition { 
                            name: value.clone(), 
                            assigment_token: next_token.unwrap(),
                            function_token: token.clone(),
                            definition
                        }
                    },

                    _ => 
                        Node::Literal { 
                            typ: sig.clone().into(),
                            value: Literal::Function(definition),
                            token: token.clone()
                        },
                }
            },

            Token::Newline => unreachable!(),
        };

        node.apply(type_env)?;
        typed_stack.push(node);
    }

    Ok(typed_stack)
}

fn typecheck_list(list: &TypeList) -> Result<Type, (Type, Type)> {
    // benjamin verifiziert
    match &list.vec()[..] {
        [] => Ok(var_type("a", None)),

        [one] => Ok(one.clone()),

        [first, ..]  => {
            for elem in list.vec() {
                if elem != first {
                    return Err((first.clone(), elem.clone()))
                }
            }

            Ok(first.clone())
        }
    }
}

fn typecheck_func_return(token: &Token, results: TypeList, new_env: &mut TypeEnv) -> Result<(), Error> {
    if new_env.stack.len() != results.len() {
        return Err(Error::IncompatibleFunctionReturn {  token: token.clone(), expected: results, got: new_env.clone().stack });
    }

    let env_clone = new_env.clone();
    let res = results.vec();

    for i in (0..res.len()).rev() {
        let stack_args = &new_env.stack.pop().unwrap().refresh_vars(new_env);

        res[i].unify(stack_args)
            .map_err(|msg| Error::UnificationError { token: token.clone(), msg })?;
    }

    // TODO Why is this necessary?
    if results.has_bound_vars() || env_clone.stack.has_bound_vars() {
        return Err(Error::IncompatibleFunctionReturn { token: token.clone(), expected: results, got: new_env.clone().stack });
    }

    Ok(())
}