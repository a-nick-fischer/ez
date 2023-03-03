use std::{fs, mem};

use cranelift_jit::{JITModule, JITBuilder};
use cranelift_module::Module;

use crate::{parser::{types::{type_env::TypeEnv, typelist::TypeList}, parse, node::Node}, error::{Error, error}, lexer::{lex, token::Token}, debug_printer::*, config::{DebugConfig, FileRunningConfig}, stdlib::create_stdlib};

use super::{codegen_module::CodeGenModule, fail, function_translator::FunctionOptions, jit_ffi::{RawJitState, JitState}};

pub struct Jit {
    codegen: CodeGenModule<JITModule>,

    type_env: TypeEnv,

    state: RawJitState
}

impl Jit {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder.unwrap());

        let library = create_stdlib();
        let type_env = library.type_env();
        let mut codegen = CodeGenModule::new(module);
        library.init_codegen(&mut codegen).expect("Could not init standard library");

        Self {
            type_env,

            codegen,

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
        // Parsing
        let mut ast = self.lex_and_parse(expr, debug_config)?;

        // Insert save call for saving stack state
        ast.push(Self::save_state_call());

        // Translating
        let isa = self.codegen.module.target_config();
        let options = FunctionOptions::external(&isa);

        let (id, ctx) = self.codegen
            .translate_ast("(jitstate --)".parse()?, ast)?
            .finish_anon_func(options)?;

        debug_clif(&ctx.func, debug_config);
        
        // Codegenerating
        self.codegen.module.finalize_definitions()?;
        let pointer = self.codegen.module.get_finalized_function(id);
        debug_asm(&ctx, debug_config);

        // Running
        unsafe {
            let state_ptr: *const _ = &mut self.state;

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

        let (id, ctx) = self.codegen
            .translate_ast("(--)".parse()?, ast)?
            .finish_anon_func(options)?;
        
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

    fn save_state_call() -> Node {
        Node::Call { 
            name: "__save".to_string(), 
            token: Token::Newline, // TODO It does not matter, but this is hacky anyways
            arguments: TypeList::new(),
            returns: TypeList::new(), 
            stack_size: 0
        }
    }
}
