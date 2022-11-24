use core::fmt;

use chumsky::prelude::*;

use crate::{error::{Spaned, TErr}};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Number(f32),
    Quote(String),
    Ident(String),
    Assigment(String),
    List(Vec<Token>),
    Function(SignatureElement, Vec<Spaned<Token>>),
    Newline
}

#[derive(Clone, Debug, PartialEq)]
pub enum SignatureElement {
    Kind(String, Vec<SignatureElement>),
    Function(Vec<SignatureElement>, Vec<SignatureElement>),
    Variable(String)
}


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number(a) => write!(f, "{a}"),
            Self::Quote(a) => write!(f, "{a}"),
            Self::Ident(a) => write!(f, "'{a}'"),
            //Self::List(a) => write!(f, "[{a}]"),
            _ => unimplemented!()
        }
    }
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

fn sig_lexer() -> impl Parser<char, SignatureElement, Error = Simple<char>> + Clone {
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


fn lexer() -> impl Parser<char, Vec<Spaned<Token>>, Error = Simple<char>> {
    let pad = one_of(" \t").repeated();

    recursive::<char, Spaned<Token>, _, _, Simple<char>>(|rec| {
        let number = just('-')
            .or_not()
            .chain(text::int(10))
            .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
            .collect::<String>()
            .labelled("number")
            .map_with_span(|str, span| Spaned::new(Token::Number(str.parse().unwrap()), span));

        let ident = ident_lexer()
            .labelled("identifier")
            .map_with_span(|str, span| Spaned::new(Token::Ident(str), span));

        let assigment = ident_lexer()
            .then_ignore(just(':'))
            .labelled("assigment")
            .map_with_span(|str, span| Spaned::new(Token::Assigment(str), span));

        let escape = just('\\')
            .ignore_then(
                just('\\')
                    .or(just('/'))
                    .or(just('"'))
                    .or(just('n').to('\n')),
            )
            .labelled("escape character");

        let string = just('"')
            .ignore_then(filter(|c| *c != '\\' && *c != '"').or(escape).repeated())
            .then_ignore(just('"'))
            .collect::<String>()
            .labelled("string")
            .map_with_span(|str, span| Spaned::new(Token::Quote(str), span));

        let block = rec
            .clone()
            .padded()
            .repeated()
            .delimited_by(just('['), just(']'))
            .labelled("block")
            .map_with_span(|list, span| Spaned::new(
                Token::List(
                    list.into_iter().map(|spaned| spaned.content().clone()).collect()
                ),
                span));

        let function_body = rec
            .clone()
            .padded()
            .repeated()
            .delimited_by(just('{'), just('}'));

        let function = sig_lexer()
            .padded()
            .then(function_body)
            .map_with_span(|(sig, body), span| {
                if let SignatureElement::Function(_, _) = sig {
                    Spaned::new(Token::Function(sig, body), span)
                }
                else {
                    panic!("{:?} | {:?}", sig, body);
                }
            });

        let newline = just('\n')
            .labelled("newline")
            .map_with_span(|_, span| Spaned::new(Token::Newline, span));

        string
            .or(number)
            .or(assigment)
            .or(ident)
            .or(function)
            .or(block)
            .or(newline)
    })
    .padded_by(pad)
    .repeated()
    .then_ignore(end())
    //.recover_with(nested_delimiters('[', ']', [], |_| vec![Token::Invalid]))
    .recover_with(skip_then_retry_until([']', '"']))
}

fn preprocess_tokens(tokens: Vec<Spaned<Token>>) -> Vec<Spaned<Token>> {
    tokens
        .split(|token| token.content() == &Token::Newline)
        .map(|vec| vec.into_iter().rev().cloned().collect::<Vec<Spaned<Token>>>())
        .flatten()
        .collect()
}

pub fn lex_sig(src: &str) -> Result<SignatureElement, TErr> {
    let (result, errs) = sig_lexer().parse_recovery_verbose(src.to_string());

    match result {
        Some(SignatureElement::Function(_, _)) => Ok(result.unwrap()),

        Some(_) => panic!("Not a function"), // Change to error later

        None => Err(errs)
    }
}

pub fn lex(src: &str) -> Result<Vec<Spaned<Token>>, TErr> {
    let (tokens, errs) = lexer().parse_recovery_verbose(src.to_string());

    tokens.map(preprocess_tokens).ok_or(errs)
}

pub fn num(val: f32) -> Spaned<Token> {
    Spaned::new(Token::Number(val), 0..1)
}

pub fn str(val: String) -> Spaned<Token> {
    Spaned::new(Token::Quote(val), 0..1)
}
