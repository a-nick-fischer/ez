use std::{collections::HashMap, path::Path, fs};

use cranelift::{prelude::{*, settings::Flags}, codegen::Context};
use cranelift_module::DataContext;
use cranelift_object::{ObjectModule, ObjectBuilder};

use crate::{parser::types::type_env::TypeEnv, Config};


pub struct Compiler {
    builder_context: FunctionBuilderContext,

    ctx: codegen::Context,

    data_ctx: DataContext,

    module: ObjectModule,

    type_env: TypeEnv
}

impl Compiler {
    pub fn new() -> Self {
        let isa = match cranelift_native::builder() {
            Ok(builder) => {
                // See https://github.com/bytecodealliance/wasmtime/blob/e4dc9c79443259e40f3e93b9c7815b0645ebd5c4/cranelift/jit/src/backend.rs#L50
                let mut flag_builder = settings::builder();
                flag_builder.set("use_colocated_libcalls", "false").unwrap();
                flag_builder.set("is_pic", "true").unwrap();

                let flags = Flags::new(flag_builder);
                builder.finish(flags).unwrap() // TODO Errorhandling
            },

            Err(msg) => panic!("{msg}")
        };

        let builder = ObjectBuilder::new(isa, "output", cranelift_module::default_libcall_names());

        let module = ObjectModule::new(builder.unwrap());

        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: Context::new(),
            data_ctx: DataContext::new(),
            module,
            type_env: TypeEnv::new(&HashMap::new()) // TODO Change once we have a standard library
        }
    }

    pub fn compile_file<P: AsRef<Path>>(&mut self, file: P, config: &Config){
        let src = fs::read_to_string(file);
        
    }
}