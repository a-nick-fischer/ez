use std::process::exit;

use ariadne::{Color, Fmt};
use cranelift::prelude::AbiParam;
use cranelift_module::ModuleError;

use crate::{error::Error, parser::types::{types::Type, typelist::TypeList}};

pub mod compiler;
pub mod jit;
pub mod codegen;
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

impl Into<Vec<AbiParam>> for TypeList {
    fn into(self) -> Vec<AbiParam> {
        self.vec()
            .into_iter()
            .map(|typ| AbiParam::new(typ.clone().into()))
            .collect()
    }
}

impl From<ModuleError> for Error {
    fn from(value: ModuleError) -> Self {
        Error::GeneralError { message: value.to_string() }
    }
}