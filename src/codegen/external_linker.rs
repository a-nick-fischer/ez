use std::path::Path;

use crate::{Config, error::{Error, self}, Commands};

pub fn link<P: AsRef<Path>>(path: &P, config: &Config) -> Result<(), Vec<Error>> {
    if let Some(Commands::Compile { do_not_link, linker_command, .. }) = config.command {
        if do_not_link { return Ok(()) }

        // TODO linking stuff
        todo!()

    }
    else { unreachable!() }
}