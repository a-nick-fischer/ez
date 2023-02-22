use std::fs;

use cranelift::codegen::{ir, Context};

use yansi::{Color, Paint};

use crate::{config::DebugConfig, lexer::token::Token, parser::node::Node};

const COLOR: Color = Color::Green;

const TOKEN_FILE_NAME: &str = "ez_tokens.log";
const AST_FILE_NAME: &str = "ez_ast.log";
const CLIF_FILE_NAME: &str = "ez_clif.log";
const ASM_FILE_NAME: &str = "ez_asm.log";


pub fn debug_tokens(tokens: &Vec<Token>, config: &DebugConfig){
    if config.emit_tokens {
        do_debug(
            format!("{tokens:?}"),
            "Tokens:", 
            TOKEN_FILE_NAME, 
            config
        );
    }
}

pub fn debug_ast(nodes: &Vec<Node>, config: &DebugConfig) {
    if config.emit_ast {
        do_debug(
            format!("{nodes:?}"),
            "AST:", 
            AST_FILE_NAME, 
            config
        );
    }
}

pub fn debug_clif(func: &ir::Function, config: &DebugConfig){
    if config.emit_clif {
        do_debug(
            func.to_string(),
            "CLIF:", 
            CLIF_FILE_NAME, 
            config
        );
    }
}

pub fn debug_asm(ctx: &Context, config: &DebugConfig){
    if config.emit_asm {
        let msg = ctx.compiled_code()
            .expect("Code to be generated when printing ASM")
            .disasm
            .clone()
            .expect("Cranelift to emit ASM");

        do_debug(
            msg,
            "ASM:", 
            ASM_FILE_NAME, 
            config
        );
    }
}

fn do_debug(content: String, header: &str, file: &str, config: &DebugConfig){
    if config.emits_to_files {
        fs::write(file, content)
            .unwrap_or_else(|_| panic!("beeing able to write debug output to {file}"));
    }
    else {
        let header = Paint::new(header).bg(COLOR).bold();
        println!("{header}\n{content}\n\n")
    }
}