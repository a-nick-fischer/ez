use std::{path::Path, collections::HashMap};

use cranelift::prelude::*;
use cranelift_jit::{JITModule, JITBuilder};
use cranelift_module::{DataContext, Module};

use crate::{Config, parser::types::type_env::TypeEnv};

pub struct Jit {
    builder_context: FunctionBuilderContext,

    ctx: codegen::Context,

    data_ctx: DataContext,

    module: JITModule,

    type_env: TypeEnv
}

impl Jit {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder.unwrap());
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_ctx: DataContext::new(),
            module,
            type_env: TypeEnv::new(&HashMap::new()) // TODO Change once we have a standard library
        }
    }

    pub fn run_file<P: AsRef<Path>>(&mut self, file: P, config: &Config){

    }

    pub fn run_expr(&mut self, expr: String, config: &Config){
        
    }
}