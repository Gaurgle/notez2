### Task 2.4: Implement edit

Port the legacy fuzzy-match-then-edit flow, but source the candidate notes from `aggregate::collect_in_scope` rather than re-walking the filesystem. That reuses the scope model and keeps one definition of "which notes are in scope".

**Files:**
- Modify: `crates/notez-cli/src/commands/edit.rs` (replaces the stub wholesale)
- Modify: `crates/notez-cli/src/main.rs` (pass `scope` into the call)
- Test: `crates/notez-cli/src/commands/edit.rs` (inline `mod tests`)

**Interfaces:**
- Consumes: `pub mod picker;` must already be declared in `commands/mod.rs` (Task 2.3 Step 3). `aggregate::collect_in_scope(scope: Scope, config: &Config, cwd_project: Option<&Project>) -> Vec<NoteEntry>` where `NoteEntry { path: PathBuf, name: String, scope: Scope, project: Option<String>, kind: SourceKind }`; `Project::try_detect() -> Option<Project>`; `picker::pick`; `config.editor.command`.
- Produces: `edit::filter_by_term(entries: &[NoteEntry], term: &str) -> Vec<NoteEntry>` (pure) and `edit::run(term: Option<String>, scope: Scope, &Config) -> Result<()>`.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use notez_core::core::aggregate::SourceKind;
    use std::path::PathBuf;

    fn entry(name: &str) -> NoteEntry {
        NoteEntry {
            path: PathBuf::from("/vault").join(name),
            name: name.to_string(),
            scope: Scope::Global,
            project: None,
            kind: SourceKind::Note,
        }
    }

    #[test]
    fn filter_matches_on_substring_case_insensitively() {
        let entries = vec![
            entry("2026-04-02-API-design.md"),
            entry("2026-04-02-bug-fix.md"),
            entry("2026-04-01-meeting.md"),
        ];

        let got = filter_by_term(&entries, "api");

        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "2026-04-02-API-design.md");
    }

    #[test]
    fn filter_can_match_several() {
        let entries = vec![
            entry("api-design.md"),
            entry("api-notes.md"),
            entry("meeting.md"),
        ];
        assert_eq!(filter_by_term(&entries, "api").len(), 2);
    }

    #[test]
    fn filter_with_no_match_is_empty() {
        let entries = vec![entry("meeting.md")];
        assert!(filter_by_term(&entries, "api").is_empty());
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

```bash
cd ~/Repos/notez && cargo test -p notez-cli edit
```

Expected: compile error, `cannot find function filter_by_term in this scope`.

- [ ] **Step 3: Implement**

Replace the contents of `crates/notez-cli/src/commands/edit.rs` above the test module:

```rust
//! `notez edit [term]` / `editz`: open an existing note.
//!
//! Candidates come from the scope model rather than a bespoke filesystem
//! walk, so `edit` sees exactly the notes the rest of the tool considers in
//! scope. A term that matches exactly one note skips the picker.

use std::process::Command;

use anyhow::{Context, Result, bail};

use notez_core::config::Config;
use notez_core::core::aggregate::{self, NoteEntry};
use notez_core::core::{Project, Scope};

use crate::commands::picker;

/// Notes whose filename contains `term`, case-insensitively.
pub fn filter_by_term(entries: &[NoteEntry], term: &str) -> Vec<NoteEntry> {
    let needle = term.to_lowercase();
    entries
        .iter()
        .filter(|e| e.name.to_lowercase().contains(&needle))
        .cloned()
        .collect()
}

pub fn run(term: Option<String>, scope: Scope, config: &Config) -> Result<()> {
    let project = Project::try_detect();
    let entries = aggregate::collect_in_scope(scope, config, project.as_ref());
    if entries.is_empty() {
        bail!("no notes found in {} scope", scope.label());
    }

    let candidates = match term.as_deref() {
        Some(t) => {
            let matched = filter_by_term(&entries, t);
            if matched.is_empty() {
                bail!("no notes matching \"{}\" in {} scope", t, scope.label());
            }
            matched
        }
        None => entries,
    };

    let chosen = if candidates.len() == 1 {
        candidates[0].clone()
    } else {
        let labels: Vec<String> = candidates.iter().map(|e| e.name.clone()).collect();
        let index = picker::pick("note> ", &labels, config.tools.fzf)?;
        candidates[index].clone()
    };

    Command::new(&config.editor.command)
        .arg(&chosen.path)
        .status()
        .with_context(|| format!("failed to launch {}", config.editor.command))?;
    Ok(())
}
```

- [ ] **Step 4: Update the dispatch to pass scope**

In `crates/notez-cli/src/main.rs`, the `Edit`/`Editz` arm currently calls `commands::edit::run(term, &config)`. Change it to:

```rust
        Commands::Edit { term } | Commands::Editz { term } => {
            commands::edit::run(term, scope, &config)
        }
```

- [ ] **Step 5: Run the tests to verify they pass**

```bash
cd ~/Repos/notez && cargo test -p notez-cli edit
```

Expected: all three PASS.

- [ ] **Step 6: Update the help text**

In `crates/notez-cli/src/main.rs:216`, change:

```rust
    cmd("notez edit [term]", "open an existing note (not ported yet)");
```

to:

```rust
    cmd("notez edit [term]", "open an existing note (fuzzy match)");
```

- [ ] **Step 7: Commit**

```bash
git -C ~/Repos/notez add crates/notez-cli/src/commands/edit.rs crates/notez-cli/src/main.rs
git -C ~/Repos/notez commit -m "feat: implement edit with scope-aware note picker"
```

---

