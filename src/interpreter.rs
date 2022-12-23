use crate::env::*;
use crate::error::Spaned;
use crate::error::report_errors;
use crate::lexer::*;
use crate::types::*;

pub struct Interpreter {
    tenv: TypeEnv,
    env: Env
}

impl Interpreter {
    pub fn new(binding: &Bindings) -> Self {
        Interpreter { 
            tenv: TypeEnv::new(&binding),
            env: Env::new(binding.clone())
        }
    }

    pub fn run(&mut self, src: String){
        let result  = lex(&src)
            .and_then(|tokens| typecheck(tokens, self.tenv.clone()));

        match result {
            Ok((env, nodes)) => {
                self.tenv = env;
                
                for node in nodes {
                    node.token.act(&mut self.env);
                }
            },

            Err(errs) => report_errors(&src, errs)
        }        
    }
}