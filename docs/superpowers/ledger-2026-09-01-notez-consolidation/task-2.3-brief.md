### Task 2.3: Implement nav

The legacy `nav` picked over `~/notez/`'s numbered directories. The new model has no numbered dirs; it has the vault root's subdirectories plus `personal/<project>/`. Expanding `personal` one level keeps every project reachable in one hop instead of two.

**Files:**
- Modify: `crates/notez-cli/src/commands/nav.rs` (replaces the stub wholesale)
- Test: `crates/notez-cli/src/commands/nav.rs` (inline `mod tests`)

**Interfaces:**
- Consumes: `config.notez_root_path() -> PathBuf`; `picker::pick(prompt: &str, lines: &[String], use_fzf: bool) -> Result<usize>`; `logz::open_dir(&Path, &Config) -> Result<()>` from Task 2.2; `config.tools.fzf: bool`.
- Produces: `nav::candidates(root: &Path) -> Vec<PathBuf>` (pure, sorted) and `nav::run(&Config) -> Result<()>`.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn candidates_lists_vault_dirs_and_expands_personal() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        for p in [
            "00_quick-notes",
            "01_daily-logs",
            "personal/alpha",
            "personal/beta",
            ".git/objects",
        ] {
            std::fs::create_dir_all(root.join(p)).unwrap();
        }
        std::fs::write(root.join("TODO.md"), "x").unwrap();

        let got = candidates(root);

        assert_eq!(
            got,
            vec![
                root.join("00_quick-notes"),
                root.join("01_daily-logs"),
                root.join("personal").join("alpha"),
                root.join("personal").join("beta"),
            ],
            "hidden dirs and plain files are excluded, personal is expanded"
        );
    }

    #[test]
    fn candidates_on_an_empty_root_is_empty() {
        let dir = tempdir().unwrap();
        assert!(candidates(dir.path()).is_empty());
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

```bash
cd ~/Repos/notez && cargo test -p notez-cli nav
```

Expected: compile error, `cannot find function candidates in this scope`.

- [ ] **Step 3: Declare the picker module**

`crates/notez-cli/src/commands/picker.rs` exists on disk but is **not declared
in `commands/mod.rs`**, so it has never been compiled and its tests have never
run. Both this task and Task 2.4 depend on it. Add to
`crates/notez-cli/src/commands/mod.rs`, in alphabetical position:

```rust
pub mod picker;
```

Then confirm it compiles and its three dormant tests pass:

```bash
cd ~/Repos/notez && cargo test -p notez-cli picker
```

Expected: `parses_leading_index`, `single_candidate_skips_the_picker` and
`empty_candidates_bail` all PASS. If any fail, fix `picker.rs` before
continuing; it has never been exercised.

- [ ] **Step 4: Implement**

Replace the entire contents of `crates/notez-cli/src/commands/nav.rs` above the test module:

```rust
//! `notez nav` (and `notez -n`): pick a directory in the vault, then open it.
//!
//! The legacy version picked over numbered directories. The scope model has
//! none, so the candidates are the vault root's own subdirectories, with
//! `personal/` expanded one level so every project is one hop away.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use notez_core::config::Config;

use crate::commands::logz;
use crate::commands::picker;

/// Directories worth navigating to, sorted, relative order stable.
///
/// Hidden directories and plain files are excluded. `personal/` is replaced
/// by its children, because `personal` itself holds nothing directly.
pub fn candidates(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return out;
    };

    let mut tops: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter(|p| !is_hidden(p))
        .collect();
    tops.sort();

    for top in tops {
        if top.file_name().is_some_and(|n| n == "personal") {
            let Ok(children) = std::fs::read_dir(&top) else {
                continue;
            };
            let mut subs: Vec<PathBuf> = children
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .filter(|p| !is_hidden(p))
                .collect();
            subs.sort();
            out.extend(subs);
        } else {
            out.push(top);
        }
    }
    out
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with('.'))
}

pub fn run(config: &Config) -> Result<()> {
    let root = config.notez_root_path();
    if !root.is_dir() {
        bail!(
            "notez root does not exist: {}. Run `notez setup` first.",
            root.display()
        );
    }

    let dirs = candidates(&root);
    if dirs.is_empty() {
        bail!(
            "no directories in {}. Create one with `notez -g mkdir <name>`.",
            root.display()
        );
    }

    let labels: Vec<String> = dirs
        .iter()
        .map(|p| {
            p.strip_prefix(&root)
                .unwrap_or(p)
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    let index = picker::pick("nav> ", &labels, config.tools.fzf)?;
    logz::open_dir(&dirs[index], config)
}
```

- [ ] **Step 5: Run the test to verify it passes**

```bash
cd ~/Repos/notez && cargo test -p notez-cli nav
```

Expected: both tests PASS.

- [ ] **Step 6: Confirm the dispatch needs no change**

`main.rs` already reads `Commands::Nav => commands::nav::run(&config),` and
`nav::run` keeps that signature, so no dispatch edit is needed. Verify:

```bash
grep -n "Commands::Nav" ~/Repos/notez/crates/notez-cli/src/main.rs
```

Expected: `Commands::Nav => commands::nav::run(&config),`

- [ ] **Step 7: Commit**

```bash
git -C ~/Repos/notez add crates/notez-cli/src/commands/nav.rs crates/notez-cli/src/commands/mod.rs
git -C ~/Repos/notez commit -m "feat: implement nav vault directory picker"
```

---

