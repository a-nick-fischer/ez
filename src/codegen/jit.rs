use std::{collections::HashMap, path::PathBuf, fs};

use ariadne::{Color, Fmt};
use cranelift::prelude::*;
use cranelift_jit::{JITModule, JITBuilder};
use cranelift_module::{DataContext, Module};

use crate::{Config, parser::{types::type_env::TypeEnv, parse}, error::{report_errors, Error, error}, lexer::lex};

use super::translator::Translator;

pub struct Jit {
    translator: Translator<JITModule>,

    type_env: TypeEnv
}

impl Jit {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder.unwrap());

        Self {
            translator: Translator {
                builder_context: FunctionBuilderContext::new(),
                ctx: module.make_context(),
                data_ctx: DataContext::new(),
                module,
                naming_idx: 0
            },

            type_env: TypeEnv::new(&HashMap::new()), // TODO Change once we have a standard library
        }
    }

    pub fn run_file(&mut self, config: &Config){
        let input_file = config.file.clone().expect("not triggering a compilter bug");

        match fs::read_to_string(input_file) {
            Ok(src) => self.run_expr(src, config),

            Err(err) => fail(vec![error(err)], "".to_string()),
        }
    }

    pub fn run_expr(&mut self, expr: String, config: &Config){
        
    }

    pub fn do_run(&mut self, expr: String, config: &Config) -> Result<(), Vec<Error>> {
        let tokens = lex(expr)?;

        let ast = parse(tokens, &mut self.type_env)?;

        let func = self.translator.translate(None, ast)?;
    }
}

fn fail(errs: Vec<Error>, src: String){
    let len = errs.len();
    report_errors(src, errs);

    let msg = format!("Build failed with {} errors", len);
    println!("\n\t{}", msg.fg(Color::Red));
}
