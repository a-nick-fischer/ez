pub mod type_env;
pub mod typ;
pub mod typelist;

use std::sync::{Mutex, Arc};

use self::{typ::{Type, VarContent}, typelist::TypeList};

pub fn typ(name: &str, inner: Vec<Type>) -> Type {
    Type::Kind(name.to_string(), TypeList::from(inner))
}

pub const QUOTE_TYPE_NAME: &str = "str";
pub const NUMBER_TYPE_NAME: &str = "num";
pub const LIST_TYPE_NAME: &str = "list";
pub const FUNC_TYPE_NAME: &str = "fun";

pub fn quote_type() -> Type {
   typ(QUOTE_TYPE_NAME, vec![])
}

pub fn number_type() -> Type {
    typ(NUMBER_TYPE_NAME, vec![])
}

pub fn list_type(inner: Type) -> Type {
    typ(LIST_TYPE_NAME, vec![inner])
}

pub fn func_type(args: Vec<Type>, result: Vec<Type>) -> Type {
    typ(FUNC_TYPE_NAME, vec![
        typ("arg", args),
        typ("ret", result)
    ])
}

pub fn var_type(name: &str, value: Option<Type>) -> Type {
    Type::Variable(name.to_string(), Arc::new(Mutex::new(value)))
}

pub fn var_type_raw(name: &str, value: VarContent) -> Type {
    Type::Variable(name.to_string(), value)
}