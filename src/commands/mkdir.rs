//! `notez mkdir`: create a new subdirectory under the scope's root.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::config::Config;
use crate::core::{Scope, resolve};
use crate::util::sanitize;

/// Create the directory. Returns its absolute path.
pub fn run(name_words: Vec<String>, scope: Scope, config: &Config) -> Result<PathBuf> {
    let raw = name_words.join(" ");
    let cleaned = sanitize::name(&raw);
    if cleaned.is_empty() {
        bail!("directory name cannot be empty");
    }

    let root = resolve::root(scope, config)?;
    let path = root.join(&cleaned);
    std::fs::create_dir_all(&path)
        .with_context(|| format!("failed to create directory {}", path.display()))?;

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
    fn creates_global_subdir() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());

        let p = run(vec!["my".into(), "ideas".into()], Scope::Global, &config).unwrap();
        assert!(p.is_dir());
        assert!(p.ends_with("my-ideas"));
    }

    #[test]
    fn empty_name_errors() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());
        let err = run(vec!["   ".into()], Scope::Global, &config).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn sanitizes_specials() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());
        let p = run(vec!["Hello, World!".into()], Scope::Global, &config).unwrap();
        assert!(p.ends_with("hello-world"));
    }
}
