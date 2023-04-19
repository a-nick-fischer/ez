use std::collections::{HashSet, HashMap};
use std::fmt::format;

use crate::parser::node::{Node, Literal};
use crate::parser::signature_parser::TypedSignature;
use crate::parser::types::type_env::{TypeEnv, self, TypeBindings};

pub type FuncID = String;

pub struct FuncInfo {
    pub id: FuncID,
    pub sig: TypedSignature,
    pub nodes: Vec<Node>,
    pub instances: HashSet<TypedSignature>,
    pub captures_vars: HashSet<String>,
    pub calls: Vec<(FuncID, TypedSignature)>
}

pub struct CodeGraphBuilder {
    funcs: Vec<FuncInfo>
}

impl CodeGraphBuilder {
    fn analyze(
        &mut self,
        id: FuncID,
        sig: TypedSignature,
        nodes: &Vec<Node>,
        bindings: TypeBindings,
        mut scoped_names: HashMap<String, String>
    ) {
        let mut local_vars = HashSet::new();
        let mut captures_vars = HashSet::new();

        let mut calls = Vec::new();
        let mut env = TypeEnv::new(&bindings);

        let mut closure_idx = 0;

        for node in nodes {
            match node {
                Node::Assigment { name, .. } => {
                    scoped_names.insert(name.to_owned(), format!("{id}${name}")); // WRONG
                    local_vars.insert(name);
                    node.apply(&mut env).unwrap();
                },

                Node::Variable { name, .. } => {
                    if !local_vars.contains(name){
                        captures_vars.insert(name.to_string());
                    }
                    node.apply(&mut env).unwrap();
                },

                Node::Call { name, arguments, returns, .. } => {
                    let actual_args = env.stack.clone_top(arguments.len());

                    node.apply(&mut env).unwrap();

                    let actual_returns = env.stack.clone_top(returns.len());

                    calls.push((
                        scoped_names.get(name).unwrap().clone(),
                        TypedSignature::new(actual_args.into(), actual_returns.into())));
                },

                Node::Literal { value: Literal::Function(sig, nodes), .. } => {
                    self.analyze(
                        format!("{id}${closure_idx}"), 
                        sig.clone(),
                        nodes, 
                        env.bindings.clone(),
                        scoped_names.clone()
                    );

                    closure_idx += 1;

                    node.apply(&mut env).unwrap();
                }

                Node::Literal { .. } => {
                    node.apply(&mut env).unwrap();
                }
            }
        }


        let info = FuncInfo {
            id,
            sig,
            nodes: nodes.to_vec(),
            instances: HashSet::new(),
            captures_vars,
            calls
        };

        todo!()
    }
}


// Task:
// - Find what variables a function captures / if it's a closure
//   - A function captures all variables which are used but not declared in it
//   - A single exception are non-closure functions, which are never captured
//
// - Find with which types a function is called
//
// - Mangle function names (needed if two function with equal names exist in different scopes)