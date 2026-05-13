//! `notez attach`: register a project on this machine.
//!
//! Replaces notez-cli's automatic `ensure_home_project_dir` + symlink
//! creation. The registration is explicit and stored per-machine; no
//! filesystem state is shared between machines.

use std::path::PathBuf;

use anyhow::Result;

use crate::config::ProjectRegistry;
use crate::core::Project;

/// Result of an attach operation.
pub struct AttachResult {
    pub name: String,
    pub local_path: PathBuf,
    pub already_existed: bool,
}

pub fn run(name: Option<String>, path: Option<String>) -> Result<AttachResult> {
    run_with_registry(name, path, &mut ProjectRegistry::load()?, save_default)
}

fn save_default(reg: &ProjectRegistry) -> Result<()> {
    reg.save()
}

/// Pure variant for testing: takes the registry and a save function so tests
/// can pass in a tempdir-rooted registry.
pub fn run_with_registry(
    name: Option<String>,
    path: Option<String>,
    registry: &mut ProjectRegistry,
    save: impl FnOnce(&ProjectRegistry) -> Result<()>,
) -> Result<AttachResult> {
    let local_path = match path {
        Some(p) => crate::util::tilde::expand(&p),
        None => std::env::current_dir()?,
    };

    let name = match name {
        Some(n) => crate::util::sanitize::name(&n),
        None => Project::detect_from(&local_path).name,
    };

    let already_existed = registry.projects.contains_key(&name);
    registry.attach(&name, &local_path);
    save(registry)?;

    Ok(AttachResult {
        name,
        local_path,
        already_existed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn attach_with_explicit_name_and_path() {
        let mut reg = ProjectRegistry::default();
        let dir = tempdir().unwrap();
        let project_root = dir.path().join("my-project");
        std::fs::create_dir(&project_root).unwrap();

        let saved = std::cell::Cell::new(false);
        let result = run_with_registry(
            Some("my-project".into()),
            Some(project_root.to_string_lossy().into_owned()),
            &mut reg,
            |_r| {
                saved.set(true);
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(result.name, "my-project");
        assert_eq!(result.local_path, project_root);
        assert!(!result.already_existed);
        assert!(saved.get());

        assert!(reg.projects.contains_key("my-project"));
    }

    #[test]
    fn second_attach_marks_already_existed() {
        let mut reg = ProjectRegistry::default();
        let dir = tempdir().unwrap();
        let project_root = dir.path().join("foo");
        std::fs::create_dir(&project_root).unwrap();

        run_with_registry(
            Some("foo".into()),
            Some(project_root.to_string_lossy().into_owned()),
            &mut reg,
            |_| Ok(()),
        )
        .unwrap();

        let again = run_with_registry(
            Some("foo".into()),
            Some(project_root.to_string_lossy().into_owned()),
            &mut reg,
            |_| Ok(()),
        )
        .unwrap();

        assert!(again.already_existed);
    }

    #[test]
    fn attach_without_name_derives_from_path_basename() {
        let mut reg = ProjectRegistry::default();
        let dir = tempdir().unwrap();
        let project_root = dir.path().join("cool-thing");
        std::fs::create_dir(&project_root).unwrap();

        let result = run_with_registry(
            None,
            Some(project_root.to_string_lossy().into_owned()),
            &mut reg,
            |_| Ok(()),
        )
        .unwrap();

        assert_eq!(result.name, "cool-thing");
    }
}
