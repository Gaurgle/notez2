//! `notez log` and `zlog`: append a timestamped entry to today's daily log.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::config::Config;
use crate::core::{Scope, note, resolve};

/// Append a log entry. Returns the file path written to.
pub fn run(message_words: Vec<String>, scope: Scope, config: &Config) -> Result<PathBuf> {
    let message = message_words.join(" ").trim().to_string();
    if message.is_empty() {
        bail!("log message cannot be empty");
    }

    let dir = resolve::daily_logs(scope, config)?;
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create log dir {}", dir.display()))?;

    let path = dir.join(note::todays_log_filename());
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let updated = note::append_log_entry(&existing, &message);
    std::fs::write(&path, updated)
        .with_context(|| format!("failed to write log {}", path.display()))?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn config_in(root: &std::path::Path) -> Config {
        let mut c = Config::defaults();
        c.paths.notez_root = root.to_string_lossy().into_owned();
        c
    }

    #[test]
    fn empty_message_errors() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());
        let err = run(vec!["".into()], Scope::Global, &config).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn first_call_creates_file_with_header() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());

        let path = run(
            vec!["first".into(), "entry".into()],
            Scope::Global,
            &config,
        )
        .unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.starts_with("# Daily Log - "));
        assert!(body.contains(" - first entry"));
    }

    #[test]
    fn second_call_appends() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());

        run(vec!["first".into()], Scope::Global, &config).unwrap();
        let path = run(vec!["second".into()], Scope::Global, &config).unwrap();

        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains(" - first"));
        assert!(body.contains(" - second"));
        assert!(body.starts_with("# Daily Log - "));
    }

    #[test]
    #[serial_test::serial]
    fn personal_outside_git_falls_back_to_global() {
        let notez_root = tempdir().unwrap();
        let config = config_in(notez_root.path());

        let cwd = tempdir().unwrap();
        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(cwd.path()).unwrap();
        let result = run(vec!["hi".into()], Scope::Personal, &config);
        std::env::set_current_dir(saved).unwrap();

        let path = result.unwrap();
        // Personal fallback puts the log under <notez_root>/01_daily-logs/.
        assert_eq!(
            path.parent().unwrap(),
            notez_root.path().join("01_daily-logs"),
        );
    }
}
