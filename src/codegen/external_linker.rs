use std::path::PathBuf;
use std::process::Command;

use crate::{error::{Error, error}, config::LinkageConfig};

const OUTFILE_EXT: &str = if cfg!(unix){ "" } else { "exe" };

pub fn link(input_file: &PathBuf, config: &LinkageConfig) -> Result<(), Error> {
    if config.do_not_link { return Ok(()); }

    let mut command = if let Some(mut command) = config.linker_command.clone() {
        custom_command(&mut command)
    }
    else {
        let mut output_file = input_file.clone();
        output_file.set_extension(OUTFILE_EXT);

        host_command(input_file, &output_file)
    };

    let mut child = command
        .spawn()
        .map_err(|err| 
            error(format!("Could not link object file, is `ld` or `link.exe` in PATH?: {err}")))?;

    let exit_code = child
        .wait()
        .expect("not triggering a compiler bug");

    if !exit_code.success() {
        return Err(error(format!("Linker exited with status code {exit_code}")))
    }
    
    Ok(())
}

#[cfg(target_family = "unix")]
fn host_command(input: &PathBuf, output: &PathBuf) -> Command {
    let mut command = Command::new("ld");
    
    command
        .arg("-pie")
        .arg("-O2")
        .arg("--dynamic-linker=/lib64/ld-linux-x86-64.so.2")
        .arg("-o")
        .arg(output)
        .arg("-lc")
        .arg(input);

    command
}

#[cfg(target_family = "windows")]
fn hosta_command(input: &PathBuf, output: &PathBuf) -> Command {
    let mut command = Command::new("link.exe");
    
    command.arg("-o")
        .arg(output)
        .arg(input);

    command
}

fn custom_command(custom: &mut Vec<String>) -> Command {
    let executable = custom.pop()
        .expect("not triggering a compiler bug");

    let mut command = Command::new(executable);

    for arg in custom {
        command.arg(arg);
    }

    command
}