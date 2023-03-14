use std::{process::exit, sync::Arc};

use ariadne::{Color, Fmt};
use cranelift::prelude::{AbiParam, isa::TargetIsa, settings::{*, Flags, self}};
use cranelift_module::ModuleError;

use crate::{error::Error, parser::types::{typ::Type, typelist::TypeList, NUMBER_TYPE_NAME}};

pub mod compiler;
pub mod jit;
pub mod codegen_module;
pub mod external_linker;
pub mod function_translator;
pub mod jit_ffi;

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

fn native_isa() -> Arc<dyn TargetIsa> {
    return match cranelift_native::builder() {
        Ok(builder) => {
            // See https://github.com/bytecodealliance/wasmtime/blob/e4dc9c79443259e40f3e93b9c7815b0645ebd5c4/cranelift/jit/src/backend.rs#L50
            let mut flag_builder = settings::builder();
            /*flag_builder.set("use_colocated_libcalls", "false").unwrap();
            flag_builder.set("is_pic", "true").unwrap();
            flag_builder.set("opt_level", "speed").unwrap();
            flag_builder.set("regalloc_checker", "true").unwrap();
            flag_builder.set("enable_alias_analysis", "true").unwrap();
            flag_builder.set("enable_verifier", "true").unwrap();
            flag_builder.set("enable_probestack", "false").unwrap();*/
            //flag_builder.set("use_egraphs", "true").unwrap();

            let flags = Flags::new(flag_builder);
            builder.finish(flags).unwrap() // TODO Errorhandling
        },

        Err(msg) => panic!("{msg}")
    };
}

impl From<Type> for cranelift::prelude::Type {
    fn from(val: Type) -> Self {
        match val {
            Type::Kind(name, _) if name == NUMBER_TYPE_NAME => cranelift::prelude::types::F64,

            Type::Kind(name, _) if name == "ci32" => cranelift::prelude::types::I32,

            Type::Kind(name, _) if name == "ci64" => cranelift::prelude::types::I64,

            Type::Kind(name, _) if name == "ci128" => cranelift::prelude::types::I128,

            Type::Kind(_, _) => pointer_type(),

            Type::Variable(_, _) => panic!("Variables not allowed"),
        }
    }
}

impl From<TypeList> for Vec<AbiParam> {
    fn from(val: TypeList) -> Self {
        val.vec()
            .iter()
            .map(|typ| AbiParam::new(typ.clone().into()))
            .collect()
    }
}

impl From<ModuleError> for Error {
    fn from(value: ModuleError) -> Self {
        Error::General { message: value.to_string() }
    }
}