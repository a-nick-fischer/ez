use core::slice;
use std::mem;

use crate::{parser::types::{type_env::TypeEnv, types::Type, *}, error::Error};

// The struct is only allocated inside our Jit which should in theory align
// this thing
#[repr(C)]
struct RawJitState<'a> {
    stack: &'a [*const usize],
    vars: &'a [*const usize]
}

enum JitValue {
    Number(f64),
    Quote(String),
    List(Vec<JitValue>),
    Other(String, usize)
}

struct JitState {
    stack: Vec<JitValue>,
    vars: Vec<JitValue>
}

pub fn print_stack(){
   
}

pub fn to_jit_state(pointer: *const usize, type_env: &TypeEnv) -> Result<JitState, Error> {
    unsafe {
        let raw_state: &RawJitState = mem::transmute(pointer); // TODO Can we transmute a slice like that?
    }

    todo!()
}

unsafe fn convert(pointer: *const usize, typ: Type) -> JitValue {
    match typ {
        Type::Kind(name, _) if name == NUMBER_TYPE_NAME => todo!(),

        Type::Kind(name, _) if name == QUOTE_TYPE_NAME => todo!(),

        Type::Kind(name, _) if name == LIST_TYPE_NAME => todo!(),

        Type::Kind(name, _) => JitValue::Other(name, pointer as *const _ as usize),

        Type::Variable(_, _) => panic!("Variables not allowed"),
    }
}