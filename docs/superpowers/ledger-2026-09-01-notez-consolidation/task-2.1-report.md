# Task 2.1 Report: Root scratch and public stores at the project root

## What was implemented

`resolve::root` in `crates/notez-core/src/core/resolve.rs` previously resolved
`Scope::Local` to `current_dir()?.join(".notez")` and `Scope::Public` to
`current_dir()?.join("notez")`, with no upward walk to the project root. Running
any command from a subdirectory created a brand-new note store there instead of
using the project's single store.

Added a private helper `project_root()` that returns the git toplevel (via
`Project::try_detect()`) when inside a repo, otherwise falls back to the cwd
unchanged. Updated the `Local` and `Public` match arms in `root()` to call
`project_root()` instead of `std::env::current_dir()` directly. `Personal` and
`Global` arms are untouched.

Updated the doc comment on `root()` to describe the new behavior (git
toplevel, else cwd) instead of the old cwd-always behavior.

`resolve::quick_notes` and `resolve::daily_logs` both call `root()` internally
and needed no changes; they inherit the fix.

## Files changed

- `crates/notez-core/src/core/resolve.rs` (75 insertions, 4 deletions)
  - Added `project_root()` helper function with doc comment
  - Changed `Scope::Local` and `Scope::Public` arms in `root()`
  - Updated doc comment on `root()`
  - Added test helper `git_repo_with_subdir()`
  - Added 3 new tests: `local_resolves_to_project_root_from_a_subdirectory`,
    `public_resolves_to_project_root_from_a_subdirectory`,
    `daily_logs_follow_the_project_root_from_a_subdirectory`

No other files changed. No new dependencies added (confirmed `clap`,
`ratatui`, `dialoguer`, `console` are absent from `notez-core/Cargo.toml`).

## TDD evidence

### RED

Command:
```
cd /Users/at-a/Repos/notez2/.claude/worktrees/phase2 && cargo test -p notez-core resolve:: -- --nocapture
```

Output (relevant excerpt):
```
test core::resolve::tests::public_resolves_to_project_root_from_a_subdirectory ... FAILED
test core::resolve::tests::daily_logs_follow_the_project_root_from_a_subdirectory ... FAILED
test core::resolve::tests::local_resolves_to_project_root_from_a_subdirectory ... FAILED
test core::resolve::tests::personal_inside_git_uses_personal_subdir ... ok
test core::resolve::tests::local_uses_dot_notez_in_cwd ... ok
test core::resolve::tests::personal_falls_back_to_global_outside_git ... ok
test core::resolve::tests::public_uses_notez_in_cwd ... ok

test result: FAILED. 6 passed; 3 failed; 0 ignored; 0 measured; 118 filtered out

thread 'core::resolve::tests::public_resolves_to_project_root_from_a_subdirectory' panicked:
assertion `left == right` failed
  left: ".../.tmpbm1aVf/crates/deep/notez"
 right: ".../.tmpbm1aVf/notez"

thread 'core::resolve::tests::daily_logs_follow_the_project_root_from_a_subdirectory' panicked:
assertion `left == right` failed
  left: ".../.tmpLaMxK3/crates/deep/.notez/01_daily-logs"
 right: ".../.tmpLaMxK3/.notez/01_daily-logs"

thread 'core::resolve::tests::local_resolves_to_project_root_from_a_subdirectory' panicked:
assertion `left == right` failed
  left: ".../.tmpOMLYKk/crates/deep/.notez"
 right: ".../.tmpOMLYKk/.notez"
```

Why this failure was expected: before the fix, `root()` used
`current_dir()` directly, so from the nested `crates/deep` subdirectory it
produced a store under that subdirectory (`left`) instead of the project
toplevel (`right`). The two pre-existing fallback tests
(`local_uses_dot_notez_in_cwd`, `public_uses_notez_in_cwd`), which run in a
non-git tempdir with no subdirectory nesting, correctly passed both before
and after, since cwd IS the project root in that case.

### GREEN

Command:
```
cd /Users/at-a/Repos/notez2/.claude/worktrees/phase2 && cargo test -p notez-core
```
Result: `127 passed` (0 failed).

Command:
```
cd /Users/at-a/Repos/notez2/.claude/worktrees/phase2 && cargo test -p notez-core -p notez-cli
```
Result: `195 passed` (0 failed).

Command:
```
cd /Users/at-a/Repos/notez2/.claude/worktrees/phase2 && cargo test --workspace
```
Result: `199 passed` (6 suites, 0 failed).

## Self-review findings

- Diff matches the brief verbatim: helper function, doc comment, match arm
  changes, and all three new tests copied exactly, including the deliberate
  `toplevel` (not `root`) local binding naming to avoid shadowing the function
  under test.
- `cargo fmt --check -p notez-core` reports pre-existing formatting diffs in
  other files (`config/mod.rs`, `config/registry.rs`, `core/aggregate.rs`,
  `core/project.rs`, `filter.rs`, `migrate.rs`, `todo/mod.rs`) but none in
  `resolve.rs` -- confirmed via `grep -i resolve.rs` returning no matches.
  These pre-existing diffs are out of scope for this task and were not
  touched.
- `cargo clippy -p notez-core --all-targets` reports 5 pre-existing warnings,
  none in `resolve.rs` (confirmed via grep). Out of scope, not touched.
- `git diff HEAD~1 HEAD --stat` confirms only `resolve.rs` changed.
- No em-dashes/en-dashes in code or comments (checked visually).
- The two existing tests `local_uses_dot_notez_in_cwd` and
  `public_uses_notez_in_cwd` were left completely unmodified and both still
  pass, now exercising the outside-a-git-repo (non-git tempdir) fallback
  path through `project_root()`'s `None` branch.
- `notez_root` config test (`quick_notes_joins_subdir`) unaffected since it
  uses `Scope::Global`.

## Concerns

None. Implementation is a minimal, surgical change exactly matching the
brief's interfaces and constraints. All 199 workspace tests pass.

## Commit

```
36e408d8cf2118babe2b1024f133d5f4dd1b9618 fix: root scratch and public notes at the project root
```
