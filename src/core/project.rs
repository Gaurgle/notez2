//! Project identity and detection.
//!
//! A `Project` is a named handle to a directory on the user's machine. The
//! name is derived from the git repository toplevel (or the directory name
//! if the path is not a git repo) and sanitized to a filesystem slug.
//!
//! Two flavors of detection:
//!
//! - [`Project::detect`] always returns a project. Outside a git repo it
//!   falls back to the directory basename. Used for `attach`, where the
//!   user explicitly opts in to registering whatever they're sitting in.
//! - [`Project::try_detect`] returns `Option<Project>` and only succeeds
//!   inside a git repo. Used by commands like `add` in personal scope:
//!   if there is no git project, the command should fall back to global
//!   rather than write to a wrong-named "personal" folder.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::util::sanitize;

/// A named project rooted at a directory on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// Sanitized slug name (filesystem-safe).
    pub name: String,
    /// Absolute path to the project root on this machine.
    pub root: PathBuf,
}

impl Project {
    /// Lossy detection: always returns a project.
    ///
    /// Order of resolution:
    /// 1. `git rev-parse --show-toplevel`
    /// 2. Basename of `dir`
    /// 3. `"unnamed"` if `dir` has no basename
    pub fn detect_from(dir: &Path) -> Self {
        if let Some(p) = Self::try_detect_from(dir) {
            return p;
        }

        let name = dir
            .file_name()
            .map(|n| sanitize::name(&n.to_string_lossy()))
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "unnamed".to_string());

        Self {
            name,
            root: dir.to_path_buf(),
        }
    }

    /// Strict detection: returns `Some` only if `dir` is inside a git repo.
    pub fn try_detect_from(dir: &Path) -> Option<Self> {
        let toplevel = git_toplevel(dir)?;
        let name = toplevel
            .file_name()
            .map(|n| sanitize::name(&n.to_string_lossy()))
            .filter(|n| !n.is_empty())?;

        Some(Self {
            name,
            root: toplevel,
        })
    }

    /// Detect from the current working directory (always returns).
    pub fn detect() -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()?;
        Ok(Self::detect_from(&cwd))
    }

    /// Detect from the current working directory only if inside a git repo.
    pub fn try_detect() -> Option<Self> {
        let cwd = std::env::current_dir().ok()?;
        Self::try_detect_from(&cwd)
    }
}

fn git_toplevel(dir: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let line = String::from_utf8(output.stdout).ok()?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(trimmed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn git_init(path: &Path) {
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(path)
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()
            .unwrap();
    }

    #[test]
    fn detect_outside_git_uses_dirname() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("my-cool-project");
        std::fs::create_dir(&sub).unwrap();

        let p = Project::detect_from(&sub);
        assert_eq!(p.name, "my-cool-project");
        assert_eq!(p.root, sub);
    }

    #[test]
    fn detect_sanitizes_name() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("My Cool Project!");
        std::fs::create_dir(&sub).unwrap();

        let p = Project::detect_from(&sub);
        assert_eq!(p.name, "my-cool-project");
    }

    #[test]
    fn detect_inside_git_uses_toplevel() {
        let dir = tempdir().unwrap();
        git_init(dir.path());

        let nested = dir.path().join("src").join("commands");
        std::fs::create_dir_all(&nested).unwrap();

        let p = Project::detect_from(&nested);
        assert_eq!(p.root.canonicalize().unwrap(), dir.path().canonicalize().unwrap());
        let expected_name = sanitize::name(
            &dir.path().file_name().unwrap().to_string_lossy(),
        );
        assert_eq!(p.name, expected_name);
    }

    #[test]
    fn try_detect_outside_git_returns_none() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("not-a-repo");
        std::fs::create_dir(&sub).unwrap();

        assert!(Project::try_detect_from(&sub).is_none());
    }

    #[test]
    fn try_detect_inside_git_returns_some() {
        let dir = tempdir().unwrap();
        git_init(dir.path());

        let p = Project::try_detect_from(dir.path()).unwrap();
        let expected_name = sanitize::name(
            &dir.path().file_name().unwrap().to_string_lossy(),
        );
        assert_eq!(p.name, expected_name);
    }
}
