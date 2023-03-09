use std::fs;

use cranelift::codegen::{ir, Context};

use yansi::{Color, Paint};

use crate::{config::DebugConfig, lexer::token::Token, parser::node::Node};

const COLOR: Color = Color::Green;

const TOKEN_FILE_NAME: &str = "ez_tokens.log";
const AST_FILE_NAME: &str = "ez_ast.log";
const CLIF_FILE_NAME: &str = "ez_clif.log";
const ASM_FILE_NAME: &str = "ez_asm.log";


pub fn debug_tokens(tokens: &[Token], config: &DebugConfig){
    if config.emit_tokens || config.emit_all {
        let msg = tokens.iter()
            .map(|t| format!("{t:?}"))
            .collect::<Vec<String>>()
            .join("\n");

        do_debug(
            msg,
            "Tokens:", 
            TOKEN_FILE_NAME, 
            config
        );
    }
}

pub fn debug_ast(nodes: &[Node], config: &DebugConfig) {
    if config.emit_ast || config.emit_all {
        let msg = nodes.iter()
            .map(|n| format!("{n:?}"))
            .collect::<Vec<String>>()
            .join("\n");

        do_debug(
            msg,
            "AST:", 
            AST_FILE_NAME, 
            config
        );
    }
}

pub fn debug_clif(func: &ir::Function, config: &DebugConfig){
    if config.emit_clif || config.emit_all {
        do_debug(
            func.to_string(),
            "CLIF:", 
            CLIF_FILE_NAME, 
            config
        );
    }
}

pub fn debug_asm(ctx: &Context, config: &DebugConfig){
    if config.emit_asm || config.emit_all {
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
    if config.emit_to_files {
        fs::write(file, content)
            .unwrap_or_else(|_| panic!("beeing able to write debug output to {file}"));
    }
    else {
        let header = Paint::new(header).bg(COLOR).bold();
        println!("{header}\n{content}\n\n")
    }
}