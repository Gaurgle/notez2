//! `notez add` and `znote`: create a new note.

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cli;
use crate::config::Config;
use crate::core::{Note, Scope};

/// Where a new note should be written.
fn target_dir(config: &Config, scope: Scope, _in_arg: Option<&str>, _in_local: bool) -> Result<PathBuf> {
    // Today's scope: minimal implementation. `--in` and `--in-local` picker
    // semantics will be wired in when fzf-integration lands.
    let dir = match scope {
        Scope::Global => config.quick_notes_path(),
        Scope::Public => {
            let cwd = std::env::current_dir()?;
            cwd.join("notez").join(&config.paths.quick_notes_dir)
        }
        Scope::Private => {
            let cwd = std::env::current_dir()?;
            cwd.join(".notez").join(&config.paths.quick_notes_dir)
        }
    };
    Ok(dir)
}

/// Write the new note to disk and return its absolute path.
pub fn run(
    title_words: Vec<String>,
    in_arg: Option<String>,
    in_local: bool,
    scope: Scope,
    config: &Config,
) -> Result<PathBuf> {
    let (title, body) = cli::split_title_body(title_words);
    let title = title.unwrap_or_else(|| "untitled".to_string());

    let note = Note::new(title, body);
    let dir = target_dir(config, scope, in_arg.as_deref(), in_local)?;
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create note dir {}", dir.display()))?;

    let path = dir.join(note.filename());
    std::fs::write(&path, note.rendered())
        .with_context(|| format!("failed to write note {}", path.display()))?;

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
    fn add_global_writes_into_quick_notes() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());

        let path = run(
            vec!["my".into(), "first".into(), "note".into()],
            None,
            false,
            Scope::Global,
            &config,
        )
        .unwrap();

        assert!(path.exists());
        let parent = path.parent().unwrap();
        assert!(parent.ends_with("00_quick-notes"));

        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("# my first note"));
    }

    #[test]
    fn add_private_writes_under_dot_notez_in_cwd() {
        let cwd_holder = tempdir().unwrap();
        // Temporarily change cwd to the temp dir.
        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(cwd_holder.path()).unwrap();

        let result = run(
            vec!["hello".into()],
            None,
            false,
            Scope::Private,
            &Config::defaults(),
        );

        // Restore cwd before asserting so a panic in assert doesn't leave the
        // process in a weird state.
        std::env::set_current_dir(saved).unwrap();

        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("/.notez/00_quick-notes/"));
    }

    #[test]
    fn add_with_body_includes_body() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());

        let path = run(
            vec!["title".into(), "this is the body".into()],
            None,
            false,
            Scope::Global,
            &config,
        )
        .unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("# title"));
        assert!(body.contains("this is the body"));
    }

    #[test]
    fn empty_title_becomes_untitled() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());

        let path = run(vec![], None, false, Scope::Global, &config).unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.starts_with("# untitled\n"));
    }
}
