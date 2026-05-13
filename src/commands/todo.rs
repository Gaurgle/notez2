//! `notez todo` / `todoz`: interactive todo manager TUI.
//!
//! Placeholder. The current notez-cli todo TUI is the most feature-rich part
//! of the project (subtasks, tags, drag-to-reorder, code TODO scanning); it
//! ports to notez2 in a follow-up. The TUI logic itself moves with minor
//! changes; only the file-source layer changes (registry-based instead of
//! symlinks).

use anyhow::{Result, bail};

use crate::config::Config;
use crate::core::Scope;

pub fn run(_item: Option<String>, _scope: Scope, _config: &Config) -> Result<()> {
    bail!("todo TUI not yet implemented in notez2; coming in the next milestone");
}
