use clap::Parser;
use codegen::compiler::Compiler;
use codegen::jit::Jit;
use config::{Config, Commands};
use repl::Repl;

mod lexer;
mod error;
mod parser;
mod repl;
mod codegen;
mod config;
mod debug_printer;

fn main() {
    let config = Config::parse();

    if let Some(ref path) = config.file {
        Jit::new().run_file(&config);
        return
    }

    match config.command {
        Some(Commands::Compile { comp_config }) =>
            Compiler::new().compile_file(&comp_config),

        None => 
            Repl::new().start(),
    }
}
