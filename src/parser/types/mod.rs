pub mod type_env;
pub mod types;
pub mod typelist;

use std::{rc::Rc, cell::RefCell};

use self::{types::{Type, VarContent}, typelist::TypeList};

pub fn typ(name: &str, inner: Vec<Type>) -> Type {
    Type::Kind(name.to_string(), TypeList::from(inner))
}

pub fn quote_type() -> Type {
   typ("str", vec![])
}

pub fn number_type() -> Type {
    typ("num", vec![])
}

pub fn list_type(inner: Type) -> Type {
    typ("list", vec![inner])
}

pub fn func_type(args: Vec<Type>, result: Vec<Type>) -> Type {
    typ("fun", vec![
        typ("arg", args),
        typ("ret", result)
    ])
}

pub fn var_type(name: &str, value: Option<Type>) -> Type {
    Type::Variable(name.to_string(), Rc::new(RefCell::new(value)))
}

pub fn var_type_raw(name: &str, value: VarContent) -> Type {
    Type::Variable(name.to_string(), value)
}