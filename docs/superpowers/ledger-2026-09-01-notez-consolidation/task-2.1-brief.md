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

