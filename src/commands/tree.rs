//! `notez tree` / `treez`: interactive tree browser TUI.
//!
//! Placeholder: full port of the notez-cli tree TUI lands in a follow-up.
//! The new data source will be the registry plus the global notez root,
//! eliminating symlink walking entirely.

use anyhow::{Result, bail};

use crate::config::Config;
use crate::core::Scope;

pub fn run(_scope: Scope, _config: &Config) -> Result<()> {
    bail!("tree TUI not yet implemented in notez2; coming in the next milestone");
}
