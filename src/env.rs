use crate::error::*;
use crate::lexer::Token;
use crate::types::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

pub type Bindings = HashMap<String, Action>;

#[derive(Debug)]
pub struct Env {
    stack: Vec<Spaned<Token>>,
    vars: Bindings
}

impl Env {
    pub fn new(vars: Bindings) -> Self {
        Env { stack: Vec::new(), vars }
    }

    pub fn push(&mut self, value: Spaned<Token>) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Token {
        self.stack.pop().unwrap().content().clone()
    }

    pub fn pops(&mut self) -> Spaned<Token> {
        self.stack.pop().unwrap()
    }

    pub fn get_var(&self, ident: &String) -> Action {
        self.vars.get(ident).unwrap().clone()
    }

    pub fn set_var(&mut self, ident: &String, value: Action){
        self.vars.insert(ident.to_owned(), value);
    }
}

pub type Action = Arc<dyn EnvAction + Send + Sync>;

pub trait EnvAction: Debug {
    fn act(&self, env: &mut Env);

    fn signature(&self, tenv: &TypeEnv) -> Result<Arc<dyn TypeEnvMod>, String>;
}

impl EnvAction for Spaned<Token> {
    fn act(&self, env: &mut Env) {
        match self.content() {
            Token::Number(_) | Token::Quote(_) => env.push(self.clone()),
            
            Token::Ident(ident) => env.get_var(&ident).act(env),

            _ => todo!()
        }
    }

    fn signature(&self, tenv: &TypeEnv) -> Result<Arc<dyn TypeEnvMod>, String> {
        match self.content() {
            Token::Number(_) => Ok(Arc::new(Signature::new("() -> (num)"))),

            Token::Quote(_) => Ok(Arc::new(Signature::new("() -> (str)"))),

            Token::List(list) if list.is_empty() => Ok(Arc::new(Signature::new("() -> (list['a])"))),

            Token::List(list) => {
                let typ = get_typ(list.first().unwrap(), tenv)?;

                for elem in list {
                    let other = get_typ(elem, tenv)?;
                    if other != typ {
                        return Err(format!("{other} not allowed in list of {typ}"));
                    }
                }

                Ok(Arc::new(Signature::new(format!("() -> (list[{typ}])").as_str()))) // Not the best approach...
            },

            Token::Function(sig, tokens) => {
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

fn get_typ(tok: &Token, tenv: &TypeEnv) -> Result<Type, String> {
    let spaned = Spaned::new(tok.clone(), 0..1);
    let sig = spaned.signature(tenv)?;

    if !sig.arguments().is_empty() {
        return Err("Functions inside lists are not allowed to take arguments".to_string());
    }

    if sig.results().len() != 1 {
        return Err("Functions inside lists must return exactly one value".to_string());
    }

    Ok(sig.results().first().unwrap().clone())
}