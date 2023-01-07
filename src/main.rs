mod lexer;
mod error;
mod typechecker;
mod env_modifier;
mod parser;
mod repl;
mod interpreter;
mod stdlib;

#[macro_use]
extern crate lazy_static;

fn main() {
    repl::repl();
}
