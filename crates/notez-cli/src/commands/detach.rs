//! `notez detach`: remove a project from this machine's registry.
//!
//! Does not touch the project's notes on disk. Reversible by `attach` again.

use anyhow::{Result, bail};

use notez_core::config::ProjectRegistry;

pub fn run(name: String) -> Result<()> {
    let mut reg = ProjectRegistry::load()?;
    if !reg.detach(&name) {
        bail!("no project named '{}' is attached on this machine", name);
    }
    reg.save()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn detach_existing_returns_true() {
        let mut reg = ProjectRegistry::default();
        let home = dirs::home_dir().unwrap();
        reg.attach("foo", &home.join("foo"));
        assert!(reg.detach("foo"));
    }

    #[test]
    fn detach_missing_returns_false() {
        let mut reg = ProjectRegistry::default();
        assert!(!reg.detach("nope"));
    }

    #[test]
    fn round_trip_via_disk() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("registry.toml");

        let mut reg = ProjectRegistry::default();
        let home = dirs::home_dir().unwrap();
        reg.attach("foo", &home.join("foo"));
        reg.save_to(&path).unwrap();

        let mut reloaded = ProjectRegistry::load_from(&path).unwrap();
        assert!(reloaded.detach("foo"));
        reloaded.save_to(&path).unwrap();

        let again = ProjectRegistry::load_from(&path).unwrap();
        assert!(again.projects.is_empty());
    }
}
