use std::{collections::HashMap, path::PathBuf, fs};

use cranelift::{prelude::{*, settings::Flags}};

use cranelift_module::Module;
use cranelift_object::{ObjectModule, ObjectBuilder};

use crate::{parser::{types::type_env::TypeEnv, parse}, lexer::lex, error::{Error, error}, config::{CompilationConfig, DebugConfig}};

use super::{codegen::CodeGen, external_linker::link, success, fail, function_translator::FunctionOptions};
pub struct Compiler {
    translator: CodeGen<ObjectModule>,

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
                flag_builder.set("opt_level", "speed").unwrap();
                flag_builder.set("regalloc_checker", "true").unwrap();
                flag_builder.set("enable_alias_analysis", "true").unwrap();
                //flag_builder.set("use_egraphs", "true");
                flag_builder.set("preserve_frame_pointers", "false");

                let flags = Flags::new(flag_builder);
                builder.finish(flags).unwrap() // TODO Errorhandling
            },

            Err(msg) => panic!("{msg}")
        };

        let builder = ObjectBuilder::new(isa, "output", cranelift_module::default_libcall_names());

        let module = ObjectModule::new(builder.unwrap());

        Self {
            translator: CodeGen::new(module),

            type_env: TypeEnv::new(&HashMap::new()), // TODO Change once we have a standard library
        }
    }

    pub fn compile_file(self, config: &CompilationConfig, debug_config: &DebugConfig) {
        let (input_file, output_file) = extract_file_paths(config);

        let src = match fs::read_to_string(input_file) {
            Ok(src) => src,
            
            Err(err) => fail(error(err), "".to_string())
        };

        let compilation_result = self.do_compile(src.clone(), &output_file);

        let result = compilation_result
            .and_then(|_| link(&output_file, &config.linkage))
            .and_then(|_| fs::remove_file(&output_file) // Delete object file, not the actual output executable
                .map_err(|err| error(err)));

        match result {
            Ok(_) => success(),

            Err(err) => fail(err, src),
        }
    }

    fn do_compile(mut self, src: String, outfile: &PathBuf) -> Result<(), Error> {
        let tokens = lex(src)?;

        let ast = parse(tokens, &mut self.type_env)?;

        let isa = self.translator.module.target_config();
        self.translator.translate(ast, FunctionOptions::external(&isa))?
            .to_func("main", "(--)".parse()?); // TODO Must we accept args and return a code?

        let result = self.translator.module.finish();

        let bytes = result.emit()
            .map_err(|err| error(err))?;

        fs::write(outfile, bytes)
            .map_err(|err| error(err))
    }
}

fn extract_file_paths(config: &CompilationConfig) -> (PathBuf, PathBuf) {
    let input_file = &config.input_file;

    let mut output_file = config.output_file
        .clone()
        .unwrap_or_else(|| input_file.clone());

    output_file.set_extension(".o");

    (input_file.clone(), output_file)
}
