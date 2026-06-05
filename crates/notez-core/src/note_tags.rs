//! Per-root `.tags` files: persist note importance flags keyed by path
//! relative to the notes root. Format is one `relpath:flagbyte` line each,
//! identical to notez-cli's tree browser so the two stay interoperable.

use std::collections::HashMap;
use std::path::Path;

/// Load `<root>/.tags` into a `relpath -> flags` map. Missing file → empty.
pub fn load_tags(root: &Path) -> HashMap<String, u8> {
    let path = root.join(".tags");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    content
        .lines()
        .filter_map(|line| {
            let (name, flags) = line.split_once(':')?;
            Some((name.to_string(), flags.trim().parse::<u8>().ok()?))
        })
        .collect()
}

/// Write `<root>/.tags`, dropping zero-flag entries, sorted by key.
pub fn save_tags(root: &Path, tags: &HashMap<String, u8>) -> std::io::Result<()> {
    let mut entries: Vec<_> = tags.iter().filter(|(_, &f)| f != 0).collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::new();
    for (name, flags) in entries {
        out.push_str(&format!("{name}:{flags}\n"));
    }
    std::fs::write(root.join(".tags"), out)
}

/// Relative key for `abs_path` under `root`, or `None` if it isn't under it.
fn rel_key(root: &Path, abs_path: &Path) -> Option<String> {
    abs_path
        .strip_prefix(root)
        .ok()
        .map(|r| r.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
}

/// Flags for a single note at `abs_path` within `root`.
pub fn get(root: &Path, abs_path: &Path) -> u8 {
    match rel_key(root, abs_path) {
        Some(key) => load_tags(root).get(&key).copied().unwrap_or(0),
        None => 0,
    }
}

/// Set flags for a single note; a zero clears its entry.
pub fn set(root: &Path, abs_path: &Path, flags: u8) -> std::io::Result<()> {
    let Some(key) = rel_key(root, abs_path) else {
        return Ok(());
    };
    let mut tags = load_tags(root);
    if flags == 0 {
        tags.remove(&key);
    } else {
        tags.insert(key, flags);
    }
    save_tags(root, &tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn set_get_round_trip() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let note = root.join("sub/2026-01-01-x.md");
        set(root, &note, 0b101).unwrap();
        assert_eq!(get(root, &note), 0b101);
    }

    #[test]
    fn zero_clears_entry() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let note = root.join("a.md");
        set(root, &note, 4).unwrap();
        set(root, &note, 0).unwrap();
        assert_eq!(get(root, &note), 0);
        // File written without the cleared key.
        assert!(!load_tags(root).contains_key("a.md"));
    }

    #[test]
    fn format_matches_relpath_colon_byte() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        set(root, &root.join("03_proj"), 4).unwrap();
        let raw = std::fs::read_to_string(root.join(".tags")).unwrap();
        assert_eq!(raw, "03_proj:4\n");
    }
}
