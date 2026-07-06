//! `notez add` and `znote`: create a new note.
//!
//! `--in <dir>` targets a subdirectory instead of quick-notes: under the
//! global root by default, under the current scope's root with `--in-local`
//! (legacy semantics). Bare `--in` opens an fzf picker over the existing
//! subdirectories of that root.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::cli;
use notez_core::config::Config;
use notez_core::core::{Note, Scope, project, resolve};
use notez_core::util::sanitize;

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
    let dir = match &in_arg {
        None => resolve::quick_notes(scope, config)?,
        Some(target) => {
            let root = in_root(in_local, scope, config)?;
            if target.is_empty() {
                pick_directory(&root, config)?
            } else {
                resolve_target_dir(&root, target)?
            }
        }
    };
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create note dir {}", dir.display()))?;
    if scope == Scope::Local {
        project::ensure_scratch_gitignored(&dir);
    }

    let path = dir.join(note.filename());
    std::fs::write(&path, note.rendered())
        .with_context(|| format!("failed to write note {}", path.display()))?;

    Ok(path)
}

/// The root `--in` resolves under: global by default, the current scope's
/// root with `--in-local` (mirrors legacy, where "local" meant the project
/// store; the scope flags pick which one).
fn in_root(in_local: bool, scope: Scope, config: &Config) -> Result<PathBuf> {
    let root_scope = if in_local { scope } else { Scope::Global };
    resolve::root(root_scope, config)
}

/// Resolve `--in <target>` to a directory under `root`: an existing subdir
/// as-is, else a case-insensitive substring match on the first path segment
/// (legacy fuzzy matching), else the sanitized target is created fresh.
fn resolve_target_dir(root: &Path, target: &str) -> Result<PathBuf> {
    let joined = root.join(target);
    if joined.is_dir() {
        return Ok(joined);
    }

    let (head, rest) = match target.split_once('/') {
        Some((h, r)) => (h, Some(r)),
        None => (target, None),
    };
    let needle = head.to_lowercase();
    let matched = subdirs(root)?
        .into_iter()
        .find(|name| name.to_lowercase().contains(&needle));

    let base = match matched {
        Some(name) => root.join(name),
        None => {
            let cleaned = sanitize::name(head);
            if cleaned.is_empty() {
                bail!("invalid --in directory name: {target:?}");
            }
            root.join(cleaned)
        }
    };
    Ok(match rest {
        Some(rest) if !rest.is_empty() => base.join(rest),
        _ => base,
    })
}

/// Bare `--in`: fzf over the existing subdirectories of `root`.
fn pick_directory(root: &Path, config: &Config) -> Result<PathBuf> {
    let names = subdirs(root)?;
    if names.is_empty() {
        bail!(
            "no subdirectories under {} to pick from; create one with `notez mkdir <name>`",
            root.display()
        );
    }
    if !config.tools.fzf {
        bail!("bare --in needs fzf for the directory picker; pass a name instead: --in <dir>");
    }

    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new("fzf")
        .args(["--prompt", "directory> "])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to launch fzf")?;
    child
        .stdin
        .as_mut()
        .expect("stdin was piped")
        .write_all(names.join("\n").as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!("directory picker cancelled");
    }
    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        bail!("directory picker cancelled");
    }
    Ok(root.join(selected))
}

/// Immediate visible subdirectories of `root`, sorted.
fn subdirs(root: &Path) -> Result<Vec<String>> {
    let mut names: Vec<String> = match std::fs::read_dir(root) {
        Ok(entries) => entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| !n.starts_with('.'))
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    Ok(names)
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

    #[test]
    fn in_arg_uses_existing_subdir() {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("research")).unwrap();
        let dir = resolve_target_dir(root.path(), "research").unwrap();
        assert_eq!(dir, root.path().join("research"));
    }

    #[test]
    fn in_arg_substring_matches_existing_subdir() {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("jobbansokningar")).unwrap();
        let dir = resolve_target_dir(root.path(), "jobb").unwrap();
        assert_eq!(dir, root.path().join("jobbansokningar"));
    }

    #[test]
    fn in_arg_creates_missing_subdir_sanitized() {
        let root = tempdir().unwrap();
        let dir = resolve_target_dir(root.path(), "New Ideas").unwrap();
        assert_eq!(dir, root.path().join("new-ideas"));
    }

    #[test]
    fn in_arg_resolves_nested_remainder_under_match() {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("reference")).unwrap();
        let dir = resolve_target_dir(root.path(), "ref/Kotlin").unwrap();
        assert_eq!(dir, root.path().join("reference").join("Kotlin"));
    }

    #[test]
    fn add_with_in_writes_into_subdir() {
        let root = tempdir().unwrap();
        let config = config_in(root.path());
        std::fs::create_dir(root.path().join("ideas")).unwrap();

        let path = run(
            vec!["spark".into()],
            Some("ideas".into()),
            false,
            Scope::Global,
            &config,
        )
        .unwrap();
        assert_eq!(path.parent().unwrap(), root.path().join("ideas"));
    }

    #[test]
    #[serial_test::serial]
    fn add_local_gitignores_scratch_store_in_git_repo() {
        let cwd_holder = tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(cwd_holder.path())
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(cwd_holder.path()).unwrap();

        let result = run(
            vec!["scratch".into()],
            None,
            false,
            Scope::Local,
            &Config::defaults(),
        );

        std::env::set_current_dir(saved).unwrap();
        result.unwrap();
        let gitignore =
            std::fs::read_to_string(cwd_holder.path().join(".gitignore")).unwrap();
        assert!(gitignore.lines().any(|l| l.trim() == ".notez"));
    }
}
