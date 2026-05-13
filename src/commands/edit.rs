//! `notez edit` / `editz`: open an existing note (fuzzy-match by filename).
//!
//! Placeholder pending fzf integration.

use anyhow::{Result, bail};

use crate::config::Config;

pub fn run(_term: Option<String>, _config: &Config) -> Result<()> {
    bail!("edit not yet implemented in notez2; coming in the next milestone");
}
