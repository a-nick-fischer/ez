use crate::env::{Env, Bindings};
use crate::error::report_errors;
use crate::lexer::lex;
use crate::types::*;

pub struct Interpreter<'a, 'b> {
    tenv: TypeEnv<'a>,
    env: Env<'b>
}

impl<'a, 'b> Interpreter<'a, 'b> {
    pub fn new(binding: &'a Bindings<'a>) -> Self {
        Interpreter { 
            tenv: TypeEnv::new(&binding),
            env: Env::new()
        }
    }

    pub fn run(&mut self, src: String){
        let result  = lex(&src)
            .and_then(|tokens| typecheck(tokens, &self.tenv));

        match result {
            Ok(types) => {
                println!("{types:?}");
                self.tenv = types;
            },

            Err(errs) => report_errors(&src, errs)
        }        
    }
}