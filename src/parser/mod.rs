mod signature_parser;
pub mod node;
pub mod types;

use crate::{lexer::token::Token, error::Error};

use self::{node::{Node, Literal}, types::{*, type_env::TypeEnv, typelist::TypeList, types::Type}, signature_parser::parse_signature};

pub fn parse(tokens: Vec<Token>, type_env: &mut TypeEnv) -> Result<Vec<Node>, Vec<Error>> {
    let mut typed_stack = Vec::new();
    let mut tokens = tokens.clone();

    let apply = |node: Node, type_env: &mut TypeEnv, typed_stack: &mut Vec<Node>| {
        node.apply(type_env);
        typed_stack.push(node);
    };
    
    while !tokens.is_empty() {
        let ref token = tokens.pop().unwrap();

        match token.clone() {
            Token::Number { value, .. } => apply(
                Node::Literal { 
                    typ: number_type(), 
                    token: token.clone(), 
                    value: Literal::Number(value),
                    stack_size:  type_env.stack.len()
                },
                type_env,
                &mut typed_stack
            ),
            
            Token::Quote { value, .. } => apply(
                Node::Literal { 
                    typ: quote_type(), 
                    token: token.clone(),
                    value: Literal::Quote(value),
                    stack_size:  type_env.stack.len()
                },
                type_env,
                &mut typed_stack
            ),
            
            Token::Ident { ref value, .. } => {
                let typ = type_env.bindings.get(value)
                    .ok_or_else(|| vec![Error::VariableNotFound { token: token.clone() }])?;

                let node = if let Some((args, ret)) = typ.extract_function() {
                    Node::Call { 
                        name: value.clone(), 
                        token: token.clone(),
                        arguments: args,
                        returns: ret,
                        stack_size:  type_env.stack.len()
                    }
                }
                else {
                    Node::Variable { 
                        name: value.clone(), 
                        token: token.clone(), 
                        typ: typ.clone(),
                        stack_size:  type_env.stack.len()
                    }
                };

                apply(node, type_env, &mut typed_stack);
            },
            
            Token::GetIdent { ref value, .. } => {
                let typ = type_env.bindings.get(value)
                    .ok_or_else(|| vec![Error::VariableNotFound { token: token.clone() }])?;

                let node = Node::Variable { 
                    name: value.clone(), 
                    token: token.clone(), 
                    typ: typ.clone(),
                    stack_size:  type_env.stack.len()
                };

                apply(node, type_env, &mut typed_stack);
            },

            Token::Assigment { ref value, .. } => {
                if let Some(val) = type_env.stack.pop() {
                    let node = Node::Assigment { 
                        name: value.clone(), 
                        token: token.clone(), 
                        typ: val,
                        stack_size:  type_env.stack.len()
                    };

                    apply(node, type_env, &mut typed_stack);
                }
                else {
                    return Err(vec![Error::AssigmentEmptyStack { token: token.clone() }])
                }
            },

            Token::List { ref value, .. } if value.is_empty() => apply(
                Node::Literal { 
                    typ: var_type("a", None),
                    value: Literal::List(Vec::new()),
                    token: token.clone(),
                    stack_size:  type_env.stack.len()
                },
                type_env,
                &mut typed_stack
            ),

            Token::List { ref value, .. } => {
                let mut new_env = type_env.clone();
                new_env.stack.clear();

                let ast = parse(value.clone(), &mut new_env)?;
                
                match typecheck_list(&new_env.stack) {
                    Ok(typ) => apply(
                        Node::Literal { 
                            typ: list_type(typ), 
                            token: token.clone(),
                            value: Literal::List(ast),
                            stack_size: type_env.stack.len()
                        },
                        type_env,
                        &mut typed_stack
                    ),

                    Err((expected, got)) => {
                        return Err(vec![Error::WrongTypeInList { token: token.clone(), expected, got }]);
                    },
                }                
            },

            Token::Function { sig, body, .. } => {
                let (args, ret) = parse_signature(sig);

                let mut new_env = type_env.clone();
                new_env.stack = args.clone();

                // Typecheck args
                let ast = parse(body, &mut new_env)?;

                // Typecheck return
                typecheck_func_return(token, ret.clone(), &mut new_env)
                    .map_err(|err| vec![err])?;

                let node = Node::Literal { 
                    typ: func_type(
                        args.vec().clone(), 
                        ret.vec().clone()
                    ),
                    value: Literal::Function(ast),
                    token: token.clone(), 
                    stack_size: type_env.stack.len()
                };
                
                apply(node, type_env, &mut typed_stack);
            },

            Token::Newline => unreachable!(),
        }
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