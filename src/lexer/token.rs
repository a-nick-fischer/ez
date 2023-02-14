use std::ops::Range;

use super::sig_lexer::LexedSignature;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Number { value: f64, range: Range<usize> },
    Quote { value: String, range: Range<usize> },
    Ident { value: String, range: Range<usize> },
    GetIdent { value: String, range: Range<usize> },
    Assigment { value: String, range: Range<usize> },
    List { value: Vec<Token>, range: Range<usize> },
    Function { sig: LexedSignature, body: Vec<Token>, range: Range<usize> },
    Newline
}

impl Token {
    pub fn range(&self) -> &Range<usize> {
        match &self {
            Token::Number { range, .. } => range,

            Token::Quote { range, .. } => range,

            Token::Ident { range, .. } => range,

            Token::GetIdent { range, .. } => range,

            Token::Assigment { range, .. } => range,

            Token::List { range, .. } => range,

            Token::Function { range, .. } => range,

            Token::Newline => unreachable!(),
        }
    }
}