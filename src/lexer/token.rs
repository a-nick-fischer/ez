use core::fmt;
use std::ops::Range;

use super::sig_lexer::SignatureElement;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Number { value: f32, range: Range<usize> },
    Quote { value: String, range: Range<usize> },
    Ident { value: String, range: Range<usize> },
    GetIdent { value: String, range: Range<usize> },
    Assigment { value: String, range: Range<usize> },
    List { value: Vec<Token>, range: Range<usize> },
    Function { sig: SignatureElement, body: Vec<Token>, range: Range<usize> },
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number { value, .. } => write!(f, "{value}"),

            Self::Quote { value, .. } => write!(f, "{value}"),
            
            Self::Ident { value, .. } => write!(f, "'{value}'"),
            
            //Self::List(a) => write!(f, "[{a}]"),
            
            _ => unimplemented!()
        }
    }
}