use std::rc::Rc;

use crate::{lexer::token::Token, typechecker::{type_env::TypeEnvModifier, signature::Signature, types::Type}, error::TErr};


impl TryInto<TypeEnvModifier> for Token {
    type Error = TErr;

    fn try_into(self) -> Result<TypeEnvModifier, Self::Error> {
        match self {
            Token::Number { .. } => 
                Ok(Rc::new("() -> (num)".parse::<Signature>())),

            Token::Quote { .. } => 
                Ok(Rc::new("() -> (str)".parse())),

            Token::List { value, .. } if value.is_empty() => 
                Ok(Rc::new("() -> (list['a])".parse())),

            Token::List { value, .. } => {
                let typ = get_typ(list.first().unwrap())?;

                for elem in list {
                    let other = get_typ(elem, tenv)?;
                    if other != typ {
                        return Err(format!("{other} not allowed in list of {typ}"));
                    }
                }

                Ok(Rc::new(format!("() -> (list[{typ}])").into())) // Not the best approach...
            },

            Token::Function { sig, body, .. } => {
                // TODO: Move this
                let fun = Signature::from_sig(sig.clone());

                let mut fun_env = tenv.clone();
                fun_env.stack = fun.arguments();

                let results = fun.results();

                // We need more complex error handing...
                let (mut new_env, _) =  typecheck(tokens.clone(), fun_env)
                    .map_err(|err| err_to_str(err))?;

                let type_error = |a: &Vec<Type>, b: &Vec<Type>| format!("Expected func to return {}, got {}", 
                    tlist_to_str(a), 
                    tlist_to_str(b));

                if new_env.stack.len() != results.len() {
                    return Err(type_error(&results, &new_env.stack));
                }

                let env_clone = new_env.clone();
                
                // Return type check.. should be reworked prob
                for i in (0..results.len()).rev() {
                    let stack_args = &new_env.stack.pop().unwrap().refresh_vars(&mut new_env);
        
                    results[i].unify(stack_args)
                        .map_err(|_| type_error(&results, &env_clone.stack))?;
                }

                // Check if any variables were bound
                let has_binds = |list: &Vec<Type>| list.into_iter().any(|t| t.has_bound_vars());

                if has_binds(&results) || has_binds(&new_env.stack) {
                    return Err(type_error(&results, &env_clone.stack));
                }

                Ok(
                    Arc::new(Signature::from_types(vec![], vec![fun.to_type()]))
                )
            },

            Token::Ident(ident) => 
                tenv.bindings.get(ident).ok_or(format!("{ident} not found")).cloned(),

            Token::Assigment(ident) => {
                Ok(Arc::new(Assigment::new(ident.clone())))
            }

            _ => unreachable!()
        }
    }
}

fn get_typ(tok: &Token) -> Result<Type, String> {
    let sig = Signature::try_from(tok);

    // TODO Do not require this bs
    if !sig.arguments().is_empty() {
        return Err("Functions inside lists are not allowed to take arguments".to_string());
    }

    if sig.results().len() != 1 {
        return Err("Functions inside lists must return exactly one value".to_string());
    }

    Ok(sig.results().first().unwrap().clone())
}