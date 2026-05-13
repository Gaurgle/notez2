//! `notez mkdir`: create a new subdirectory under the scope's root.
//!
//! Unlike notez-cli, notez2 does not allocate numbered prefixes. The user
//! supplies the full directory name (sanitized). If the user wants the
//! `NN_name` convention they can include the prefix themselves.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::config::Config;
use crate::core::Scope;
use crate::util::sanitize;

fn root_for(config: &Config, scope: Scope) -> Result<PathBuf> {
    Ok(match scope {
        Scope::Global => config.notez_root_path(),
        Scope::Public => std::env::current_dir()?.join("notez"),
        Scope::Private => std::env::current_dir()?.join(".notez"),
    })
}

/// Create the directory. Returns its absolute path.
pub fn run(name_words: Vec<String>, scope: Scope, config: &Config) -> Result<PathBuf> {
    let raw = name_words.join(" ");
    let cleaned = sanitize::name(&raw);
    if cleaned.is_empty() {
        bail!("directory name cannot be empty");
    }

    let root = root_for(config, scope)?;
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
