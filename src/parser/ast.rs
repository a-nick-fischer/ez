use crate::{lexer::token::Token, typechecker::types::Type};

pub enum ASTNode {
    Assigment {
        ident: String,
        value: Box<ASTNode>
    },

    Call {
        ident: String,
        args: Vec<ASTNode>,
        token: Token,
        typ: Vec<Type>
    },

    Push {
        value: Box<ASTNode>,
        token: Token,
        typ: Type
    }
}