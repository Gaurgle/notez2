# notez Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Collapse notez-cli, notez2 and the leftover epoz clone into one repo named `notez`, close the three stubbed commands, fix subdirectory note resolution, and cut the installed binary over to it on both machines without losing a note.

**Architecture:** notez2's structure is already the target shape, so this is a rename plus a cutover rather than a restructure. `crates/notez-core` stays the GUI-agnostic engine; `crates/notez-cli` is the `notez` binary with argv-0 aliases; `app/` is epoz. The one real code change is in `core/resolve.rs`, which currently roots scratch and public stores at the cwd instead of the project root.

**Tech Stack:** Rust 2021, anyhow, clap 4, ratatui 0.29, serde, tempfile + serial_test for tests. External tools shelled out to: git, fzf, yazi, rg.

**Spec:** `docs/superpowers/specs/2026-09-01-notez-consolidation-design.md`

## Global Constraints

- **Never delete anything holding notes.** The vault is renamed, never removed. `notez-cli` is archived, not deleted. The only deletion anywhere in this plan is `~/Repos/epoz`, a redundant local clone.
- **No em-dashes or en-dashes** in any output: code, comments, docs, commit messages, chat. Use a hyphen, a colon, parentheses, or rewrite. This is a hard rule from the user's global CLAUDE.md.
- **Conventional Commits**, first line under 72 characters. **No `Co-Authored-By` lines, ever.**
- **Commits are presented, not run.** The user runs git themselves unless they say "ship it". Every commit step below gives the exact command to hand over.
- **Direct commits on `main`.** No PR, no branch per phase. The default branch is renamed from `master` to `main` in Task 1.2.
- **Never edit, delete or weaken a test to make code pass.** Task 2.1 changes behavior that two existing tests assert; those tests are kept and new ones added beside them, because both remain correct under the fallback.
- **`notez-core` stays GUI-agnostic.** No clap, ratatui, dialoguer or console dependencies added to it.
- **macOS ARM:** `codesign --force --sign -` must run after any `cp` over an existing Mach-O, or the kernel SIGKILLs the binary on launch.
- **`~/notes` and `Gaurgle/notes` are out of scope.** Untouched.
- **Two machines.** Machine A is this one. Steps marked **[MACHINE B]** must be run by the user on the second machine and cannot be done from here.

---

## File Structure

**Created:**
- `install.sh` (repo root): build, install, codesign, symlink the aliases.
- `crates/notez-cli/src/commands/logz.rs`: resolve and open the daily-logs directory.
- `CLAUDE.md` is explicitly NOT created here. It is noted as follow-up work in the spec.

**Modified:**
- `crates/notez-core/src/core/resolve.rs`: root `Local` and `Public` at the project root. The single behavioral fix.
- `crates/notez-cli/src/commands/nav.rs`: replace the stub with a vault picker.
- `crates/notez-cli/src/commands/edit.rs`: replace the stub with a fuzzy note picker.
- `crates/notez-cli/src/commands/mod.rs`: register `logz`.
- `crates/notez-cli/src/main.rs`: dispatch `Logz`/`Logs`, `Nav`, `Edit`; reword the `demo` stub; help header string.
- `README.md`, `DESIGN.md`, `app/README.md`: naming.
- 19 files total carry the string "notez2" (31 occurrences). Task 1.3 covers them.

---

## Phase 1: Naming

### Task 1.1: Move the vault aside and repoint both machines

This is the only step in the plan that can lose data if done out of order. Renaming `Gaurgle/notez` leaves a GitHub redirect to the new name. Creating a new `Gaurgle/notez` in Task 1.2 **drops that redirect**. A machine whose `~/notez` remote still points at the old URL would then push private notes at a public code repo.

Both machines must be repointed before Task 1.2 begins.

**Files:** none (GitHub and git remotes only)

**Interfaces:**
- Produces: `Gaurgle/notez-vault` exists; `Gaurgle/notez` is free; both machines' `~/notez` remotes point at `notez-vault`.

- [ ] **Step 1: Commit the vault's current state as a restore point**

The vault had 6 dirty entries as of 2026-09-01. Capture them before anything moves.

```bash
git -C ~/notez status --short
git -C ~/notez add -A
git -C ~/notez commit -m "chore: checkpoint before notez consolidation"
git -C ~/notez push
```

- [ ] **Step 2: Rename the vault repo on GitHub**

```bash
gh repo rename notez-vault --repo Gaurgle/notez
```

Expected: confirmation that the repo is now `Gaurgle/notez-vault`.

- [ ] **Step 3: Repoint machine A's vault remote**

```bash
git -C ~/notez remote set-url origin git@github.com:Gaurgle/notez-vault.git
git -C ~/notez remote -v
git -C ~/notez fetch
```

Expected: both lines read `notez-vault.git`, and the fetch succeeds.

- [ ] **Step 4: [MACHINE B] Repoint machine B's vault remote**

Hand these to the user to run on the second machine:

```bash
git -C ~/notez remote set-url origin git@github.com:Gaurgle/notez-vault.git
git -C ~/notez remote -v
git -C ~/notez fetch
```

- [ ] **Step 5: Confirm before proceeding**

Do not start Task 1.2 until the user confirms Step 4 is done. Ask explicitly.

---

### Task 1.2: Rename the code repo and its default branch

**Files:** none (GitHub, local directory, git refs)

**Interfaces:**
- Consumes: `Gaurgle/notez` is free (Task 1.1).
- Produces: `Gaurgle/notez` is the code repo; local checkout at `~/Repos/notez`; default branch `main`.

- [ ] **Step 1: Rename the repo on GitHub**

```bash
gh repo rename notez --repo Gaurgle/notez2
```

- [ ] **Step 2: Move the local checkout and repoint its remote**

```bash
mv ~/Repos/notez2 ~/Repos/notez
git -C ~/Repos/notez remote set-url origin https://github.com/Gaurgle/notez.git
git -C ~/Repos/notez remote -v
```

- [ ] **Step 3: Rename the default branch to main**

```bash
git -C ~/Repos/notez branch -m master main
git -C ~/Repos/notez push -u origin main
gh repo edit Gaurgle/notez --default-branch main
git -C ~/Repos/notez push origin --delete master
```

- [ ] **Step 4: Verify**

```bash
git -C ~/Repos/notez branch --show-current
git -C ~/Repos/notez fetch --prune && git -C ~/Repos/notez status -sb
```

Expected: `main`, and a clean tracking status against `origin/main`.

---

### Task 1.3: Rename "notez2" to "notez" throughout the repo

31 occurrences across 19 files. Most are mechanical. Three are naming-rationale prose that must be rewritten by hand, because they explain the very split this change removes.

**Files:**
- Modify (mechanical): `crates/notez-cli/src/main.rs:57,82,93,196,235`, `crates/notez-cli/src/cli/mod.rs`, `crates/notez-cli/src/commands/{edit,migrate,nav}.rs`, `crates/notez-core/Cargo.toml:5`, `crates/notez-core/src/{lib.rs,migrate.rs}`, `crates/notez-core/src/core/scope.rs`, `crates/notez-core/src/util/tilde.rs`, `crates/notez-core/src/config/mod.rs`, `crates/notez-core/src/todo/mod.rs`, `app/src-tauri/Cargo.toml:4`, `app/src-tauri/src/{lib.rs,commands.rs}`, `app/src/app.css:1`
- Modify (by hand): `README.md:1,5`, `app/README.md:5`, `DESIGN.md:1,272,533`

- [ ] **Step 1: Inventory before changing anything**

```bash
cd ~/Repos/notez
grep -rn "notez2" . --exclude-dir=.git --exclude-dir=node_modules | wc -l
```

Expected: 31 (plus any occurrences inside `docs/`, which are historical records in the spec and plan and must be left alone).

- [ ] **Step 2: Rewrite the three naming paragraphs by hand**

`README.md:5` currently reads:

> **Naming:** notez2 is the CLI and core (`crates/`); the desktop app in `app/` is **epoz**. Same data model, two surfaces. (The epoz name previously belonged to a standalone repo-dashboard TUI, which lives on as **fleetz**.)

Replace with:

> **Naming:** `notez` is the CLI and core (`crates/`); the desktop app in `app/` is **epoz**. Same engine, two surfaces. (The epoz name previously belonged to a standalone repo-dashboard TUI, which lives on as **fleetz**.)

`app/README.md:5` gets the same treatment: "notez2 is the CLI/core" becomes "`notez` is the CLI/core".

`DESIGN.md:533` explains the notez2-versus-epoz naming split. Rewrite it to describe one repo with two front ends on a shared core, with no "notez2" anywhere.

`DESIGN.md:272` lists "notez2" as a repo name in an example repo list. Change to "notez".

- [ ] **Step 3: Sweep the mechanical remainder**

Excludes `docs/`, which holds the spec and this plan and must keep its historical references.

```bash
cd ~/Repos/notez
grep -rl "notez2" . --exclude-dir=.git --exclude-dir=node_modules --exclude-dir=docs \
  | xargs sed -i '' 's/notez2/notez/g'
```

- [ ] **Step 4: Verify nothing outside docs survives**

```bash
cd ~/Repos/notez
grep -rn "notez2" . --exclude-dir=.git --exclude-dir=node_modules --exclude-dir=docs
```

Expected: no output.

- [ ] **Step 5: Check the sweep did not mangle prose**

`sed` will have turned phrases like "notez2's scope model" into "notez's scope model", which is correct, but read the diff for sentences that now read oddly.

```bash
git -C ~/Repos/notez diff
```

- [ ] **Step 6: Build and test**

```bash
cd ~/Repos/notez && cargo build --workspace && cargo test --workspace
```

Expected: clean build, all tests pass.

- [ ] **Step 7: Commit**

```bash
git -C ~/Repos/notez add -A
git -C ~/Repos/notez commit -m "refactor: rename notez2 to notez throughout"
```

---

### Task 1.4: Archive notez-cli behind a pointer

**Files:**
- Modify: `~/Repos/notez-cli/README.md` (replaced wholesale)

- [ ] **Step 1: Replace the README with a pointer**

```bash
cat > ~/Repos/notez-cli/README.md <<'EOF'
# notez-cli (archived)

This repository is archived. It has been superseded by
[notez](https://github.com/Gaurgle/notez), which contains the same tool
rebuilt on a shared core, plus the epoz desktop app.

Nothing here is maintained. The history is kept because the current tool
grew out of it, and because `notez migrate-from-legacy` exists to convert
the note layout this version created.

- Current CLI and core: https://github.com/Gaurgle/notez
- Multi-repo dashboard TUI (unrelated, formerly named epoz): https://github.com/Gaurgle/fleetz
EOF
```

- [ ] **Step 2: Commit and push**

```bash
git -C ~/Repos/notez-cli add README.md
git -C ~/Repos/notez-cli commit -m "docs: archive in favor of notez"
git -C ~/Repos/notez-cli push
```

- [ ] **Step 3: Archive on GitHub**

```bash
gh repo archive Gaurgle/notez-cli --yes
```

Expected: the repo is marked archived and becomes read-only.

---

### Task 1.5: Delete the stale epoz clone

`~/Repos/epoz` is a checkout of what GitHub already renamed to `fleetz` back in August. Its remote redirects. It holds no unique history and no notes.

- [ ] **Step 1: Prove it is redundant before deleting**

```bash
git -C ~/Repos/epoz status --short
git -C ~/Repos/epoz log --oneline -1
git -C ~/Repos/fleetz log --oneline --all | grep -c "$(git -C ~/Repos/epoz log -1 --format=%s)"
```

Expected: clean status, and the epoz tip commit's subject is present in fleetz history. **If the status is dirty or the commit is absent, stop and report rather than deleting.**

- [ ] **Step 2: Delete**

```bash
rm -rf ~/Repos/epoz
```

---

## Phase 2: Code

### Task 2.1: Root scratch and public stores at the project root

This is the original bug. `resolve::root` returns `current_dir()?.join(".notez")` for `Local` and `current_dir()?.join("notez")` for `Public`, so every subdirectory becomes its own note store. Every command goes through this function.

`Personal`, the default scope, is already correct: it uses `Project::try_detect()`, which is the git toplevel.

**Files:**
- Modify: `crates/notez-core/src/core/resolve.rs`
- Test: `crates/notez-core/src/core/resolve.rs` (inline `mod tests`)

**Interfaces:**
- Consumes: `Project::try_detect() -> Option<Project>` where `Project { name: String, root: PathBuf }`, from `crate::core::Project`. `root` is the git toplevel.
- Produces: `resolve::root(scope, config) -> Result<PathBuf>` unchanged in signature; `Local` and `Public` now resolve relative to the project root. `resolve::quick_notes` and `resolve::daily_logs` inherit the fix, since both call `root`.

- [ ] **Step 1: Write the failing tests**

Add to the existing `mod tests` in `crates/notez-core/src/core/resolve.rs`. Note the local binding is named `toplevel`, not `root`, to avoid shadowing the `root` function under test.

```rust
    /// Create a git repo with a nested subdirectory. Returns (repo root, subdir),
    /// both canonicalized so they compare equal to what git reports.
    fn git_repo_with_subdir() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempdir().unwrap();
        let toplevel = dir.path().canonicalize().unwrap();
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&toplevel)
            .status()
            .unwrap();
        let sub = toplevel.join("crates").join("deep");
        std::fs::create_dir_all(&sub).unwrap();
        (dir, toplevel, sub)
    }

    #[test]
    #[serial_test::serial]
    fn local_resolves_to_project_root_from_a_subdirectory() {
        let (_guard, toplevel, sub) = git_repo_with_subdir();
        let config = Config::defaults();

        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(&sub).unwrap();
        let r = root(Scope::Local, &config);
        std::env::set_current_dir(saved).unwrap();

        assert_eq!(r.unwrap(), toplevel.join(".notez"));
    }

    #[test]
    #[serial_test::serial]
    fn public_resolves_to_project_root_from_a_subdirectory() {
        let (_guard, toplevel, sub) = git_repo_with_subdir();
        let config = Config::defaults();

        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(&sub).unwrap();
        let r = root(Scope::Public, &config);
        std::env::set_current_dir(saved).unwrap();

        assert_eq!(r.unwrap(), toplevel.join("notez"));
    }

    #[test]
    #[serial_test::serial]
    fn daily_logs_follow_the_project_root_from_a_subdirectory() {
        let (_guard, toplevel, sub) = git_repo_with_subdir();
        let mut config = Config::defaults();
        config.paths.daily_logs_dir = "01_daily-logs".to_string();

        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(&sub).unwrap();
        let r = daily_logs(Scope::Local, &config);
        std::env::set_current_dir(saved).unwrap();

        assert_eq!(r.unwrap(), toplevel.join(".notez").join("01_daily-logs"));
    }
```

The two existing tests `local_uses_dot_notez_in_cwd` and `public_uses_notez_in_cwd` use a non-git tempdir and stay valid: they now assert the outside-a-repo fallback. Do not modify or delete them.

- [ ] **Step 2: Run the tests to verify they fail**

```bash
cd ~/Repos/notez && cargo test -p notez-core resolve:: -- --nocapture
```

Expected: the three new tests FAIL, asserting `<subdir>/.notez` where `<toplevel>/.notez` was expected. The two existing fallback tests PASS.

- [ ] **Step 3: Implement**

In `crates/notez-core/src/core/resolve.rs`, add the helper above `pub fn root`:

```rust
/// Root of the project the cwd belongs to.
///
/// The git toplevel when inside a repo, otherwise the cwd itself. Scratch
/// and public stores hang off this rather than off the cwd, so a note taken
/// three directories deep lands in the project's one store instead of
/// spawning a new store beside it.
fn project_root() -> Result<PathBuf> {
    match Project::try_detect() {
        Some(p) => Ok(p.root),
        None => Ok(std::env::current_dir()?),
    }
}
```

Then change the two arms in `root`:

```rust
        Scope::Local => project_root()?.join(".notez"),
        Scope::Public => project_root()?.join("notez"),
```

Update the doc comment on `root` so it stops promising the old behavior:

```rust
/// Resolve the root directory for the given scope.
///
/// - `Local`: `<project root>/.notez/` (git toplevel, else cwd)
/// - `Public`: `<project root>/notez/` (git toplevel, else cwd)
/// - `Personal`: `<notez_root>/personal/<project>/` if cwd is inside a git
///   repo; otherwise falls back to `<notez_root>/` (same as Global).
/// - `Global`: `<notez_root>/`
```

- [ ] **Step 4: Run the tests to verify they pass**

```bash
cd ~/Repos/notez && cargo test -p notez-core
```

Expected: all pass, including the two pre-existing fallback tests.

- [ ] **Step 5: Verify the whole workspace still builds and passes**

```bash
cd ~/Repos/notez && cargo test --workspace
```

- [ ] **Step 6: Commit**

```bash
git -C ~/Repos/notez add crates/notez-core/src/core/resolve.rs
git -C ~/Repos/notez commit -m "fix: root scratch and public notes at the project root"
```

---

### Task 2.2: Implement logz

Currently `main.rs:82` bails. Port `notez-cli/src/commands/browse.rs:run_logz`, resolving through the scope model so it inherits Task 2.1.

**Files:**
- Create: `crates/notez-cli/src/commands/logz.rs`
- Modify: `crates/notez-cli/src/commands/mod.rs`, `crates/notez-cli/src/main.rs:81-83`
- Test: `crates/notez-cli/src/commands/logz.rs` (inline `mod tests`)

**Interfaces:**
- Consumes: `resolve::daily_logs(scope, config) -> Result<PathBuf>`; `config.tools.yazi: bool`; `config.editor.command: String`.
- Produces: `logz::prepare(scope, &Config) -> Result<PathBuf>` (resolve and create, pure enough to test) and `logz::run(scope, &Config) -> Result<()>` (prepare, then launch). `logz::open_dir(&Path, &Config) -> Result<()>` is reused by Task 2.3.

- [ ] **Step 1: Write the failing test**

Create `crates/notez-cli/src/commands/logz.rs` containing only the test module and the function signatures it needs, so the test compiles and fails on behavior rather than on a missing file.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use notez_core::core::Scope;
    use tempfile::tempdir;

    #[test]
    #[serial_test::serial]
    fn prepare_creates_the_logs_dir_under_the_project_root() {
        let dir = tempdir().unwrap();
        let toplevel = dir.path().canonicalize().unwrap();
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&toplevel)
            .status()
            .unwrap();
        let sub = toplevel.join("src").join("nested");
        std::fs::create_dir_all(&sub).unwrap();

        let mut config = Config::defaults();
        config.paths.daily_logs_dir = "01_daily-logs".to_string();

        let saved = std::env::current_dir().unwrap();
        std::env::set_current_dir(&sub).unwrap();
        let got = prepare(Scope::Local, &config);
        std::env::set_current_dir(saved).unwrap();

        let expected = toplevel.join(".notez").join("01_daily-logs");
        let got = got.unwrap();
        assert_eq!(got, expected);
        assert!(got.is_dir(), "prepare should create the directory");
    }
}
```

Add `serial_test = "3"` to `crates/notez-cli/Cargo.toml` under `[dev-dependencies]` if not already present. It is already listed there.

- [ ] **Step 2: Run the test to verify it fails**

```bash
cd ~/Repos/notez && cargo test -p notez-cli logz
```

Expected: compile error, `cannot find function prepare in this scope`.

- [ ] **Step 3: Implement**

Put this above the test module in `crates/notez-cli/src/commands/logz.rs`:

```rust
//! `notez logz` / `logs` / `zlogs`: open the daily-logs directory.
//!
//! Resolution goes through the scope model, so from a subdirectory this
//! opens the project's logs rather than creating a new store in place.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use notez_core::config::Config;
use notez_core::core::{Scope, resolve};

/// Resolve the daily-logs directory for `scope` and create it if absent.
///
/// Creating here rather than failing keeps the first run after a fresh
/// install from erroring on a directory that simply does not exist yet.
pub fn prepare(scope: Scope, config: &Config) -> Result<PathBuf> {
    let dir = resolve::daily_logs(scope, config)?;
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(dir)
}

/// Open `dir` in yazi when available, otherwise in the configured editor.
pub fn open_dir(dir: &Path, config: &Config) -> Result<()> {
    let program = if config.tools.yazi {
        "yazi"
    } else {
        config.editor.command.as_str()
    };
    Command::new(program)
        .arg(dir)
        .status()
        .with_context(|| format!("failed to launch {}", program))?;
    Ok(())
}

pub fn run(scope: Scope, config: &Config) -> Result<()> {
    let dir = prepare(scope, config)?;
    open_dir(&dir, config)
}
```

- [ ] **Step 4: Register the module**

In `crates/notez-cli/src/commands/mod.rs`, add alongside the existing declarations:

```rust
pub mod logz;
```

- [ ] **Step 5: Wire the dispatch**

In `crates/notez-cli/src/main.rs`, replace:

```rust
        Commands::Logz | Commands::Logs => Err(anyhow::anyhow!(
            "browsing daily logs is not yet implemented in notez; coming in the next milestone"
        )),
```

with:

```rust
        Commands::Logz | Commands::Logs => commands::logz::run(scope, &config),
```

- [ ] **Step 6: Run the test to verify it passes**

```bash
cd ~/Repos/notez && cargo test -p notez-cli logz
```

Expected: PASS.

- [ ] **Step 7: Update the help text**

In `crates/notez-cli/src/main.rs:221`, change:

```rust
    cmd("notez logz / logs", "browse daily logs (not ported yet)");
```

to:

```rust
    cmd("notez logz / logs", "browse daily logs");
```

- [ ] **Step 8: Commit**

```bash
git -C ~/Repos/notez add crates/notez-cli/src/commands/logz.rs crates/notez-cli/src/commands/mod.rs crates/notez-cli/src/main.rs
git -C ~/Repos/notez commit -m "feat: implement logz daily-log browsing"
```

---

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

### Task 2.5: Reword the demo stub

`demo` stays unimplemented by decision, but its message promises a milestone that no longer exists.

**Files:**
- Modify: `crates/notez-cli/src/main.rs:92-94`

- [ ] **Step 1: Change the message**

```rust
        Commands::Demo { view: _ } => Err(anyhow::anyhow!(
            "demo is not implemented; it was a screenshot helper in the legacy CLI"
        )),
```

- [ ] **Step 2: Verify the workspace builds and all tests pass**

```bash
cd ~/Repos/notez && cargo build --workspace && cargo test --workspace
```

- [ ] **Step 3: Commit**

```bash
git -C ~/Repos/notez add crates/notez-cli/src/main.rs
git -C ~/Repos/notez commit -m "docs: drop stale milestone promise from demo stub"
```

---

## Phase 3: Install

### Task 3.1: Port install.sh with the codesign step

notez2 has no installer. The codesign step is load-bearing on macOS ARM: `cp` over an existing Mach-O invalidates the ad-hoc linker signature and the kernel SIGKILLs the binary on launch.

**Files:**
- Create: `install.sh` (repo root, mode 755)

**Interfaces:**
- Produces: `~/.local/bin/notez` plus symlinks `todoz zlog logz znote treez editz findz zlogs`.

- [ ] **Step 1: Write the installer**

```bash
cat > ~/Repos/notez/install.sh <<'EOF'
#!/usr/bin/env bash
set -eo pipefail

BOLD=$(printf '\033[1m')
GREEN=$(printf '\033[38;2;166;227;161m')
PEACH=$(printf '\033[38;2;250;179;135m')
RESET=$(printf '\033[0m')

echo ""
echo "  ${BOLD}notez${RESET} installer"
echo ""

if ! command -v cargo &>/dev/null; then
    echo "  ${PEACH}x${RESET} cargo not found. Install Rust: https://rustup.rs"
    exit 1
fi
echo "  ${GREEN}+${RESET} cargo found"

INSTALL_DIR="${1:-$HOME/.local/bin}"
echo "  ${GREEN}+${RESET} install directory: $INSTALL_DIR"

echo ""
echo "  Building notez..."
cargo build --release --quiet -p notez-cli

mkdir -p "$INSTALL_DIR"
cp target/release/notez "$INSTALL_DIR/notez"
chmod +x "$INSTALL_DIR/notez"

# Re-sign on macOS. `cp` over an existing Mach-O invalidates the ad-hoc
# linker signature, and the kernel then SIGKILLs the new binary on launch.
# Re-applying an ad-hoc signature is a no-op on Linux.
if command -v codesign &>/dev/null; then
    codesign --force --sign - "$INSTALL_DIR/notez" 2>/dev/null || true
fi

# Argv-0 dispatch: each name re-enters the same binary as a subcommand.
#   z<verb>  write and append commands
#   <noun>z  view and manage TUIs
for cmd in todoz zlog logz zlogs znote treez editz findz; do
    ln -sf "$INSTALL_DIR/notez" "$INSTALL_DIR/$cmd"
done

echo ""
echo "  ${GREEN}+${RESET} installed to $INSTALL_DIR/notez"
echo "  ${GREEN}+${RESET} aliases: todoz, zlog, logz, zlogs, znote, treez, editz, findz"

if ! echo "$PATH" | tr ':' '\n' | grep -q "^$INSTALL_DIR$"; then
    echo ""
    echo "  ${PEACH}!${RESET} $INSTALL_DIR is not in your PATH"
    echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
fi

echo ""
echo "  Tab completions (zsh):"
echo "    mkdir -p ~/.zfunc"
echo "    notez completions zsh > ~/.zfunc/_notez"
echo "    # in .zshrc: fpath=(~/.zfunc \$fpath)"
echo ""
echo "  Run ${BOLD}notez setup${RESET} to get started."
echo ""
EOF
chmod +x ~/Repos/notez/install.sh
```

- [ ] **Step 2: Run it**

```bash
cd ~/Repos/notez && ./install.sh
```

Expected: builds, installs, prints the alias list.

- [ ] **Step 3: Verify the binary is the new one and is not killed**

```bash
notez --help | head -3
notez --version
ls -l ~/.local/bin/todoz
```

Expected: the help header reads `notez`, the process does not die with "killed", and `todoz` is a symlink to `notez`. A "zsh: killed" here means the codesign step did not take.

- [ ] **Step 4: Verify the original bug is fixed end to end**

```bash
cd ~/Repos/notez/crates/notez-core/src && notez -l add subdir-check "written from a subdirectory"
ls ~/Repos/notez/.notez/
find ~/Repos/notez/crates -name ".notez" -maxdepth 4
```

Expected: the note lands under `~/Repos/notez/.notez/`, and `find` returns nothing, proving no store was created in the subdirectory.

- [ ] **Step 5: Commit**

```bash
git -C ~/Repos/notez add install.sh
git -C ~/Repos/notez commit -m "feat: add installer with macOS codesign step"
```

- [ ] **Step 6: [MACHINE B] Install there too**

Machine B must run the new binary before the migration lands, because the old binary rewrites legacy structures on `-g` commands and would undo the migration. Hand over:

```bash
cd ~/Repos && git clone git@github.com:Gaurgle/notez.git
cd ~/Repos/notez && ./install.sh
notez --version
```

---

## Phase 4: Migrate the vault

### Task 4.1: Migrate legacy layout, one machine, with review

`migrate-from-legacy` is conservative by design: it previews a plan, merges entry by entry, never overwrites, materializes private symlink targets into real files, prunes dangling links, and reports collisions instead of guessing. The vault currently holds 18 symlinks in its top two levels pointing at machine-specific absolute paths, which is what breaks it across machines today.

**Files:** none in the repo. This operates on `~/notez`.

**Interfaces:**
- Consumes: the installed `notez` binary from Task 3.1.
- Produces: a migrated vault, pushed; machine B pulled and re-attached.

- [ ] **Step 1: Confirm the restore point exists**

```bash
git -C ~/notez log --oneline -3
git -C ~/notez status --short
```

Expected: the checkpoint commit from Task 1.1 Step 1 is present and the tree is clean. **If it is dirty, stop and commit before going further.**

- [ ] **Step 2: Preview the migration and read the plan**

```bash
notez migrate-from-legacy --dry-run
```

This prints one `from -> to [note]` line per planned move and changes nothing
(`commands/migrate.rs:21`). Read every line before executing. The `[note]`
column is where collisions are reported, and those are the whole point of the
preview.

If it prints "Nothing to migrate", the legacy project registry at
`~/.config/notez/projects` did not match any numbered directory. Stop and
report rather than proceeding: it means the migration cannot see the vault's
structure, not that there is nothing to do.

- [ ] **Step 3: Resolve the known duplicate directories by hand**

The vault contains three pairs that will surface as collisions:

- `02_app2` and `11_app2`
- `05_repoz` and `09_repoz`
- `07_wireless-test-hub` and `14_wireless-test-hub`

For each pair, inspect both and merge into one before migrating:

```bash
ls -la ~/notez/02_app2 ~/notez/11_app2
ls -la ~/notez/05_repoz ~/notez/09_repoz
ls -la ~/notez/07_wireless-test-hub ~/notez/14_wireless-test-hub
```

Move files from the redundant directory into the keeper, then remove the emptied directory. **Ask the user which of each pair to keep. Do not choose for them.**

- [ ] **Step 4: Execute the migration**

Run the command without the preview flag. Then read what it reports, particularly any remaining collisions.

- [ ] **Step 5: Review the diff before committing**

```bash
git -C ~/notez status --short | head -50
git -C ~/notez status --short | wc -l
find ~/notez -maxdepth 2 -type l | wc -l
```

Expected: the symlink count has dropped toward 0, and the status shows moves into `personal/`. **If notes appear deleted without a corresponding addition, stop and report.**

- [ ] **Step 6: Commit and push the migrated vault**

```bash
git -C ~/notez add -A
git -C ~/notez commit -m "chore: migrate vault to the scope model"
git -C ~/notez push
```

- [ ] **Step 7: [MACHINE B] Pull and rebuild the local registry**

The registry is per-machine by design and is not synced, so machine B has to re-attach its projects. This is expected, not a failure.

```bash
git -C ~/notez pull
cd ~/Repos/<each-project> && notez attach
notez list
```

- [ ] **Step 8: Verify on both machines from a subdirectory**

```bash
cd ~/Repos/notez/crates/notez-core && notez tree
notez list
```

Expected: the same notes visible on both machines, and no `.notez` created in the subdirectory.

---

## Phase 5: Docs

### Task 5.1: Update the references that still describe the old world

**Files:**
- Modify: `~/.claude/CLAUDE.md` (the notez line under Shell Environment)
- Modify: the `notez` skill
- Modify: `~/Repos/notez/README.md` (installer section)

- [ ] **Step 1: Fix the global CLAUDE.md pointer**

It currently reads:

> - Notes & todos: `notez`/`todoz` (custom CLI, source at `~/Repos/notez-cli`) - usage details in the `notez` skill

Change `~/Repos/notez-cli` to `~/Repos/notez`.

- [ ] **Step 2: Update the notez skill**

Locate it:

```bash
grep -rln "notez" ~/.claude/skills ~/claude-config 2>/dev/null | head
```

The skill documents the legacy command surface. It needs:
- the two-axis scope model (personal / public / global / scratch) replacing the old global-versus-local framing
- the `-l` scratch flag, which did not exist before
- `attach`, `detach`, `list`, `sync`, `migrate-from-legacy`
- the note that scratch and public stores now resolve at the project root, so subdirectories no longer spawn their own
- `demo` removed from the documented surface

- [ ] **Step 3: Document the installer in the repo README**

Add an Install section to `~/Repos/notez/README.md`:

```markdown
## Install

```bash
git clone git@github.com:Gaurgle/notez.git
cd notez && ./install.sh
notez setup
```

Installs `notez` to `~/.local/bin` along with the alias symlinks
(`todoz`, `zlog`, `logz`, `zlogs`, `znote`, `treez`, `editz`, `findz`).
```

- [ ] **Step 4: Final verification**

```bash
cd ~/Repos/notez && cargo test --workspace
grep -rn "notez2" ~/Repos/notez --exclude-dir=.git --exclude-dir=node_modules --exclude-dir=docs
grep -rn "notez-cli" ~/.claude/CLAUDE.md
```

Expected: tests pass, no `notez2` outside `docs/`, no stale `notez-cli` path in the global config.

- [ ] **Step 5: Commit**

```bash
git -C ~/Repos/notez add README.md
git -C ~/Repos/notez commit -m "docs: document the installer"
```

The `~/.claude/CLAUDE.md` and skill edits live in `~/claude-config`; commit them there per that repo's own ship policy (commit and push to `main`).

---

### Task 4.2: Sweep orphaned subdirectory stores

Task 2.1 stops the resolver looking below the project root, which is the point.
But stores created by the old cwd-based behavior already exist in
subdirectories, and after Task 2.1 notez stops seeing them. Nothing is
deleted; the notes simply go invisible. This task moves them where the
resolver will find them.

As of 2026-09-01 there are six, four holding a note each:

| Orphan store | Project root | `.md` files |
| --- | --- | --- |
| `~/Repos/career/applications/marshall/.notez` | `~/Repos/career` | 1 |
| `~/Repos/career/applications/marshall/study/.notez` | `~/Repos/career` | 1 |
| `~/Repos/IMRSV/file-gatherer/spike/2026-08-31-dry-run/.notez` | `~/Repos/IMRSV/file-gatherer` | 1 |
| `~/Repos/repoz/notez/.notez` | `~/Repos/repoz` | 1 |
| `~/Repos/IMRSV/file-gatherer/docs/superpowers/.notez` | `~/Repos/IMRSV/file-gatherer` | 0 |
| `~/Repos/J24-examen/examen/.notez` | `~/Repos/J24-examen` | 0 |

Note that `~/Repos/IMRSV/file-gatherer/.notez`, `~/Repos/Rust/rustfinity/.notez`,
`~/Repos/Sigma/wireless-test-hub/.notez` and `~/Repos/Sigma/app2/.notez` are
**not** orphans. They sit at their own git toplevel and are merely nested
inside a grouping folder. Leave them alone.

**Files:** none in the repo. This operates on stores under `~/Repos`.

- [ ] **Step 1: Re-scan, because the list may have changed since the plan was written**

```bash
for d in ~/Repos/*/ ~/Repos/*/*/; do
  [ -d "$d/.notez" ] || continue
  top=$(git -C "$d" rev-parse --show-toplevel 2>/dev/null) || continue
  [ "$top" = "${d%/}" ] && continue
  echo "ORPHAN under $top -> ${d}.notez ($(find "$d/.notez" -name '*.md' | wc -l | tr -d ' ') md)"
done
```

Work from this output, not from the table above.

- [ ] **Step 2: Show the user what each orphaned note contains**

For every orphan with `.md` files:

```bash
find ~/Repos/career/applications/marshall/.notez -name '*.md' -exec echo "--- {}" \; -exec head -20 {} \;
```

Repeat per orphan. **Ask the user what to do with each one.** A note in
`spike/2026-08-31-dry-run/` may be worth keeping next to that spike as an
ordinary file rather than as a notez store.

- [ ] **Step 3: Move the keepers into the project root store**

For each note the user wants kept, with `<ORPHAN>` and `<ROOT>` filled in:

```bash
mkdir -p <ROOT>/.notez
mv -n <ORPHAN>/.notez/<note>.md <ROOT>/.notez/
```

`mv -n` refuses to overwrite. If it reports a collision, rename the incoming
file with a suffix rather than clobbering, and tell the user.

- [ ] **Step 4: Remove the emptied stores**

Only directories that are now genuinely empty:

```bash
find ~/Repos -type d -name ".notez" -empty -mindepth 3 -print -delete
```

The `-print` shows exactly what went. `-empty` means nothing with content can
be caught by this.

- [ ] **Step 5: Verify no orphans remain**

Re-run the Step 1 scan. Expected: no output.

- [ ] **Step 6: Confirm the notes are now reachable**

```bash
cd ~/Repos/career/applications/marshall && notez -l tree
```

Expected: the swept notes appear, resolved from `~/Repos/career/.notez`.

---

## Follow-up, deliberately not scheduled here

- **13 em-dashes remain in the Rust sources**, one of them in user-visible
  output: `crates/notez-cli/src/commands/migrate.rs:21` prints
  `(dry run — nothing changed)`. This violates the global no-em-dash rule.
  Cleaning them is a one-line sweep but it is not part of this cutover, and
  notez-cli has a precedent commit for exactly this (`docs: replace em-dashes
  in README`). Raise it with the user as its own small change.
- **`CLAUDE.md` for the repo.** The global convention requires every repo to
  carry one with the house rules block plus its branch and PR policy. This
  repo has none. It becomes the surviving repo, so it should get one shortly
  after the cutover.

## Done when

- `Gaurgle/notez` is the code, `Gaurgle/notez-vault` is the vault, `notez-cli` is archived, `~/Repos/epoz` is gone.
- `notez --help` shows no "not ported yet" except `demo`, which says plainly that it is not implemented.
- `notez -l add` from a subdirectory writes to the project root's `.notez/`.
- `cargo test --workspace` passes.
- Both machines run the same binary and see the same notes after `notez sync`.
- `todoz -g` shows the global board, all five `_todos/` categories
  (general, IDEAS, IMRSV, MARSHALL, work) and every registered project.
- No `.notez` store exists below any project root.
