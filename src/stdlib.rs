use std::collections::HashMap;
use std::sync::Arc;

use crate::env::*;
use crate::types::*;

lazy_static! {
    pub static ref STDLIB: Bindings = stdlib();
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

        impl EnvAction for $struct_name {
            fn act(&self, env: &mut Env) {
                $act(env);
            }

            fn signature(&self, _: &TypeEnv) -> Result<Arc<dyn TypeEnvMod>, String> {
                Ok(Arc::new(Signature::new($sig)))
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
    Add ( "(num num) -> (num)" ) |env: &mut Env| {
        match (env.pop(), env.pop()){
            (Number(a), Number(b)) => env.push(num(a + b)),

            _ => panic!()
        }
    }
);

define_func!(
    Sub ( "(num num) -> (num)" ) |env: &mut Env| {
        match (env.pop(), env.pop()){
            (Number(a), Number(b)) => env.push(num(a - b)),

            _ => panic!()
        }
    }
);

define_func!(
    Mul ( "(num num) -> (num)" ) |env: &mut Env| {
        match (env.pop(), env.pop()){
            (Number(a), Number(b)) => env.push(num(a * b)),

            _ => panic!()
        }
    }
);

define_func!(
    Dup ( "('a) -> ('a 'a)" ) |env: &mut Env| {
        let val = env.pops();
        env.push(val.clone());
        env.push(val);
    }
);

define_func!(
    Swap ( "('a 'b) -> ('b 'a)" ) |env: &mut Env| {
        let a = env.pops();
        let b = env.pops();
        env.push(a);
        env.push(b);
    }
);

define_func!(
    Over ( "('a 'b 'c) -> ('c 'b 'a)" ) |env: &mut Env| {
        let a = env.pops();
        let b = env.pops();
        let c = env.pops();
        env.push(a);
        env.push(b);
        env.push(c);
    }
);

define_func!(
    Drop ( "('a) -> ()" ) |env: &mut Env| {
        env.pops();
    }
);

define_func!(
    Print ( "('a) -> ()" ) |env: &mut Env| {
        println!("{}", env.pop())
    }
);


pub fn stdlib() -> Bindings {
    let mut map: HashMap<String, Action> = HashMap::new();

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