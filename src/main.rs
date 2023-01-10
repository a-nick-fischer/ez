mod lexer;
mod error;
mod parser;
mod repl;
//mod stdlib;

#[macro_use]
extern crate lazy_static;

fn main() {
    repl::repl();
}
