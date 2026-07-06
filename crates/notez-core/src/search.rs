//! Full-text search across every note source.
//!
//! Rides on [`crate::core::aggregate`]: the same walk that feeds the tree
//! browser and the desktop note list also defines the search universe, so
//! search can never disagree with what the UIs show. Matching is a plain
//! case-insensitive substring scan; at notez data sizes (a few MB of
//! markdown) that is milliseconds, no index needed.

use anyhow::Result;

use crate::config::{Config, NotezMetadata, ProjectRegistry};
use crate::core::aggregate::{collect_all, NoteEntry};

/// One file that matched, anchored at its first matching line.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchHit {
    pub entry: NoteEntry,
    /// 1-based line number of the first match.
    pub line: usize,
    /// The first matching line, trimmed.
    pub snippet: String,
    /// Total matching lines in the file.
    pub match_count: usize,
    /// True when the filename itself also matches the query.
    pub name_match: bool,
}

/// Search every note and `TODO.md` reachable from the registry + global root.
/// Returns one hit per file, in aggregation order (projects, then global).
pub fn search_notes(
    query: &str,
    config: &Config,
    registry: &ProjectRegistry,
) -> Result<Vec<SearchHit>> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let needle = query.to_lowercase();
    let entries = collect_all(config, registry, &NotezMetadata::default())?;

    let mut hits = Vec::new();
    for entry in entries {
        let name_match = entry.name.to_lowercase().contains(&needle);
        let Ok(content) = std::fs::read_to_string(&entry.path) else {
            // Unreadable/binary-ish file: fall back to a name-only hit.
            if name_match {
                hits.push(name_only_hit(entry));
            }
            continue;
        };

        let mut first: Option<(usize, String)> = None;
        let mut count = 0usize;
        for (i, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&needle) {
                count += 1;
                if first.is_none() {
                    first = Some((i + 1, line.trim().to_string()));
                }
            }
        }

        match first {
            Some((line, snippet)) => hits.push(SearchHit {
                entry,
                line,
                snippet,
                match_count: count,
                name_match,
            }),
            None if name_match => hits.push(name_only_hit(entry)),
            None => {}
        }
    }
    Ok(hits)
}

fn name_only_hit(entry: NoteEntry) -> SearchHit {
    SearchHit {
        entry,
        line: 1,
        snippet: String::new(),
        match_count: 0,
        name_match: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;

    /// Build an isolated notez root + registry with one project.
    fn fixture() -> (tempfile::TempDir, Config) {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();
        unsafe { std::env::set_var("XDG_CONFIG_HOME", base.join("xdg")) };

        let root = base.join("notezroot");
        fs::create_dir_all(root.join("00_quick-notes")).unwrap();
        fs::write(
            root.join("00_quick-notes/hardware.md"),
            "# Hardware\n\nBuy an nRF5340 Audio DK.\nAlso a Pixel 8a.\n",
        )
        .unwrap();
        fs::write(root.join("TODO.md"), "# TODO\n\n- [ ] order the DK\n").unwrap();

        let mut config = Config::defaults();
        config.paths.notez_root = root.to_string_lossy().into_owned();
        (tmp, config)
    }

    #[test]
    #[serial]
    fn finds_content_across_files_case_insensitively() {
        let (_t, config) = fixture();
        let hits = search_notes("dk", &config, &ProjectRegistry::default()).unwrap();
        assert_eq!(hits.len(), 2);
        let by_name: Vec<_> = hits.iter().map(|h| h.entry.name.as_str()).collect();
        assert!(by_name.contains(&"hardware.md"));
        assert!(by_name.contains(&"TODO.md"));
        let hw = hits.iter().find(|h| h.entry.name == "hardware.md").unwrap();
        assert_eq!(hw.line, 3);
        assert_eq!(hw.snippet, "Buy an nRF5340 Audio DK.");
        assert_eq!(hw.match_count, 1);
    }

    #[test]
    #[serial]
    fn filename_match_without_content_match_still_hits() {
        let (_t, config) = fixture();
        let hits = search_notes("hardware", &config, &ProjectRegistry::default()).unwrap();
        // "# Hardware" matches in content AND the filename matches.
        let hw = hits.iter().find(|h| h.entry.name == "hardware.md").unwrap();
        assert!(hw.name_match);
        assert!(hw.match_count >= 1);
    }

    #[test]
    #[serial]
    fn empty_query_returns_nothing() {
        let (_t, config) = fixture();
        assert!(search_notes("  ", &config, &ProjectRegistry::default())
            .unwrap()
            .is_empty());
    }
}
