use crate::error::Spaned;
use crate::lexer::Token;
use crate::types::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

pub type Bindings<'a> = HashMap<String, Action<'a>>;

#[derive(Debug)]
pub struct Env<'a> {
    stack: Vec<Spaned<Token>>,
    vars: Bindings<'a>
}

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Env { stack: Vec::new(), vars: HashMap::new() }
    }

    pub fn push_action(&'a mut self, action: Action<'a>) {
        action.act(self);
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

    pub fn get_var(&self, ident: &String) -> Action<'a> {
        self.vars.get(ident).unwrap().clone()
    }
}

pub type Action<'a> = Arc<dyn EnvAction<'a> + Send + Sync>;

pub trait EnvAction<'a>: Debug {
    fn act(&self, env: &'a mut Env<'a>);

    fn signature(&self, tenv: &TypeEnv) -> Result<Signature, String>;
}


impl<'a> EnvAction<'a> for Spaned<Token> {
    fn act(&self, env: &'a mut Env<'a>) {
        match self.content() {
            Token::Number(_) | Token::Quote(_) => env.push(self.clone()),
            
            Token::Ident(ident) => env.get_var(&ident).act(env),

            _ => unreachable!()
        }
    }

    fn signature(&self, tenv: &TypeEnv) -> Result<Signature, String> {
        match self.content() {
            Token::Number(_) => Ok(Signature::new("() -> (num)")),

            Token::Quote(_) => Ok(Signature::new("() -> (str)")),

            Token::List(list) if list.is_empty() => Ok(Signature::new("() -> (list['a])")),

            Token::List(list) => {
                let typ = get_typ(list.first().unwrap(), tenv)?;

                for elem in list {
                    let other = get_typ(elem, tenv)?;
                    if other != typ {
                        return Err(format!("{other} not allowed in list of {typ}"));
                    }
                }

                Ok(Signature::new(format!("() -> (list[{typ}])").as_str())) // Not the best approach...
            },

            Token::Function(fun, tokens) => {
                todo!()
            },
            
            Token::Ident(ident) => 
                tenv.bindings.get(ident).ok_or(format!("{ident} not found")).cloned(),

            _ => unreachable!()
        }
    }
}

fn get_typ(tok: &Token, tenv: &TypeEnv) -> Result<Type, String> {
    let spaned = Spaned::new(tok.clone(), 0..1);
    let sig = spaned.signature(tenv)?;

    if !sig.arguments.is_empty() {
        return Err("Functions inside lists are not allowed to take arguments".to_string());
    }

    if sig.results.len() != 1 {
        return Err("Functions inside lists must return exactly one value".to_string());
    }

    Ok(sig.results.first().unwrap().clone())
}