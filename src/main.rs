mod lexer;
mod error;
mod types;

/*mod env;
mod repl;
mod interpreter;
mod stdlib;

#[macro_use]
extern crate lazy_static;
*/

fn main() {
    //repl::repl();
    println!("{:?}", lexer::lex("('s func[args['s]][ret['b]]) -> ('b) { a }"))
}
