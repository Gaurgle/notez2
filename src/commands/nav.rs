//! `notez nav` (and `notez -n`): launch the global directory picker, then yazi.

use anyhow::{Result, bail};

use crate::config::Config;

pub fn run(_config: &Config) -> Result<()> {
    bail!("nav not yet implemented in notez2; coming in the next milestone");
}
