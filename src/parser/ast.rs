use std::collections::{HashMap, HashSet};

use super::{node::Node, types::types::Type};

pub type FuncInstances = HashSet<HashMap<String, Type>>;

pub struct FuncInfo {
    pub instances: FuncInstances,
    pub captured_vars: HashSet<String>
    pub closures: HashMap<String, FuncInfo>
}

pub struct Ast {
    pub nodes: Vec<Node>,
    pub captured_vars: HashSet<String>,
    pub func_info: HashMap<String, FuncInfo>
}

impl Ast {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self {
            nodes,

        }
    }
}

fn analyze_instances(nodes: &Vec<Node>) -> FuncInstances {

}

fn vars_captured_in_closures(nodes: &Vec<Node>) -> HashSet<String> {

}