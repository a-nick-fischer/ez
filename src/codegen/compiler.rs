use std::{collections::HashMap, path::Path, fs};

use ariadne::{Fmt, Color};
use cranelift::{prelude::{*, settings::Flags}, codegen::Context};
use cranelift_module::DataContext;
use cranelift_object::{ObjectModule, ObjectBuilder};

use crate::{parser::{types::type_env::TypeEnv, parse}, Config, lexer::lex, error::{Error, report_errors}};

use super::Translator;

pub struct Compiler {
    translator: Translator<ObjectModule>,

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
            translator: Translator { 
                builder_context: FunctionBuilderContext::new(),
                ctx: Context::new(),
                data_ctx: DataContext::new(),
                module,
                naming_idx: 0,
            },

            type_env: TypeEnv::new(&HashMap::new()), // TODO Change once we have a standard library
        }
    }

    pub fn compile_file<P: AsRef<Path>>(&mut self, file: P, config: &Config) {
        let result = fs::read_to_string(file)
            .map_err(|err| 
                (vec![Error::GeneralError { message: err.to_string() }], "".to_owned()))
            .and_then(|src| 
                self.compile(src, &config)
                    .map_err(|errs| (errs, src)));

        match result {
            Ok(_) => {
                println!("\n\t{}", "Build succeeded".fg(Color::Green));
            },

            Err((errs, src)) => {
                let len = errs.len();
                report_errors(src, errs);

                let msg = format!("Build failed with {} errors", len);
                println!("\n\t{}", msg.fg(Color::Red));
            },
        }
    }

    fn compile(&mut self, src: String, config: &Config) -> Result<(), Vec<Error>> {
        let tokens = lex(src)?;

        let ast = parse(tokens, &mut self.type_env)?;

        self.translator.translate(ast)?;

        let result = self.translator.module.finish(); // TODO How to entrypoint?

        todo!()
    }
} 