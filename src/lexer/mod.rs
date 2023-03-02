use chumsky::Parser;

use crate::error::Error;

use self::{token::Token, ez_lexer::lexer};

mod ez_lexer;
pub mod sig_lexer;
pub mod token;

fn preprocess_tokens(tokens: Vec<Token>) -> Vec<Token> {
    tokens
        .split(|token| matches!(token, Token::Newline))
        .rev()
        .flat_map(|vec| vec.to_vec())
        .collect()
}

pub fn lex(src: String) -> Result<Vec<Token>, Error> {
    let (tokens, errs) = lexer().parse_recovery_verbose(src);

    tokens.map(preprocess_tokens)
        .ok_or(Error::Lexer { inner: errs })
}
