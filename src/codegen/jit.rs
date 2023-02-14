use std::{collections::HashMap, fs, mem};

use cranelift_jit::{JITModule, JITBuilder};

use crate::{Config, parser::{types::type_env::TypeEnv, parse}, error::{Error, error}, lexer::lex, debug_printer::{debug_tokens, debug_ast}};

use super::{codegen::CodeGen, fail, function_translator::FunctionOptions};

pub struct Jit {
    codegen: CodeGen<JITModule>,

    type_env: TypeEnv
}

impl Jit {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder.unwrap());

        Self {
            codegen: CodeGen::new(module),

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
        // Lexing
        let tokens = lex(expr)?;
        debug_tokens(&tokens, &config.debug_config);

        // Parsing
        let ast = parse(tokens, &mut self.type_env)?;
        debug_ast(&ast, &config.debug_config);

        // Compiling
        let isa = self.codegen.target_config();
        let func = self.codegen.translate(None, ast, FunctionOptions::external(&isa))?;
        self.codegen.module.finalize_definitions()?;

        // Running
        let pointer = self.codegen.module.get_finalized_function(func);

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
