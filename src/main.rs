use clap::Parser;
use codegen::compiler::Compiler;
use codegen::jit::Jit;
use config::{Config, Commands};
use cranelift_object::ObjectModule;
use repl::Repl;

mod lexer;
mod error;
mod parser;
mod repl;
mod codegen;
mod config;
mod debug_printer;
mod stdlib;

#[macro_use]
extern crate lazy_static;

fn main() {
    crate::stdlib::test::<ObjectModule>();
    return;

    let config = Config::parse();

    match config.command {
        Some(Commands::Run { run_config }) =>
            Jit::new().run_file(&run_config, &config.debug_config),
        
        Some(Commands::Compile { comp_config }) =>
            Compiler::new().compile_file(&comp_config, &config.debug_config),

        None => 
            Repl::new(config).start(),
    }
}
