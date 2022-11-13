use std::fmt::{self, write};

use chumsky::prelude::*;

use crate::{error::{Spaned, TErr}};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Number(f32),
    String(String),
    Ident(String),
    List(Vec<Token>),
    Function(SignatureElement, Vec<Spaned<Token>>),
    Newline,
    Invalid,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SignatureElement {
    Kind(String, Vec<SignatureElement>),
    Function(Vec<SignatureElement>, Vec<SignatureElement>),
    Variable(String)
}

/* 
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number(a) => write!(f, "{a}"),
            Self::String(a) => write!(f, "\"{a}\""),
            Self::Ident(a) => write!(f, "'{a}'"),
            Self::List(a) => write!(f, "[{a}]"),
            Self::Function(args, ) => write!(f, "({sig})"),
            _ => unimplemented!()
        }
    }
}*/

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

        let punctuation = filter(|c: &char| {
            c.is_ascii_punctuation() && !['[', ']', '{', '}', '(', ')', '"', '\'', '$'].contains(c)
        });

        let ident_raw = filter(|c: &char| c.is_alphabetic())
            .or(punctuation)
            .chain(
                filter(|c: &char| c.is_alphanumeric())
                    .or(punctuation)
                    .repeated(),
            )
            .collect::<String>();

        let ident = ident_raw
            .labelled("identifier")
            .map_with_span(|str, span| Spaned::new(Token::Ident(str), span));

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
            .map_with_span(|str, span| Spaned::new(Token::String(str), span));

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


        let function_signature = recursive(|func|{
            let var = just('\'')
                .ignore_then(ident_raw)
                .map(|name| SignatureElement::Variable(name));

            let polytype = func.delimited_by(just('['), just(']'));
            
            let kind = ident_raw
                .then(polytype.repeated())
                .map(|(name, typs)| SignatureElement::Kind(name, typs));

            let elem = var.or(kind.clone());

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
        });

        let function_body = rec
            .clone()
            .padded()
            .repeated()
            .delimited_by(just('{'), just('}'));

        let function = function_signature
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

pub fn lex(src: &str) -> Result<Vec<Spaned<Token>>, TErr> {
    let (tokens, errs) = lexer().parse_recovery_verbose(src.to_string());

    tokens.map(preprocess_tokens).ok_or(errs)
}