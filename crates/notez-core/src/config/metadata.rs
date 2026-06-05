//! Synced project metadata.
//!
//! Lives at `<notez_root>/.notez-config.toml` and syncs with `~/notez/` via
//! git. Holds display names, tags and ordering that should be the same on
//! every machine. The per-machine paths live in [`super::registry`]
//! separately.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// The full synced metadata document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotezMetadata {
    #[serde(default)]
    pub projects: BTreeMap<String, ProjectMetadata>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Human-readable display name. If absent, the project's sanitized name
    /// is used in the UI.
    #[serde(default)]
    pub display_name: Option<String>,

    /// Free-form tags for grouping in the UI.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Sort order in the UI. Lower numbers come first.
    #[serde(default)]
    pub order: Option<u32>,
}

impl NotezMetadata {
    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let body = std::fs::read_to_string(path)?;
        let m: Self = toml::from_str(&body)?;
        Ok(m)
    }

    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let body = toml::to_string_pretty(self)?;
        std::fs::write(path, body)?;
        Ok(())
    }

    /// Get the display name for a project, falling back to the sanitized name.
    pub fn display_for<'a>(&'a self, project_name: &'a str) -> &'a str {
        self.projects
            .get(project_name)
            .and_then(|m| m.display_name.as_deref())
            .unwrap_or(project_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn empty_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("meta.toml");

        let m = NotezMetadata::default();
        m.save_to(&path).unwrap();
        let back = NotezMetadata::load_from(&path).unwrap();
        assert!(back.projects.is_empty());
    }

    #[test]
    fn project_metadata_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("meta.toml");

        let mut m = NotezMetadata::default();
        m.projects.insert(
            "app2".to_string(),
            ProjectMetadata {
                display_name: Some("App2 (Android)".to_string()),
                tags: vec!["sigma".to_string(), "lia".to_string()],
                order: Some(2),
            },
        );
        m.save_to(&path).unwrap();

        let back = NotezMetadata::load_from(&path).unwrap();
        let app2 = back.projects.get("app2").unwrap();
        assert_eq!(app2.display_name.as_deref(), Some("App2 (Android)"));
        assert_eq!(app2.tags, vec!["sigma", "lia"]);
        assert_eq!(app2.order, Some(2));
    }

    #[test]
    fn display_for_falls_back_to_key() {
        let m = NotezMetadata::default();
        assert_eq!(m.display_for("unknown"), "unknown");
    }

    #[test]
    fn display_for_uses_metadata_when_present() {
        let mut m = NotezMetadata::default();
        m.projects.insert(
            "app2".to_string(),
            ProjectMetadata {
                display_name: Some("App2".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(m.display_for("app2"), "App2");
    }
}
