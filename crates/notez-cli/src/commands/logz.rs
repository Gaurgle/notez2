//! `notez logz` / `logs` / `zlogs`: open the daily-logs directory.
//!
//! Resolution goes through the scope model, so from a subdirectory this
//! opens the project's logs rather than creating a new store in place.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use notez_core::config::Config;
use notez_core::core::{Scope, resolve};

/// Resolve the daily-logs directory for `scope` and create it if absent.
///
/// Creating here rather than failing keeps the first run after a fresh
/// install from erroring on a directory that simply does not exist yet.
pub fn prepare(scope: Scope, config: &Config) -> Result<PathBuf> {
    let dir = resolve::daily_logs(scope, config)?;
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(dir)
}

/// Open `dir` in yazi when available, otherwise in the configured editor.
pub fn open_dir(dir: &Path, config: &Config) -> Result<()> {
    let program = if config.tools.yazi {
        "yazi"
    } else {
        config.editor.command.as_str()
    };
    Command::new(program)
        .arg(dir)
        .status()
        .with_context(|| format!("failed to launch {}", program))?;
    Ok(())
}

pub fn run(scope: Scope, config: &Config) -> Result<()> {
    let dir = prepare(scope, config)?;
    open_dir(&dir, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[serial_test::serial]
    fn prepare_creates_the_logs_dir_under_the_project_root() {
        let dir = tempdir().unwrap();
        let toplevel = dir.path().canonicalize().unwrap();
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&toplevel)
            .status()
            .unwrap();
        let sub = toplevel.join("src").join("nested");
        std::fs::create_dir_all(&sub).unwrap();

        let mut config = Config::defaults();
        config.paths.daily_logs_dir = "01_daily-logs".to_string();

        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(&sub).unwrap();
        let got = prepare(Scope::Local, &config);
        std::env::set_current_dir(saved).unwrap();

        let expected = toplevel.join(".notez").join("01_daily-logs");
        let got = got.unwrap();
        assert_eq!(got, expected);
        assert!(got.is_dir(), "prepare should create the directory");
    }
}
