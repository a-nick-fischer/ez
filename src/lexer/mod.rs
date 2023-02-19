use chumsky::Parser;

use crate::error::Error;

use self::{token::Token, ez_lexer::lexer};

mod ez_lexer;
pub mod sig_lexer;
pub mod token;

fn preprocess_tokens(tokens: Vec<Token>) -> Vec<Token> {
    tokens
        .split(|token| matches!(token, Token::Newline))
        .map(|vec| vec.into_iter().rev().cloned().collect::<Vec<Token>>())
        .flatten()
        .collect()
}

pub fn lex(src: String) -> Result<Vec<Token>, Error> {
    let (tokens, errs) = lexer().parse_recovery_verbose(src);

    tokens.map(preprocess_tokens)
        .ok_or_else(|| Error::Lexer { inner: errs })
}
