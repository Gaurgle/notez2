# SDD ledger — plan: /Users/at-a/Repos/notez2/docs/superpowers/plans/2026-09-01-notez-consolidation.md

Spec: /Users/at-a/Repos/notez2/docs/superpowers/specs/2026-09-01-notez-consolidation-design.md (reachable)
Scope of this run: Phase 2 only (Tasks 2.1 - 2.5). Phases 1, 3, 4, 5 are held
back deliberately: Phase 1 Task 1.1 is gated on the user repointing a second
machine's git remote, and Phases 3-5 depend on Phase 1 landing.
Worktree: /Users/at-a/Repos/notez2/.claude/worktrees/phase2 (branch phase2-subdir-and-stubs)
Baseline at 3e5175e: notez-core 124 passed, notez-cli 68 passed, 0 failed.

## Pre-flight conflict scan

### Pairs sharing a file or an interface

| A | B | A produces / B consumes | Finding |
| --- | --- | --- | --- |
| 2.1 | 2.2, 2.3, 2.4 | `resolve::root`/`daily_logs` rooted at project root | Clean. 2.1 lands first; consumers inherit it. |
| 2.2 | 2.3 | `logz::open_dir(&Path, &Config)` | Clean. 2.3 imports it; 2.2 defines it as `pub`. |
| 2.3 | 2.4 | `pub mod picker;` declaration in `commands/mod.rs` | **CONFLICT (ordering).** See ruling 2 below. |
| 2.2 | 2.3 | both edit `commands/mod.rs` | Clean. Sequential dispatch, different lines. |
| 2.2, 2.4, 2.5 | - | all edit `main.rs` | Clean. Sequential dispatch, different match arms. |

### Per-task self-consistency

| Task | Tests specified vs code specified | Files created vs later touched | Finding |
| --- | --- | --- | --- |
| 2.1 | Tests call `root()`; binding named `toplevel` to avoid shadowing the fn | resolve.rs only | Clean. Existing two fallback tests stay valid (non-git tempdir). |
| 2.2 | Test needs `prepare`; impl defines it | Creates logz.rs, registers it in mod.rs at Step 4 | **DEFECT.** See ruling 1 below. |
| 2.3 | Test asserts sorted order 00, 01, personal/alpha, personal/beta; impl sorts tops then extends personal children | nav.rs, mod.rs | Clean. `personal` sorts after `01_`, so the expected order holds. |
| 2.4 | Test builds `NoteEntry` with all 5 fields; type derives Clone | edit.rs, main.rs | Clean. `Scope::label()` and `collect_in_scope` both verified to exist. |
| 2.5 | No test (string change only) | main.rs | Clean. Covered by the workspace build. |

## Rulings

Ruling 1 (Task 2.2, plan defect): The brief puts `pub mod logz;` registration
at Step 4, after Step 2 runs the test. A Rust test in an unregistered module
is never compiled, so Step 2 cannot fail the way the brief predicts — it
would report zero tests, not a missing function. Decision: registration moves
ahead of the first test run. Carried in the dispatch. Cost if wrong: none;
this is the only order in which the RED state is observable.

Ruling 2 (Tasks 2.3 / 2.4, ordering): `commands/picker.rs` is an orphan file,
never declared, never compiled. Task 2.3 declares it; Task 2.4 depends on it.
Decision: keep the declaration in 2.3 and dispatch strictly 2.3 before 2.4.
Cost if wrong: 2.4 fails to compile, caught immediately by its own test run.

Ruling 3 (Task 2.2, RED state): The brief says the test should "fail on
behavior rather than on a missing file", then predicts a compile error. In
Rust these are the same RED state for a function that does not yet exist.
Decision: a compile error naming the missing function is an acceptable RED.
Cost if wrong: none; it is the conventional Rust TDD red.

## Progress
Task 2.1: dispatched (implementer, sonnet, BASE=3e5175e)
Task 2.1: implementer DONE (commit 36e408d, 199 workspace tests pass, no concerns)
Task 2.1: task reviewer dispatched (sonnet, package review-3e5175e..36e408d.diff)
Task 2.1: task reviewer KILLED by host reboot at 13:45 UTC, no verdict emitted.
Task 2.1: review redone by hand 2026-09-01 (diff is one file, 79 lines).
  Verdict PASS. project_root() delegates to Project::try_detect() (git toplevel,
  cwd fallback); Local/Public root there; Personal/Global untouched; doc comment
  updated to match. Three new tests are serial_test because they mutate process
  cwd, canonicalize paths to match git's report, and restore cwd afterwards.
  199 workspace tests pass. cargo fmt and clippy report pre-existing drift in
  todo/mod.rs, tree.rs, tui/todo.rs, text.rs and aggregate.rs; none in
  resolve.rs, so this change introduced none of it.
  Nit, not blocking: the tests restore cwd after calling root(), so a panic
  inside root() would leak the cwd change. serial_test bounds the blast radius.
Task 2.1: merged to master as 064c61d (by the user, outside the agent session).
  Both master and phase2-subdir-and-stubs are pushed to origin.

Phase 1 Task 1.1 Step 1 is DONE (vault checkpoint), out of plan order:
  ~/notez committed as e05661c "chore: checkpoint before notez consolidation",
  then merged with 3 incoming commits from machine B as 114ea17, then pushed.
  Two conflicts resolved by hand, see the report in that session.
  Steps 2-5 of Task 1.1 (GitHub rename, remote repointing) are NOT done and
  still gate everything in Phase 1 after them.

NEXT: Task 2.2 (logz). Ruling 1 applies: register `pub mod logz;` BEFORE the
first test run. Then 2.3 (nav) strictly before 2.4 (edit), per ruling 2.
