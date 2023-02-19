use std::fmt::{Display, write};

use super::{typ::Type, type_env::TypeEnv};

#[derive(Debug, PartialEq, Clone)]
pub struct TypeList(Vec<Type>);

impl TypeList {
    pub fn new() -> Self {
        TypeList(vec![])
    }

    pub fn vec(&self) -> &Vec<Type> {
        &self.0
    }

    pub fn pop(&mut self) -> Option<Type> {
        self.0.pop()
    }

    pub fn push(&mut self, typ: Type) {
        self.0.push(typ)
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn extend(&mut self, other: Self) {
        self.0.extend(other.0)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear_vars(&self) { 
        self.0.iter().for_each(|typ| typ.clear_vars())
    }

    pub fn refresh_vars(&self, env: &mut TypeEnv) -> TypeList {
        TypeList(self.0.clone()
            .into_iter()
            .map(|typ| typ.refresh_vars(env))
            .collect())
    }

    pub fn has_bound_vars(&self) -> bool {
        self.0.iter().any(|typ| typ.has_bound_vars())
    }

    pub fn occurs(&self, var: &String) -> bool {
        self.0.iter().any(|t| t.occurs(var))
    }
    
    pub fn concretize(&self) -> TypeList {
        TypeList(self.0
            .clone()
            .into_iter()
            .map(|typ| typ.concretize())
            .collect())
    }
}

impl From<Vec<Type>> for TypeList {
    fn from(value: Vec<Type>) -> Self {
        TypeList(value)
    }
}

impl Display for TypeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = self.vec()
            .into_iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(" ");

        write!(f, "{msg}")
    }
}