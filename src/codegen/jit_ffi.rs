use core::slice;
use std::fmt::Display;

use crate::parser::types::{type_env::{TypeEnv, TypeBindings}, typ::Type, *, typelist::TypeList};

// The struct is only allocated inside our Jit which should in theory align
// this thing
#[repr(C, packed)]
pub struct RawJitState {
    stack: Option<*const usize>,
    vars: Option<*const usize>
}

impl RawJitState {
    pub fn new() -> Self {
        RawJitState { stack: None, vars: None }
    }

    pub unsafe fn to_jit_state(&self, tenv: &TypeEnv) -> JitState {
        let stack = self.stack
            .map_or_else(Vec::new, 
                |slice| values_from_raw(slice, &tenv.stack));

        let vars = self.vars
            .map_or_else(Vec::new, 
                |slice| values_from_raw(slice, &layout_bindings(&tenv.bindings)));

        JitState { stack, vars }
    }
}

#[derive(Debug)]
pub enum JitValue {
    Number(f64),
    Quote(String),
    List(Vec<JitValue>),
    Other(String, usize)
}

#[derive(Debug)]
pub struct JitState {
    stack: Vec<JitValue>,
    vars: Vec<JitValue>
}

pub fn type_bindings_sorted_keys(bindings: &TypeBindings) -> Vec<String> {
    let mut keys: Vec<String> = bindings.keys().cloned().collect();
    keys.sort();
    keys
}

fn layout_bindings(bindings: &TypeBindings) -> TypeList {
    let mut list = TypeList::new();

     type_bindings_sorted_keys(bindings)
        .iter()
        .map(|key| bindings.get(key).unwrap().clone())
        .for_each(|typ| list.push(typ));

    list
}

unsafe fn values_from_raw(slice_ptr: *const usize, types: &TypeList) -> Vec<JitValue> {
    let slice = slice::from_raw_parts(slice_ptr, types.len());

    slice.iter()
        .zip(types.vec())
        .map(|(ptr, typ)| convert(*ptr, typ))
        .collect()
}

unsafe fn convert(pointer: usize, typ: &Type) -> JitValue {
    match typ {
        Type::Kind(name, _) if name == NUMBER_TYPE_NAME => 
            JitValue::Number(pointer as f64),

        Type::Kind(name, _) if name == QUOTE_TYPE_NAME => {
            let ptr = pointer as *const usize as *const u64;
            let size: &u64 = &*ptr;

            let str_ptr = ptr.offset(1) as *const u8;
            
            let buffer = slice::from_raw_parts(str_ptr, *size as usize);

            let str = String::from_utf8_lossy(buffer).to_string();

            JitValue::Quote(str)
        },

        Type::Kind(name, polytypes) if name == LIST_TYPE_NAME => {
            let ptr = pointer as *const u64;
            let size: &u64 = &*ptr;

            let list_ptr = ptr.offset(1);

            let typ = polytypes.vec().first().unwrap();

            let vals: Vec<JitValue> = (0..*size)
                .into_iter()
                .map(|offset| { 
                    let ioffset: isize = offset.try_into().unwrap();
                    let ptr = list_ptr.offset(ioffset) as *const usize;

                    convert(ptr as usize, typ)
                })
                .collect();

            JitValue::List(vals)
        },

        Type::Kind(name, _) => 
            JitValue::Other(name.clone(), pointer),

        Type::Variable(_, _) => panic!("Variables not allowed"),
    }
}

impl Display for JitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", jit_values_to_str(&self.stack))
    }
}

impl Display for JitValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JitValue::Number(num) => 
                write!(f, "{num}"),

            JitValue::Quote(str) => 
                write!(f, "\"{str}\""),

            JitValue::List(vals) => 
                write!(f, "[{}]", jit_values_to_str(vals)),

            JitValue::Other(name, addr) => 
                write!(f, "{name}<{addr}>"),
        }
    }
}

fn jit_values_to_str(vals: &[JitValue]) -> String {
    vals
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}