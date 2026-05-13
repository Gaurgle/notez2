//! `notez search` / `findz`: full-text search across notes (rg + fzf).
//!
//! Placeholder pending rg/fzf integration.

use anyhow::{Result, bail};

use crate::config::Config;

pub fn run(_term: String, _config: &Config) -> Result<()> {
    bail!("search not yet implemented in notez2; coming in the next milestone");
}
