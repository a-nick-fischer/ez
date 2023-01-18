use std::path::PathBuf;

use clap::{Parser, command, Subcommand};
use compiler::Compiler;
use jit::Jit;

mod lexer;
mod error;
mod parser;
mod repl;
mod jit;
mod compiler;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
     /// (Development) If lexer-tokens should be outputed
    #[arg(long)]
    pub emit_tokens: bool,

    /// (Development) If the generated AST-nodes should be outputed.
    /// Yes, it's actually not a tree but a stack, but I'm not calling this --emit-ass 
    #[arg(long)]
    pub emit_ast: bool,

    /// (Development) If the generated Cranelift IR (CLIF) should be outputed
    #[arg(long)]
    pub emit_clif: bool,

    /// (Development) If the generated code should be dissassembled and outputed
    #[arg(long)]
    pub emit_asm: bool,

    /// (Development) No code is actually executed. Useful pared with the emit options 
    #[arg(long)]
    pub dry_run: bool,

    /// File to compile and rn
    pub file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compiles the given file
    Compile {
        /// The file to compile
        input_file: PathBuf,

        /// Optional output-file name
        #[arg(short, long)]
        output_file: Option<PathBuf>,

        /// Do not attempt to link the output to an executable, object-files (.o extension) 
        /// will be outputed instead
        #[arg(long)]
        do_not_link: bool,

        /// Compilation target triplet, e.g. x86_64-linux-musl, x86_64-windows-unknown, etc.
        /// Please note that this is for cross-compilation only - the default compilation
        /// target is optimized for the current machine, features like SIMD-support are 
        /// auto-detected. Currently there's no way to enable/disable supported features when
        /// cross-compiling.
        #[arg(short, long)]
        target: String,

        /// (Development) If the emited tokens, ast-nodes, clif etc. should be outputed
        /// to separate files instead of printed
        #[arg(long)]
        emits_to_files: bool
    }
}

fn main() {
    let config = Config::parse();

    if let Some(ref path) = config.file {
        Jit::new().run_file(path, &config);
        return
    }

    match config.command {
        Some(Commands::Compile { ref input_file, .. }) =>
            Compiler::new().compile_file(input_file, &config),

        None => 
            repl::start(),
    }
}
