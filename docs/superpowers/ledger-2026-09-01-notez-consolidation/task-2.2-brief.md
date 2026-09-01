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

