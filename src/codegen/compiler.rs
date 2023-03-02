use std::{path::PathBuf, fs};

use cranelift::{prelude::{*, settings::Flags}};

use cranelift_module::Module;
use cranelift_object::{ObjectModule, ObjectBuilder};

use crate::{parser::{types::type_env::TypeEnv, parse}, lexer::lex, error::{Error, error}, config::{CompilationConfig, DebugConfig}, debug_printer::*, stdlib::{library::Transformations, create_stdlib}};

use super::{codegen_module::CodeGenModule, external_linker::link, success, fail, function_translator::FunctionOptions};
pub struct Compiler {
    translator: CodeGenModule<ObjectModule>,

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
                //flag_builder.set("use_egraphs", "true").unwrap();
                flag_builder.set("preserve_frame_pointers", "false").unwrap();

                let flags = Flags::new(flag_builder);
                builder.finish(flags).unwrap() // TODO Errorhandling
            },

            Err(msg) => panic!("{msg}")
        };

        let builder = ObjectBuilder::new(isa, "output", cranelift_module::default_libcall_names());

        let module = ObjectModule::new(builder.unwrap());

        let library = create_stdlib();
        let mut codegen = CodeGenModule::new(module, library.transformations.clone());
        library.init_with(&mut codegen).expect("Could not init standard library");

        Self {
            type_env: library.type_env(),

            translator: codegen
        }
    }

    pub fn compile_file(self, config: &CompilationConfig, debug_config: &DebugConfig) {
        let (input_file, output_file) = extract_file_paths(config);

        let src = match fs::read_to_string(input_file) {
            Ok(src) => src,
            
            Err(err) => fail(error(err), "".to_string())
        };

        let compilation_result = self.do_compile(src.clone(), &output_file, debug_config);

        let result = compilation_result
            .and_then(|_| link(&output_file, &config.linkage))
            .and_then(|_| fs::remove_file(&output_file) // Delete object file, not the actual output executable
                .map_err(error));

        match result {
            Ok(_) => success(),

            Err(err) => fail(err, src),
        }
    }

    fn do_compile(mut self, src: String, outfile: &PathBuf, debug_config: &DebugConfig) -> Result<(), Error> {
        let tokens = lex(src)?;
        debug_tokens(&tokens, debug_config);

        let ast = parse(tokens, &mut self.type_env)?;
        debug_ast(&ast, debug_config);

        let isa = self.translator.module.target_config();
        let options = FunctionOptions::external(&isa);

        let (_, ctx) = self.translator
            .translate_ast("(--)".parse()?, ast)? // TODO Must we accept args and return a code?
            .finish_func("main", options)?;

        debug_clif(&ctx.func, debug_config);
        debug_asm(&ctx, debug_config);

        let result = self.translator.module.finish();

        let bytes = result.emit()
            .map_err(error)?;

        fs::write(outfile, bytes)
            .map_err(error)
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
