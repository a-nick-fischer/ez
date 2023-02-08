use chumsky::prelude::*;

use super::{token::Token, sig_lexer::sig_lexer};

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

pub fn lexer() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let pad = one_of(" \t").repeated();

    recursive::<char, Token, _, _, Simple<char>>(|rec| {
        let number = just('-')
            .or_not()
            .chain(text::int(10))
            .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
            .collect::<String>()
            .labelled("number")
            .map_with_span(|str, span| 
                Token::Number { value: str.parse().unwrap(), range: span });

        let ident = ident_lexer()
            .labelled("identifier")
            .map_with_span(|str, span| 
                Token::Ident { value: str, range: span });

        let get_ident = just(':')
                .ignore_then(ident_lexer())
                .labelled("get-identifier")
                .map_with_span(|str, span| 
                    Token::GetIdent { value: str, range: span });

        let assigment = ident_lexer()
            .then_ignore(just(':'))
            .labelled("assigment")
            .map_with_span(|str, span| 
                Token::Assigment { value: str, range: span });

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
            .map_with_span(|str, span| 
                Token::Quote { value: str, range: span });

        let block = rec
            .clone()
            .padded()
            .repeated()
            .delimited_by(just('['), just(']'))
            .labelled("block")
            .map_with_span(|list, span| 
                Token::List { value: list, range: span });

        let function_body = rec
            .clone()
            .padded()
            .repeated()
            .delimited_by(just('{'), just('}'));

        let function = sig_lexer()
            .padded()
            .then(function_body)
            .map_with_span(|(sig, body), span| 
                Token::Function { sig, body, range: span });

        let newline = just('\n')
            .labelled("newline")
            .map(|_| Token::Newline);

        string
            .or(number)
            .or(assigment)
            .or(ident)
            .or(get_ident)
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
