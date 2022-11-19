use types::Signature;

mod lexer;
mod error;
mod types;

mod env;
mod repl;
mod interpreter;
mod stdlib;

#[macro_use]
extern crate lazy_static;


fn main() {
    repl::repl();
    //println!("{:?}", lexer::lex_sig("() -> (num)"))
}
