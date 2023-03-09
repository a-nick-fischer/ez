use std::path::PathBuf;

use clap::{Parser, command, Subcommand, Args};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[clap(flatten)]
    pub debug_config: DebugConfig,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Parser, Debug)]
pub struct FileRunningConfig {
    /// File to compile and run
    pub file: PathBuf
}

#[derive(Debug, Args)]
pub struct DebugConfig {
    /// (Development) No code is actually executed. Useful pared with the emit options 
    #[arg(long)]
    pub dry_run: bool,

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

    /// (Development) Sets all emit options to true
    #[arg(long)]
    pub emit_all: bool,

    /// (Development) If the emited tokens, ast-nodes, clif etc. should be outputed
    /// to separate files instead of printed. For compiling or running files only.
    #[arg(long)]
    pub emit_to_files: bool
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compiles the given file
    Compile {
        #[clap(flatten)]
        comp_config: CompilationConfig
    },

    /// Runs the given file
    Run {
        #[clap(flatten)]
        run_config: FileRunningConfig
    }
}

#[derive(Debug, Args)]
pub struct CompilationConfig {
    /// The file to compile
    pub input_file: PathBuf,

    /// Optional output-file name
    #[arg(short, long)]
    pub output_file: Option<PathBuf>,

    #[clap(flatten)]
    pub linkage: LinkageConfig,

    /// Compilation target triplet, e.g. x86_64-linux-musl, x86_64-windows-unknown, etc.
    /// Please note that this is for cross-compilation only - the default compilation
    /// target is optimized for the current machine, features like SIMD-support are 
    /// auto-detected. Currently there's no way to enable/disable supported features when
    /// cross-compiling.
    #[arg(short, long)]
    pub target: Option<String>,
}

#[derive(Debug, Args)]
pub struct LinkageConfig {
    /// Do not attempt to link the output to an executable, object-files (.o extension) 
    /// will be outputed instead
    #[arg(long)]
    pub do_not_link: bool,

    // Custom linker command
    #[arg(long)]
    pub linker_command: Option<Vec<String>>,
}