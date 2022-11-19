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

    pub fn push(&'a mut self, action: Action<'a>) {
        action.act(self);
    }

    pub fn push_val(&mut self, value: Spaned<Token>) {
        self.stack.push(value);
    }

    pub fn pop_val(&mut self) -> Spaned<Token> {
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
            Token::Number(_) | Token::String(_) => env.push_val(self.clone()),
            
            Token::Ident(ident) => env.get_var(&ident).act(env),

            _ => unreachable!()
        }
    }

    fn signature(&self, tenv: &TypeEnv) -> Result<Signature, String> {
        match self.content() {
            Token::Number(_) => Ok(Signature::new("() -> (num)")),

            Token::String(_) => Ok(Signature::new("() -> (str)")),

            Token::List(_) => todo!(),

            Token::Function(_, _) => todo!(),
            
            Token::Ident(ident) => 
                tenv.bindings.get(ident).ok_or(format!("{ident} not found")).cloned(),

            _ => unreachable!()
        }
    }
}