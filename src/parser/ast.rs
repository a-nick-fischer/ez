use std::collections::{HashMap, HashSet};

use super::{node::{Node, FuncID}, types::{type_env::TypeEnv}, signature_parser::TypedSignature};

pub const MAIN_FUNC_ID: FuncID = 0;
pub const INLINE_THRESHOLD: usize = 10; 

pub struct FuncInfo {
    pub instances: HashSet<TypedSignature>,
    pub captured_vars: HashSet<String>,
    pub nodes: Vec<Node>
}

impl FuncInfo {
    fn should_inline(&self) -> bool {
        self.nodes.len() <= INLINE_THRESHOLD
    }
}

pub struct Ast {
    funcs: HashMap<FuncID, FuncInfo>
}

impl Ast {
    pub fn new() -> Self {
        Self {
            funcs: HashMap::new()
        }
    }

    pub fn main(&self) -> &FuncInfo {
        self.func(MAIN_FUNC_ID)
    }

    pub fn func(&self, id: FuncID) -> &FuncInfo {
        self.funcs.get(&id)
            .unwrap_or_else(|| panic!("Expect function with id {id} to be defined"))
    }
}

fn analyze_instances(nodes: &Vec<Node>) -> HashSet<TypedSignature> {
    todo!()
}

fn vars_captured_in_closures(nodes: &Vec<Node>) -> HashSet<String> {
    todo!()
}
