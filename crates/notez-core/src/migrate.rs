//! Migration from the legacy notez-cli layout (numbered dirs + symlinks +
//! `~/.config/notez/projects`) to notez2's scope model.
//!
//! Strategy: a numbered dir `NN_<name>` whose `<name>` matches a legacy
//! project becomes that project's Personal notes — moved to
//! `<notez_root>/personal/<name>/` — and the project is attached to the
//! per-machine registry. Global dirs (quick-notes, daily-logs, _todos) and
//! unknown dirs are left untouched. Nothing is deleted; moves are renames
//! within the git-backed notez root, so they are reversible.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;

use crate::config::{paths, Config, ProjectRegistry};
use crate::util::tilde;

/// One planned migration step (preview is a `Vec<PlanItem>`).
#[derive(Serialize, Clone)]
pub struct PlanItem {
    pub name: String,
    /// Project repo path on this machine (tilde-contracted).
    pub repo_path: String,
    /// Source numbered dir (tilde-contracted).
    pub from: String,
    /// Destination personal dir (tilde-contracted).
    pub to: String,
    /// Human note, e.g. "ready" or "destination exists — will skip".
    pub note: String,
}

fn legacy_projects_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".config/notez/projects")
}

/// Parse the legacy `name=path` projects file.
pub fn read_legacy_projects() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Ok(content) = std::fs::read_to_string(legacy_projects_file()) {
        for line in content.lines() {
            if let Some((name, path)) = line.split_once('=') {
                let (name, path) = (name.trim(), path.trim());
                if !name.is_empty() && !path.is_empty() {
                    map.insert(name.to_string(), path.to_string());
                }
            }
        }
    }
    map
}

/// Strip a leading `NN_` numeric prefix: `03_notez-cli` → `notez-cli`.
fn strip_num_prefix(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() > 3 && b[0].is_ascii_digit() && b[1].is_ascii_digit() && b[2] == b'_' {
        &s[3..]
    } else {
        s
    }
}

fn is_special(dirname: &str) -> bool {
    let name = strip_num_prefix(dirname);
    matches!(dirname, "_todos" | "personal" | ".git")
        || name.starts_with("quick-note")
        || name.starts_with("daily-log")
}

/// Compute the migration plan without changing anything.
pub fn plan(config: &Config) -> Vec<PlanItem> {
    let root = config.notez_root_path();
    let personal_root = root.join("personal");
    let projects = read_legacy_projects();
    let mut out = Vec::new();

    let Ok(entries) = std::fs::read_dir(&root) else {
        return out;
    };
    let mut dirs: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();
    dirs.sort_by_key(|e| e.file_name());

    for entry in dirs {
        let dirname = entry.file_name().to_string_lossy().to_string();
        if is_special(&dirname) {
            continue;
        }
        let name = strip_num_prefix(&dirname);
        let Some(repo) = projects.get(name) else {
            continue;
        };
        let dest = personal_root.join(name);
        out.push(PlanItem {
            name: name.to_string(),
            repo_path: repo.clone(),
            from: tilde::contract(&entry.path()),
            to: tilde::contract(&dest),
            note: if dest.exists() {
                "destination exists — will skip".to_string()
            } else {
                "ready".to_string()
            },
        });
    }
    out
}

/// Apply the plan: attach each project and move its numbered dir into
/// `personal/<name>/`. Returns a per-step log. Steps whose destination
/// already exists are skipped (never overwritten).
pub fn apply(config: &Config) -> std::io::Result<Vec<String>> {
    let root = config.notez_root_path();
    std::fs::create_dir_all(root.join("personal"))?;
    let mut reg = ProjectRegistry::load().unwrap_or_default();
    let mut log = Vec::new();

    for item in plan(config) {
        reg.attach(&item.name, &tilde::expand(&item.repo_path));
        let from = tilde::expand(&item.from);
        let to = tilde::expand(&item.to);
        if to.exists() {
            log.push(format!("skipped {} (destination exists)", item.name));
            continue;
        }
        std::fs::rename(&from, &to)?;
        log.push(format!("moved {} → personal/{}", item.name, item.name));
    }

    let _ = reg.save_to(&paths::registry_file());
    log.push("attached migrated projects to the registry".to_string());
    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_numeric_prefix() {
        assert_eq!(strip_num_prefix("03_notez-cli"), "notez-cli");
        assert_eq!(strip_num_prefix("plain"), "plain");
        assert_eq!(strip_num_prefix("9_x"), "9_x"); // single digit isn't NN_
    }

    #[test]
    fn specials_are_skipped() {
        assert!(is_special("00_quick-notes"));
        assert!(is_special("01_daily-logs"));
        assert!(is_special("_todos"));
        assert!(is_special("personal"));
        assert!(!is_special("03_notez-cli"));
    }
}
