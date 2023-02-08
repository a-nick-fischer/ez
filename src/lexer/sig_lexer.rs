use chumsky::prelude::*;

use crate::error::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct Signature(Vec<SignatureElement>, Vec<SignatureElement>);

impl Signature {
    pub fn get_args(&self) -> &Vec<SignatureElement> {
        &self.0
    }

    pub fn get_returns(&self) -> &Vec<SignatureElement> {
        &self.1
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SignatureElement {
    Kind(String, Vec<SignatureElement>),
    Variable(String)
}

fn ident_lexer() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    let punctuation = filter(|c: &char| {
        c.is_ascii_punctuation() && !['[', ']', '{', '}', ':', '(', ')', '"', '\'', '$'].contains(c)
    });

    filter(|c: &char| c.is_alphabetic())
            .or(punctuation)
            .chain(
                filter(|c: &char| c.is_alphanumeric())
                    .or(punctuation)
                    .repeated(),
            )
            .collect::<String>()
}

pub fn sig_lexer() -> impl Parser<char, Signature, Error = Simple<char>> + Clone {
    let elem = recursive(|func|{
        let var = just('\'')
            .ignore_then(ident_lexer())
            .map(|name| SignatureElement::Variable(name));

        let polytypes = func.padded()
            .repeated()
            .delimited_by(just('['), just(']'));
        
        let kind = ident_lexer()
            .then(polytypes)
            .map(|(name, typs)| SignatureElement::Kind(name, typs));

        var.clone().or(kind.clone())
    });

    let side = elem.padded().repeated();

    side.clone()
        .then_ignore(just("--"))
        .then(side)
        .delimited_by(just("("), just(")"))
        .map(|(args, ret)| Signature(args, ret))
}

pub fn lex_signature(src: &str) -> Result<Signature, Error> {
    let (result, errs) = sig_lexer().parse_recovery_verbose(src.to_string());

    match result {
        Some(sig) => Ok(sig),

        None => Err(Error::LexerError { inner: errs })
    }
}