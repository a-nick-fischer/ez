mod signature_parser;
mod node;
mod types;

use crate::{lexer::token::Token, error::TErr};

use self::{node::Node, types::{*, type_env::TypeEnv, typelist::TypeList, types::Type}, signature_parser::parse_signature};

pub fn parse(tokens: Vec<Token>, type_env: &mut TypeEnv) -> Result<Vec<Node>, TErr> {
    let typed_stack = Vec::new();

    let apply = |node: Node| {
        node.apply(type_env);
        typed_stack.push(node);
    };
    
    while !tokens.is_empty() {
        let token = tokens.pop().unwrap();

        match token {
            Token::Number { .. } => apply(
                Node::Literal { typ: number_type(), token }
            ),
            
            Token::Quote { .. } => apply(
                Node::Literal { typ: quote_type(), token }
            ),
            
            Token::Ident { value, .. } => {
                let typ = type_env.bindings.get(&value).unwrap(); // TODO error handling

                let node = if let Some((args, ret)) = typ.extract_function() {
                    Node::Call { 
                        name: value, 
                        token,
                        arguments: args,
                        returns: ret 
                    }
                }
                else {
                    Node::Variable { name: value, token, typ: typ.clone() }
                };

                apply(node);
            },
            
            Token::GetIdent { value, .. } => {
                let typ = type_env.bindings.get(&value).unwrap(); // TODO error handling

                let node = Node::Variable { name: value, token, typ: typ.clone() };

                apply(node);
            },

            Token::Assigment { value, .. } => {
                if let Some(val) = type_env.stack.pop() {
                    let node = Node::Assigment { name: value, token: token, typ: val };

                    apply(node);
                }
                else {
                    panic!() // TODO Error handling
                }
            },

            Token::List { value, .. } if value.is_empty() => apply(
                Node::Literal { typ: var_type("a", None), token }
            ),

            Token::List { value, .. } => {
                let new_env = type_env.clone();
                new_env.stack.clear();

                parse(value, &mut new_env)?;
                
                match typecheck_list(&new_env.stack) {
                    Ok(typ) => apply(
                        Node::Literal { typ: list_type(typ), token }
                    ),

                    Err(wrong_typ) => {
                        panic!() // TODO Errorhandling
                    },
                }                
            },

            Token::Function { sig, body, .. } => {
                let (args, ret) = parse_signature(sig);

                let new_env = type_env.clone();
                new_env.stack = args;

                // Typecheck args
                parse(body, &mut new_env)?;

                // Typecheck return
                typecheck_func_return(ret, &mut new_env)                
            },

            Token::Newline => unreachable!(),
        }
    }

    Ok(typed_stack)
}

fn typecheck_list(list: &TypeList) -> Result<Type, Type> {
    if list.is_empty() { 
        return Ok(var_type("a", None)); 
    }

    let first = list.vec().get(0).unwrap().clone();

    if list.len() == 1 {
        return Ok(first)
    }

    for elem in list.vec() {
        if elem != &first {
            return Err(elem.clone())
        }
    }

    Ok(first)
}

fn typecheck_func_return(results: TypeList, new_env: &mut TypeEnv) {
    if new_env.stack.len() != results.len() {
        panic!() // TODO Error handling
    }

    let env_clone = new_env.clone();
    let res = results.vec();

    for i in (0..res.len()).rev() {
        let stack_args = &new_env.stack.pop().unwrap().refresh_vars(&mut new_env);

        res[i].unify(stack_args).unwrap(); // TODO Error handling
    }

    // TODO Why is this necessary?
    if results.has_bound_vars() || env_clone.stack.has_bound_vars() {
        panic!(); // TODO Error handling
    }
}