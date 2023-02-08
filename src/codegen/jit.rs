use std::{collections::HashMap, fs, mem};

use cranelift::prelude::*;
use cranelift_jit::{JITModule, JITBuilder};
use cranelift_module::{DataContext, Module};

use crate::{Config, parser::{types::type_env::TypeEnv, parse}, error::{Error, error}, lexer::lex};

use super::{codegen::CodeGen, fail};

pub struct Jit {
    translator: CodeGen<JITModule>,

    type_env: TypeEnv
}

impl Jit {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder.unwrap());

        Self {
            translator: CodeGen {
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
        let input_file = config.file.clone().expect("not triggering a compiler bug");

        match fs::read_to_string(input_file) {
            Ok(src) => self.run_expr(src, config),

            Err(err) => fail(error(err), "".to_string()),
        }
    }

    pub fn run_expr(&mut self, expr: String, config: &Config){
        match self.do_run(expr.clone(), config) {
            Ok(_) => todo!(),

            Err(errs) => fail(errs, expr),
        }
    }

    pub fn do_run(&mut self, expr: String, config: &Config) -> Result<(), Error> {
        let tokens = lex(expr)?;

        let ast = parse(tokens, &mut self.type_env)?;

        let func = self.translator.translate(None, ast)?;

        self.translator.module.finalize_definitions().unwrap(); // TODO Error handling

        let pointer = self.translator.module.get_finalized_function(func);

        unsafe {
            let fun = mem::transmute::<_, fn() -> ()>(pointer);
            fun();
        }

        Ok(())
    }

    pub fn defined_symbols(&self) -> impl Iterator<Item = &String> {
        self.type_env.bindings.keys()
    }
}
