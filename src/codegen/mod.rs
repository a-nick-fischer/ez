use std::process::exit;

use ariadne::{Color, Fmt};

use crate::error::Error;

pub mod compiler;
pub mod jit;
pub mod translator;
pub mod external_linker;

fn fail(err: Error, src: String) -> ! {
    err.report(src);
    println!("\n\t{}", "Build failed, aborting".fg(Color::Red));
    exit(1)
}

fn success(){
    println!("\n\t{}", "Build succeeded".fg(Color::Green));
}