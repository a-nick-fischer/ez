use std::collections::{HashMap, HashSet};

use super::{node::{Node, FuncID}, types::{type_env::TypeEnv}, signature_parser::TypedSignature};

pub const MAIN_FUNC_ID: FuncID = 0;
pub const INLINE_THRESHOLD: usize = 10; 

#[derive(Default)]
pub struct FuncInfo {
    pub instances: HashSet<TypedSignature>,
    pub captures_vars: HashSet<String>,
    pub nodes: Vec<Node>
}

impl FuncInfo {
    fn should_inline(&self) -> bool {
        self.nodes.len() <= INLINE_THRESHOLD
    }
}

pub struct AstBuilder {
    funcs: Vec<FuncInfo>
}

impl AstBuilder {
    pub fn new() -> Self {
        Self {
            funcs: Vec::new()
        }
    }

    pub fn add_func(&mut self, nodes: Vec<Node>) -> FuncID {
        self.funcs.push(FuncInfo {
            nodes,
            ..Default::default()
        });

        self.funcs.len() - 1
    }

    pub fn finish(self) -> Ast {
        Ast { funcs: self.funcs }
    }
}

pub struct Ast {
    funcs: Vec<FuncInfo>
}

impl Ast {
    pub fn builder(self) -> AstBuilder {
        AstBuilder { funcs: self.funcs }
    }

    pub fn main(&self) -> &FuncInfo {
        self.func(MAIN_FUNC_ID)
    }

    pub fn func(&self, id: FuncID) -> &FuncInfo {
        self.funcs.get(id)
            .unwrap_or_else(|| panic!("Expect function with id {id} to be defined"))
    }
}
