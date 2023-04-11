use std::collections::HashSet;

use crate::parser::node::{Node, Literal, FunctionDefinition};
use crate::parser::signature_parser::TypedSignature;
use crate::parser::types::type_env::{TypeEnv, self};

pub type FuncID = String;

pub struct FuncInfo {
    pub id: FuncID,
    pub definition: FunctionDefinition,
    pub init_env: TypeEnv,
    pub instances: HashSet<TypedSignature>,
    pub captures_vars: HashSet<String>,
    pub calls: HashSet<FuncID>
}

pub struct CodeGraphBuilder {
    funcs: Vec<FuncInfo>
}

impl CodeGraphBuilder {
    fn to_func(
        &mut self,
        definition: FunctionDefinition,
        tenv: TypeEnv,
        name: FuncID,

    ) -> FuncInfo {
        let mut local_vars = HashSet::new();
        let mut captures_vars = HashSet::new();

        let mut calls = HashSet::new();

        for node in nodes {
            match node {
                Node::Assigment { name, typ, .. } => { 
                    local_vars.insert(name);
                },

                Node::Variable { name, .. } => {
                    if !local_vars.contains(name){
                        captures_vars.insert(name.to_string());
                    }
                },

                Node::Call { name, .. } => {
                    calls.insert(name);
                },

                Node::FunctionDefinition { name, definition, .. } => {

                },

                Node::Literal { value: Literal::Function(definition), .. } => {
                    self.to_func(definition, /* only variabls from tenv */, );
                },

                Node::Literal { .. } => ()
            }


        }

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