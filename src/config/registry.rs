//! Per-machine project registry.
//!
//! Maps project names to their on-disk locations on this specific machine.
//! Stored as TOML at `$XDG_CONFIG_HOME/notez/registry.toml`. Paths are
//! tilde-relative; resolution happens at runtime.
//!
//! Critical design point: this file is NEVER synced between machines. The
//! whole point is that each machine learns its own paths locally, so the
//! same project can live at different absolute paths on each machine.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::paths;
use crate::util::tilde;

/// The full registry: one entry per attached project.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectRegistry {
    /// Project entries keyed by sanitized name.
    #[serde(default)]
    pub projects: BTreeMap<String, ProjectEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    /// Tilde-relative path to the project root on this machine.
    pub local_path: String,
}

impl ProjectRegistry {
    /// Load from the standard location. Returns an empty registry if the
    /// file does not exist.
    pub fn load() -> anyhow::Result<Self> {
        Self::load_from(&paths::registry_file())
    }

    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let body = std::fs::read_to_string(path)?;
        let reg: Self = toml::from_str(&body)?;
        Ok(reg)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.save_to(&paths::registry_file())
    }

    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let body = toml::to_string_pretty(self)?;
        std::fs::write(path, body)?;
        Ok(())
    }

    /// Register a project. Path is stored tilde-relative.
    pub fn attach(&mut self, name: &str, local_path: &Path) {
        self.projects.insert(
            name.to_string(),
            ProjectEntry {
                local_path: tilde::contract(local_path),
            },
        );
    }

    /// Unregister a project.
    pub fn detach(&mut self, name: &str) -> bool {
        self.projects.remove(name).is_some()
    }

    /// Resolve the absolute path for a project, expanding the stored tilde
    /// against the current user's home dir. Returns `None` if the project
    /// is not registered.
    pub fn resolve(&self, name: &str) -> Option<PathBuf> {
        self.projects.get(name).map(|e| tilde::expand(&e.local_path))
    }

    /// Iterate over (name, absolute path) pairs.
    pub fn iter_resolved(&self) -> impl Iterator<Item = (&str, PathBuf)> + '_ {
        self.projects
            .iter()
            .map(|(name, e)| (name.as_str(), tilde::expand(&e.local_path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn empty_registry_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("registry.toml");

        let reg = ProjectRegistry::default();
        reg.save_to(&path).unwrap();
        let back = ProjectRegistry::load_from(&path).unwrap();
        assert!(back.projects.is_empty());
    }

    #[test]
    fn attach_stores_tilde_relative() {
        let mut reg = ProjectRegistry::default();
        let home = dirs::home_dir().unwrap();
        let project_root = home.join("repos").join("foo");
        reg.attach("foo", &project_root);

        let entry = reg.projects.get("foo").unwrap();
        assert_eq!(entry.local_path, "~/repos/foo");
    }

    #[test]
    fn resolve_expands_tilde() {
        let mut reg = ProjectRegistry::default();
        let home = dirs::home_dir().unwrap();
        reg.attach("foo", &home.join("Repos").join("foo"));

        let resolved = reg.resolve("foo").unwrap();
        assert_eq!(resolved, home.join("Repos").join("foo"));
    }

    #[test]
    fn detach_removes() {
        let mut reg = ProjectRegistry::default();
        let home = dirs::home_dir().unwrap();
        reg.attach("foo", &home.join("foo"));
        assert!(reg.detach("foo"));
        assert!(!reg.detach("foo")); // second call returns false
        assert!(reg.projects.is_empty());
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let dir = tempdir().unwrap();
        let reg = ProjectRegistry::load_from(&dir.path().join("nope.toml")).unwrap();
        assert!(reg.projects.is_empty());
    }

    #[test]
    fn round_trip_preserves_paths() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("registry.toml");

        let mut reg = ProjectRegistry::default();
        let home = dirs::home_dir().unwrap();
        reg.attach("foo", &home.join("repos/foo"));
        reg.attach("bar", &home.join("Repos/bar"));
        reg.save_to(&path).unwrap();

        let back = ProjectRegistry::load_from(&path).unwrap();
        assert_eq!(back.projects.len(), 2);
        assert_eq!(back.projects["foo"].local_path, "~/repos/foo");
        assert_eq!(back.projects["bar"].local_path, "~/Repos/bar");
    }

    #[test]
    fn resolves_paths_using_current_home() {
        // Demonstrates the cross-machine portability: same tilde-relative
        // path, different home dirs across machines, still resolves correctly.
        let mut reg = ProjectRegistry::default();
        // Force a stored value that does not depend on this machine's home.
        reg.projects.insert(
            "foo".to_string(),
            ProjectEntry {
                local_path: "~/repos/foo".to_string(),
            },
        );

        let resolved = reg.resolve("foo").unwrap();
        let expected = dirs::home_dir().unwrap().join("repos").join("foo");
        assert_eq!(resolved, expected);
    }
}
