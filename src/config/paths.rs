//! Standard config-file locations.
//!
//! Per-machine files live under `$XDG_CONFIG_HOME/notez/` (fallback
//! `~/.config/notez/`). The synced metadata file lives under the global
//! notez root.

use std::path::PathBuf;

/// Directory for per-machine config files.
pub fn config_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("notez");
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".config")
        .join("notez")
}

/// Per-machine config file path.
pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

/// Per-machine project registry path.
pub fn registry_file() -> PathBuf {
    config_dir().join("registry.toml")
}

/// Synced metadata path under the given notez root.
pub fn metadata_file(notez_root: &std::path::Path) -> PathBuf {
    notez_root.join(".notez-config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_falls_back_to_home_config() {
        // Clear XDG so we exercise the fallback.
        let saved = std::env::var("XDG_CONFIG_HOME").ok();
        // SAFETY: tests run sequentially in this module; no other thread
        // looks at this var here.
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }

        let dir = config_dir();
        assert!(dir.ends_with(".config/notez"), "got {:?}", dir);

        // Restore.
        if let Some(v) = saved {
            unsafe {
                std::env::set_var("XDG_CONFIG_HOME", v);
            }
        }
    }

    #[test]
    fn config_dir_respects_xdg_config_home() {
        let saved = std::env::var("XDG_CONFIG_HOME").ok();
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", "/tmp/xdg-test");
        }

        let dir = config_dir();
        assert_eq!(dir, PathBuf::from("/tmp/xdg-test/notez"));

        if let Some(v) = saved {
            unsafe {
                std::env::set_var("XDG_CONFIG_HOME", v);
            }
        } else {
            unsafe {
                std::env::remove_var("XDG_CONFIG_HOME");
            }
        }
    }
}
