use chumsky::prelude::*;

use crate::error::TErr;

#[derive(Clone, Debug, PartialEq)]
pub enum SignatureElement {
    Kind(String, Vec<SignatureElement>),
    Function(Vec<SignatureElement>, Vec<SignatureElement>),
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

pub fn sig_lexer() -> impl Parser<char, SignatureElement, Error = Simple<char>> + Clone {
    recursive(|func|{
        let var = just('\'')
            .ignore_then(ident_lexer())
            .map(|name| SignatureElement::Variable(name));

        let polytype = func.delimited_by(just('['), just(']'));
        
        let kind = ident_lexer()
            .then(polytype.repeated())
            .map(|(name, typs)| SignatureElement::Kind(name, typs));

        let elem = var.clone().or(kind.clone());

        let elem_list = elem
            .padded()
            .repeated()
            .delimited_by(just('('), just(')'));

        let function_sig =
            elem_list
            .clone()
            .padded()
            .then_ignore(just("->"))
            .padded()
            .then(elem_list)
            .map(|(a, b)| SignatureElement::Function(a, b));
    
        choice((function_sig, var, kind))
    })
}

pub fn lex_sig(src: &str) -> Result<SignatureElement, TErr> {
    let (result, errs) = sig_lexer().parse_recovery_verbose(src.to_string());

    match result {
        Some(SignatureElement::Function(_, _)) => Ok(result.unwrap()),

        Some(_) => panic!("Not a function"), // Change to error later

        None => Err(errs)
    }
}