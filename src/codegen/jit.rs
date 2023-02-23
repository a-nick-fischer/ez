use std::{collections::HashMap, fs, mem};

use cranelift_jit::{JITModule, JITBuilder};
use cranelift_module::Module;

use crate::{parser::{types::type_env::TypeEnv, parse, node::Node, signature_parser::TypedSignature}, error::{Error, error}, lexer::lex, debug_printer::*, config::{DebugConfig, FileRunningConfig}};

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

    pub fn run_file(&mut self, config: &FileRunningConfig, debug_config: &DebugConfig){
        let input_file = config.file.clone();

        match fs::read_to_string(input_file) {
            Ok(src) => self.run_file_content(src, debug_config),

            Err(err) => fail(error(err), "".to_string()),
        }
    }

    fn run_file_content(&mut self, expr: String, debug_config: &DebugConfig){
        match self.run(expr.clone(), debug_config) {
            Ok(_) => todo!(),

            Err(errs) => fail(errs, expr),
        }
    }

    pub fn run_saving(&mut self, expr: String, debug_config: &DebugConfig) -> Result<(), Error> {
        self.type_env.bindings.insert("puts".to_string(), "(str --)".parse::<TypedSignature>().unwrap().into());

        // Parsing
        let ast = self.lex_and_parse(expr, debug_config)?;

        // Translating
        let isa = self.codegen.module.target_config();
        let options = FunctionOptions::external(&isa);

        let build_func = self.codegen.translate_ast(ast)?;

        let (id, ctx) = build_func.finish_anon_func("(jitstate --)".parse()?, options)?;
        debug_clif(&ctx.func, debug_config);
        
        // Codegenerating
        self.codegen.module.finalize_definitions()?;
        let pointer = self.codegen.module.get_finalized_function(id);
        debug_asm(&ctx, debug_config);

        // Running
        unsafe {
            let state_ptr: *const _ = &self.state;

            let fun = mem::transmute::<_, fn(*const RawJitState) -> ()>(pointer);
            fun(state_ptr);
        }
        
        Ok(())
    }

    pub fn run(&mut self, expr: String, debug_config: &DebugConfig) -> Result<(), Error> {
        // Parsing
        let ast = self.lex_and_parse(expr, debug_config)?;

        // Translating
        let isa = self.codegen.module.target_config();
        let options = FunctionOptions::external(&isa);

        let build_func = self.codegen.translate_ast(ast)?;

        let (id, ctx) = build_func.finish_anon_func("(--)".parse()?, options)?;
        debug_clif(&ctx.func, debug_config);
        
        // Codegenerating
        self.codegen.module.finalize_definitions()?;
        let pointer = self.codegen.module.get_finalized_function(id);
        debug_asm(&ctx, debug_config);

        // Running
        unsafe {
            let fun = mem::transmute::<_, fn() -> ()>(pointer);
            fun()
        }
        
        Ok(())
    }

    fn lex_and_parse(&mut self, expr: String, debug_config: &DebugConfig) -> Result<Vec<Node>, Error> {
        // Lexing
        let tokens = lex(expr)?;
        debug_tokens(&tokens, debug_config);

        // Parsing
        let ast = parse(tokens, &mut self.type_env)?;
        debug_ast(&ast, debug_config);

        Ok(ast)
    }

    pub fn defined_symbols(&self) -> impl Iterator<Item = &String> {
        self.type_env.bindings.keys()
    }

    pub fn jit_state(&self) -> JitState {
        unsafe {
            self.state.to_jit_state(&self.type_env)
        }
    }
}
