use std::{collections::HashMap, fs, mem};

use cranelift_jit::{JITModule, JITBuilder};
use cranelift_module::Module;

use crate::{Config, parser::{types::type_env::TypeEnv, parse, node::Node}, error::{Error, error}, lexer::lex, debug_printer::{debug_tokens, debug_ast}};

use super::{codegen_module::CodeGenModule, fail, function_translator::FunctionOptions, jit_ffi::{RawJitState, JitState}};

pub struct Jit<'a> {
    codegen: CodeGenModule<JITModule>,

    type_env: TypeEnv,

    state: RawJitState<'a>
}

impl<'a> Jit<'a> {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder.unwrap());

        Self {
            codegen: CodeGenModule::new(module),

            type_env: TypeEnv::new(&HashMap::new()), // TODO Change once we have a standard library

            state: RawJitState::new()
        }
    }

    pub fn run_file(&mut self, config: &Config){
        let input_file = config.file.clone()
            .expect("not triggering a compiler bug");

        match fs::read_to_string(input_file) {
            Ok(src) => self.run_file_content(src, config),

            Err(err) => fail(error(err), "".to_string()),
        }
    }

    fn run_file_content(&mut self, expr: String, config: &Config){
        match self.run(expr.clone(), config) {
            Ok(_) => todo!(),

            Err(errs) => fail(errs, expr),
        }
    }

    pub fn run_saving(&mut self, expr: String, config: &Config) -> Result<(), Error> {
        // Parsing
        let ast = self.lex_and_parse(expr, config)?;

        // Translating
        let isa = self.codegen.module.target_config();
        let options = FunctionOptions::external(&isa);

        let id = self.codegen
            .translate_ast(ast)?
            .finish_anon_func("(jitstate --)".parse()?, options)?;
        
        // Codegenerating
        self.codegen.module.finalize_definitions()?;
        let pointer = self.codegen.module.get_finalized_function(id);

        // Running
        unsafe {
            let state_ptr: *const _ = &self.state;

            let fun = mem::transmute::<_, fn(*const RawJitState) -> ()>(pointer);
            fun(state_ptr)
        }
        
        Ok(())
    }

    pub fn run(&mut self, expr: String, config: &Config) -> Result<(), Error> {
        // Parsing
        let ast = self.lex_and_parse(expr, config)?;

        // Translating
        let isa = self.codegen.module.target_config();
        let options = FunctionOptions::external(&isa);

        let id = self.codegen
            .translate_ast(ast)?
            .finish_anon_func("(--)".parse()?, options)?;
        
        // Codegenerating
        self.codegen.module.finalize_definitions()?;
        let pointer = self.codegen.module.get_finalized_function(id);

        // Running
        unsafe {
            let fun = mem::transmute::<_, fn() -> ()>(pointer);
            fun()
        }
        
        Ok(())
    }

    fn lex_and_parse(&mut self, expr: String, config: &Config) -> Result<Vec<Node>, Error> {
        // Lexing
        let tokens = lex(expr)?;
        debug_tokens(&tokens, &config.debug_config);

        // Parsing
        let ast = parse(tokens, &mut self.type_env)?;
        debug_ast(&ast, &config.debug_config);

        Ok(ast)
    }

    pub fn defined_symbols(&self) -> impl Iterator<Item = &String> {
        self.type_env.bindings.keys()
    }

    pub fn jit_state(&self) -> JitState {
        todo!()
    }
}
