//! `notez todo` / `todoz`: todo manager.
//!
//! Quick-add works; the interactive TUI is still a placeholder. The current
//! notez-cli todo TUI is the most feature-rich part of the project (subtasks,
//! tags, drag-to-reorder, code TODO scanning); it ports to notez2 in a
//! follow-up. The TUI logic itself moves with minor changes; only the
//! file-source layer changes (registry-based instead of symlinks).

use anyhow::{Context, Result, bail};

use notez_core::config::Config;
use notez_core::core::{Scope, resolve};
use notez_core::util::tilde;

pub fn run(item: Option<String>, scope: Scope, config: &Config) -> Result<()> {
    match item {
        Some(text) => quick_add(text, scope, config),
        None => bail!(
            "todo TUI not yet implemented in notez2; use the epoz desktop app, \
             or `todoz \"item\"` to quick-add"
        ),
    }
}

/// Append `- [ ] <text>` to the scope's `TODO.md`, creating it (with a
/// `# TODO` header) if missing.
fn quick_add(text: String, scope: Scope, config: &Config) -> Result<()> {
    let dir = resolve::root(scope, config)?;
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;

    let path = dir.join("TODO.md");
    let mut content =
        std::fs::read_to_string(&path).unwrap_or_else(|_| "# TODO\n".to_string());
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("- [ ] ");
    content.push_str(&text);
    content.push('\n');
    std::fs::write(&path, content)
        .with_context(|| format!("failed to write {}", path.display()))?;

    println!("Added to {}", tilde::contract(&path));
    Ok(())
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
    fn quick_add_creates_todo_with_header() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());

        quick_add("first item".into(), Scope::Global, &config).unwrap();

        let content = std::fs::read_to_string(dir.path().join("TODO.md")).unwrap();
        assert_eq!(content, "# TODO\n- [ ] first item\n");
    }

    #[test]
    fn quick_add_appends_without_clobbering() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());
        std::fs::write(dir.path().join("TODO.md"), "# TODO\n- [x] done #prio").unwrap();

        quick_add("next".into(), Scope::Global, &config).unwrap();

        let content = std::fs::read_to_string(dir.path().join("TODO.md")).unwrap();
        assert_eq!(content, "# TODO\n- [x] done #prio\n- [ ] next\n");
    }

    #[test]
    fn tui_without_item_still_bails() {
        let dir = tempdir().unwrap();
        let config = config_in(dir.path());
        assert!(run(None, Scope::Global, &config).is_err());
    }
}
