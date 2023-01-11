use chumsky::Parser;

use crate::error::Error;

use self::{token::Token, why_lexer::lexer};

mod why_lexer;
pub mod sig_lexer;
pub mod token;

fn preprocess_tokens(tokens: Vec<Token>) -> Vec<Token> {
    tokens
        .split(|token| matches!(token, Token::Newline))
        .map(|vec| vec.into_iter().rev().cloned().collect::<Vec<Token>>())
        .flatten()
        .collect()
}

pub fn lex(src: &str) -> Result<Vec<Token>, Vec<Error>> {
    let (tokens, errs) = lexer().parse_recovery_verbose(src.to_string());

    tokens.map(preprocess_tokens)
        .ok_or_else(||
            errs.into_iter()
                .map(|err| Error::LexerError { inner: err })
                .collect()
        )
}
