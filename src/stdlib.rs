use std::collections::HashMap;
use std::sync::Arc;

use crate::lexer::*;
use crate::lexer::Token::*;
use crate::env::*;
use crate::types::*;

lazy_static! {
    pub static ref STDLIB: Bindings<'static> = stdlib();
}

macro_rules! define_func {
    ($struct_name:ident ( $sig:literal ) $act:expr) => {
        #[derive(Debug)]
        struct $struct_name {}

        impl $struct_name {
            fn new() -> Self {
                Self {}
            }
        }

        impl<'a> EnvAction<'a> for $struct_name {
            fn act(&self, env: &'a mut Env<'a>) {
                $act(env);
            }

            fn signature(&self, _: &TypeEnv) -> Result<Signature, String> {
                Ok(Signature::new($sig))
            }
        }
    }
}

macro_rules! add_funcs {
    ( $map:expr, $($key:literal => $val:ty),* ) => {
        $(($map).insert($key.to_owned(), Arc::new(<$val>::new())));*  
    };
}

define_func!(
    Add ( "(num num) -> (num)" ) |env: &'a mut Env<'a>| {
        match (env.pop(), env.pop()){
            (Number(a), Number(b)) => env.push(num(a + b)),

            _ => panic!()
        }
    }
);

define_func!(
    Sub ( "(num num) -> (num)" ) |env: &'a mut Env<'a>| {
        match (env.pop(), env.pop()){
            (Number(a), Number(b)) => env.push(num(a - b)),

            _ => panic!()
        }
    }
);

define_func!(
    Mul ( "(num num) -> (num)" ) |env: &'a mut Env<'a>| {
        match (env.pop(), env.pop()){
            (Number(a), Number(b)) => env.push(num(a * b)),

            _ => panic!()
        }
    }
);

define_func!(
    Dup ( "('a) -> ('a 'a)" ) |env: &'a mut Env<'a>| {
        let val = env.pops();
        env.push(val.clone());
        env.push(val);
    }
);

define_func!(
    Swap ( "('a 'b) -> ('b 'a)" ) |env: &'a mut Env<'a>| {
        let a = env.pops();
        let b = env.pops();
        env.push(a);
        env.push(b);
    }
);

define_func!(
    Over ( "('a 'b 'c) -> ('c 'b 'a)" ) |env: &'a mut Env<'a>| {
        let a = env.pops();
        let b = env.pops();
        let c = env.pops();
        env.push(a);
        env.push(b);
        env.push(c);
    }
);

define_func!(
    Drop ( "('a) -> ()" ) |env: &'a mut Env<'a>| {
        env.pops();
    }
);

define_func!(
    Print ( "('a) -> ()" ) |env: &'a mut Env<'a>| {
        println!("{}", env.pop())
    }
);


pub fn stdlib<'a>() -> Bindings<'a> {
    let mut map: HashMap<String, Action<'a>> = HashMap::new();

    add_funcs!(map,
        "add" => Add,
        "sub" => Sub,
        "mul" => Mul,
        "dup" => Dup,
        "swap" => Swap,
        "over" => Over,
        "_" => Drop,
        "print" => Print
    );

    map
}