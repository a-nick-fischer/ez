use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Spaned;
use crate::lexer::Token;
use crate::env::*;
use crate::signature_lexer::lex_signature;
use crate::types::*;

lazy_static! {
    pub static ref STDLIB: Bindings<'static> = stdlib();
}

#[derive(Debug)]
struct Add {}

impl<'a> EnvAction<'a> for Add {
    fn act(&self, env: &'a mut Env<'a>) {
        match (env.pop_val().content(), env.pop_val().content()) {
            (Token::Number(a), Token::Number(b)) => env.push_val(Spaned::new(Token::Number(a + b))),

            _ => panic!()
        }
    }

    fn signature(&self, env: &TypeEnv) -> Signature {
        /*match env.stack[..] {
            [Type::Literal(Literal::Number(fa)), Type::Literal(Literal::Number(fb)), ..] => todo!()

            _ => 
        }*/

        //lex_signature("'S a b -> 'S c".to_string()).unwrap()

        Signature(vec![var("S"), number(), number()], vec![var("S"), number()])
    }
}

pub fn stdlib<'a>() -> Bindings<'a> {
    let mut map: HashMap<String, Action<'a>> = HashMap::new();
    map.insert("add".to_owned(), Arc::new(Add {}));
    map
}