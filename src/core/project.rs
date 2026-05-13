//! Project identity and detection.
//!
//! A `Project` is a named handle to a directory on the user's machine. The
//! name is derived from the git repository toplevel (or the directory name
//! if the path is not a git repo) and sanitized to a filesystem slug.

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
    /// Detect the project that owns `dir`.
    ///
    /// Order of resolution:
    ///
    /// 1. `git rev-parse --show-toplevel` to find the repo root. If it
    ///    succeeds, the project root is the git toplevel and the name is the
    ///    sanitized basename of that toplevel.
    /// 2. Fall back to the basename of `dir` itself.
    /// 3. If `dir` has no basename (root path), the name is `"unnamed"`.
    pub fn detect_from(dir: &Path) -> Self {
        if let Some(toplevel) = git_toplevel(dir) {
            if let Some(name) = toplevel
                .file_name()
                .map(|n| sanitize::name(&n.to_string_lossy()))
            {
                if !name.is_empty() {
                    return Self {
                        name,
                        root: toplevel,
                    };
                }
            }
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

    /// Detect from the current working directory.
    pub fn detect() -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()?;
        Ok(Self::detect_from(&cwd))
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
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(dir.path())
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()
            .unwrap();

        let nested = dir.path().join("src").join("commands");
        std::fs::create_dir_all(&nested).unwrap();

        let p = Project::detect_from(&nested);
        // The project root is the git toplevel, not the nested dir.
        // Compare canonical paths because macOS prefixes tempdirs with /private.
        assert_eq!(p.root.canonicalize().unwrap(), dir.path().canonicalize().unwrap());
        // Name is the toplevel's basename, sanitized.
        let expected_name = sanitize::name(
            &dir.path().file_name().unwrap().to_string_lossy(),
        );
        assert_eq!(p.name, expected_name);
    }
}
