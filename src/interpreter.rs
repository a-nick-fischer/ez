use crate::env::{Env, Bindings};
use crate::error::report_errors;
use crate::lexer::lex;
use crate::types::*;

pub struct Interpreter<'b> {
    tenv: TypeEnv,
    env: Env<'b>
}

impl<'b> Interpreter<'b> {
    pub fn new(binding: &Bindings) -> Self {
        Interpreter { 
            tenv: TypeEnv::new(&binding),
            env: Env::new()
        }
    }

    pub fn run(&mut self, src: String){
        let result  = lex(&src)
            .and_then(|tokens| typecheck(tokens, self.tenv.clone()));

        match result {
            Ok((env, nodes)) => {
                self.tenv = env;
            },

            Err(errs) => report_errors(&src, errs)
        }        
    }
}