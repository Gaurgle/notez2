//! `notez add` and `znote`: create a new note.

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cli;
use crate::config::Config;
use crate::core::{Note, Scope, resolve};

/// Write the new note to disk and return its absolute path.
pub fn run(
    title_words: Vec<String>,
    _in_arg: Option<String>,
    _in_local: bool,
    scope: Scope,
    config: &Config,
) -> Result<PathBuf> {
    let (title, body) = cli::split_title_body(title_words);
    let title = title.unwrap_or_else(|| "untitled".to_string());

    let note = Note::new(title, body);
    let dir = resolve::quick_notes(scope, config)?;
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
    #[serial_test::serial]
    fn add_local_writes_under_dot_notez_in_cwd() {
        let cwd_holder = tempdir().unwrap();
        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(cwd_holder.path()).unwrap();

        let result = run(
            vec!["hello".into()],
            None,
            false,
            Scope::Local,
            &Config::defaults(),
        );

        std::env::set_current_dir(saved).unwrap();

        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("/.notez/00_quick-notes/"));
    }

    #[test]
    #[serial_test::serial]
    fn add_personal_falls_back_to_global_outside_git() {
        let notez_root = tempdir().unwrap();
        let config = config_in(notez_root.path());

        let cwd = tempdir().unwrap();
        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(cwd.path()).unwrap();

        let result = run(vec!["hi".into()], None, false, Scope::Personal, &config);

        std::env::set_current_dir(saved).unwrap();

        let path = result.unwrap();
        // No git project => personal falls back to the global notez_root.
        let expected_parent = notez_root.path().join("00_quick-notes");
        assert_eq!(path.parent().unwrap(), expected_parent);
    }

    #[test]
    #[serial_test::serial]
    fn add_personal_inside_git_uses_personal_subdir() {
        let notez_root = tempdir().unwrap();
        let config = config_in(notez_root.path());

        let project_dir = tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(project_dir.path())
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();

        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(project_dir.path()).unwrap();
        let result = run(vec!["note".into()], None, false, Scope::Personal, &config);
        std::env::set_current_dir(saved).unwrap();

        let path = result.unwrap();
        assert!(
            path.to_string_lossy().contains("/personal/"),
            "expected path under personal/, got {:?}",
            path,
        );
        assert!(
            path.ends_with(std::path::Path::new("00_quick-notes")
                .join(path.file_name().unwrap()))
                || path.parent().unwrap().ends_with("00_quick-notes"),
        );
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
