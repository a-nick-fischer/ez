use std::process::exit;

use ariadne::{Color, Fmt};

use crate::{error::Error, parser::types::types::Type};

pub mod compiler;
pub mod jit;
pub mod translator;
pub mod external_linker;
pub mod function_translator;

fn fail(err: Error, src: String) -> ! {
    err.report(src);
    println!("\n\t{}", "Build failed, aborting".fg(Color::Red));
    exit(1)
}

fn success(){
    println!("\n\t{}", "Build succeeded".fg(Color::Green));
}

fn pointer_type() -> cranelift::prelude::Type {
    cranelift::prelude::types::I64
}

impl Into<cranelift::prelude::Type> for Type {
    fn into(self) -> cranelift::prelude::Type {
        match self {
            Type::Kind(name, _) if name == "num" => cranelift::prelude::types::F64,

            Type::Kind(_, _) => pointer_type(),

            Type::Variable(_, _) => panic!("Variables not allowed"),
        }
    }
}
