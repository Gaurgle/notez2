# notez consolidation: one repo, one name, one binary

Date: 2026-09-01
Status: approved design, pending implementation plan

## Problem

Five repos in the `~/Repos` tree carry overlapping names, and two names are
claimed by two different things each. The result is that it is no longer
obvious which repo is "notez", which binary is installed, or what `epoz`
refers to.

The concrete symptom that started this: running `notez` from a subdirectory
of a project creates and uses a `.notez/` in that subdirectory instead of at
the project root.

## Current state (verified 2026-09-01)

| Repo | What it is | State |
| --- | --- | --- |
| `notez-cli` | Original CLI. `.notez` resolved from cwd. | Deprecated 2026-07-05, but this is the installed binary |
| `notez2` | The rewrite: `notez-core` + `notez-cli` crate + `app/src-tauri` (productName `epoz`) | Last commit 2026-08-04, never built locally, three commands stubbed |
| `epoz` (local dir) | Stale clone of what became fleetz | Remote `Gaurgle/epoz` already redirects to `Gaurgle/fleetz` |
| `fleetz` | Multi-repo dashboard TUI | Active, 2026-08-28 |
| `repoz` | CLI multi-repo status | Active, 2026-08-24 |

Installed binary: `~/.local/bin/notez`, built 2026-04-30 from `notez-cli`.
Identified by the "Standalone commands" help block, which exists only in
`notez-cli/src/main.rs`, and by the absence of `notez sync` (added to
notez-cli on 2026-07-04, after the build).

GitHub name usage:

- `Gaurgle/notez` is PRIVATE, described "Personal notes, synced via notez-cli".
  It is the data vault behind `~/notez`, not code.
- `Gaurgle/notes` is PRIVATE, the older `~/notes` tree.
- `Gaurgle/epoz` already redirects to `Gaurgle/fleetz`.

### The subdir bug: half fixed by the rewrite, half still open

`notez-cli/src/project.rs:32` defines `local_private_dir()` as
`current_dir().join(".notez")`. There is no upward walk, so every
subdirectory becomes its own note root.

In notez2 the picture is split, and only part of it is already correct:

- **Fixed.** `Scope::Personal`, the default scope, resolves to
  `<notez_root>/personal/<project>/` via `Project::try_detect()`, which is
  the git toplevel. Subdir-invariant already.
- **Fixed.** The aggregation view roots local stores at
  `cwd_project.root.join(".notez")` (`core/aggregate.rs:88`), so `tree` and
  the todo roll-up already read from the project root.
- **Still broken.** `core/resolve.rs` resolves `Scope::Local` as
  `current_dir()?.join(".notez")` and `Scope::Public` as
  `current_dir()?.join("notez")`, both documented "always; never inspects
  project". Every command goes through this: `add` (`add.rs:39,84`), `log`
  (`log.rs:17`), `todo` (`todo.rs:30,78`), `mkdir` (`mkdir.rs:19`), and
  `todo/mod.rs:361`.

So `-l` scratch and `-p` public notes still scatter per subdirectory. The
original request needs a real change in `resolve.rs`, planned as Task 2.1.
It does not arrive free with the cutover.

### What happens to todoz

The `todoz` surface survives the cutover intact, and the global board
improves. Recorded here because it was not obvious and had to be checked.

- The alias is ported: argv-0 dispatch maps `todoz` to `notez todo`
  (`notez-cli/src/main.rs:156`), and the installer symlinks it. Both the TUI
  and `todoz "item"` quick-add work.
- `todoz -g` is assembled by `todo::load_board` (`notez-core/src/todo/mod.rs:291`):
  the global `TODO.md`, then every `_todos/<category>/TODO.md`, then each
  registered project's personal, public and local boards. It is driven by the
  **project registry rather than symlinks**, which is what makes it correct on
  a second machine.
- The five `_todos/` categories (general, IDEAS, IMRSV, MARSHALL, work)
  survive untouched: migration deliberately skips `_todos/`, and notez2 reads
  the directory directly. No action needed.
- The 18 per-project `NN_<project>/TODO.md` files ride along when migration
  moves each numbered dir to `personal/<name>/`, which is exactly where the
  personal board looks.

**One behavior change.** Bare `todoz` inside a project used to write to the
repo's `.notez/TODO.md` (`notez-cli/src/commands/todo.rs:1101`). The new
default scope is Personal, so it becomes
`<notez_root>/personal/<project>/TODO.md`. This is what makes todos sync
across machines, which the in-repo file never did. The old location stays
reachable with `todoz -l`.

### Cross-machine correctness

Work happens on two machines, so the vault must stay in sync. notez2 is built
for this and it is the reason the rewrite exists:

- Per-machine state (`config.toml`, `registry.toml`) lives in
  `~/.config/notez/` and is never synced (`crates/notez-core/src/config/paths.rs`).
- Synced metadata lives at `<notez_root>/.notez-config.toml`.
- `notez sync` is `git pull --rebase` then `git push` on the vault root.

The legacy layout is what breaks across machines: `~/notez` currently holds
18 symlinks in its top two levels, pointing at absolute paths that differ per
machine, plus `~/.config/notez/projects` storing absolute project paths.

## What is renamed, archived, and deleted

The word "notez" currently names two unrelated things, which is the root of
the confusion. Stated once, plainly:

| Today | Contents | Action |
| --- | --- | --- |
| `Gaurgle/notez` (private) | The notes vault behind `~/notez` | Renamed to `notez-vault`. Kept. |
| `Gaurgle/notez2` (public) | The code | Renamed to `notez`. Becomes the one repo. |
| `Gaurgle/notez-cli` (public) | Superseded code | Archived, not deleted. README becomes a pointer. |
| `~/Repos/epoz` (local only) | Stale clone of what is now fleetz | Deleted. |

Nothing holding notes is deleted at any point. The `notez` name is freed by
moving the vault aside, not by removing it. `notez-cli` is archived rather
than deleted so existing clones and links keep resolving.

## Target end state

One repo named `notez`, containing:

- `crates/notez-core`: the engine (scopes, aggregation, todo model, tags, migrate)
- `crates/notez-cli`: the `notez` binary plus its argv-0 aliases
- `app/`: epoz, the desktop app, and later its own CLI crate on the same core

epoz stays in the monorepo. That is what makes "same engine" cheap: both
front ends depend on `notez-core` by path, with no publishing step and no
cross-repo lockstep.

## Decisions

1. **Clean cutover**, not a transitional period. One binary, one name.
2. **epoz stays in the monorepo.** Assumed from the stated goal ("one core,
   implemented in epoz and as a CLI tool") rather than explicitly confirmed.
   Reversible: splitting later costs a published or git-pinned `notez-core`.
3. **The vault moves, the code takes the name.** `Gaurgle/notez` becomes
   `Gaurgle/notez-vault`; `Gaurgle/notez2` becomes `Gaurgle/notez`.
4. **`~/notes` is out of scope.** Left untouched on disk and revisited later.
5. **No stale-clone archaeology.** `~/Repos/epoz` is deleted, not archived;
   the GitHub side was already resolved by the fleetz rename.
6. **`demo` stays stubbed.** Not ported. It is a screenshot helper, not part
   of daily use.
7. **Branch flow:** the default branch is renamed from `master` to `main`,
   and work lands as direct commits on it. No PR step, no branch per phase.
   This repo carries no `CLAUDE.md`; recording the flow there is a separate
   piece of work, noted below.

## Phases

Order matters. Each phase states why it sits where it does.

### Phase 1: naming

Rename the vault first, then the code, then repoint remotes. The dangerous
window is between the two renames: GitHub leaves a redirect from `notez` to
`notez-vault`, and creating a new `Gaurgle/notez` drops that redirect. A
machine whose `~/notez` remote still says `Gaurgle/notez` would then be
pushing personal notes at the public code repo.

Rule: **update the `~/notez` remote on BOTH machines before renaming
notez2.** This is the one irreversible-if-wrong step in the plan.

1. Rename `Gaurgle/notez` to `Gaurgle/notez-vault`.
2. On machine A and machine B: `git -C ~/notez remote set-url origin git@github.com:Gaurgle/notez-vault.git`, then verify with `git -C ~/notez remote -v` and a `git fetch`.
3. Rename `Gaurgle/notez2` to `Gaurgle/notez`.
4. Move `~/Repos/notez2` to `~/Repos/notez` and update its origin URL.
5. Rename "notez2" to "notez" repo-wide: 31 occurrences across 19 files.
   Most are mechanical (doc comments, crate descriptions, the help header at
   `crates/notez-cli/src/main.rs:196`, user-visible strings at `main.rs:57`,
   `main.rs:235`). Three are not, and must be rewritten by hand rather than
   substituted: the "Naming:" paragraph in `README.md:5`, the same paragraph
   in `app/README.md:5`, and the naming rationale at `DESIGN.md:533`. All
   three explain the old notez2-versus-epoz split that this change removes.
   `DESIGN.md:272` lists "notez2" as a repo name in an example and should
   become "notez".
6. Reduce `Gaurgle/notez-cli`'s README to a pointer at `Gaurgle/notez`, then
   archive the repo.
7. Delete the stale `~/Repos/epoz` clone.

### Phase 2: close the three gaps

`edit`, `nav` and `logz` currently bail with "not yet implemented"
(`commands/edit.rs`, `commands/nav.rs`, `main.rs:81`). All three are thin
shell-outs in notez-cli and port small:

- `logz`: launch yazi (or `$EDITOR`) on the resolved daily-logs directory.
  Source: `notez-cli/src/commands/browse.rs:run_logz`, about 20 lines.
- `nav`: fzf picker over the vault, then yazi on the choice. Source:
  `notez-cli/src/commands/nav.rs`. notez2 already has `commands/picker.rs`
  to build on.
- `edit`: fzf fuzzy-match on filename, then `$EDITOR`. Source:
  `notez-cli/src/commands/edit.rs`.

Each must resolve its directory through the `Scope` model, not `current_dir()`.
That is the whole point of the cutover and the fix for the original bug.

A fourth stub, `demo` (`main.rs:93`), is deliberately left in place. Its
message should lose the "coming in the next milestone" promise and say
plainly that it is not implemented.

Tests: cover scope resolution from a subdirectory for each command, asserting
the resolved path equals the git-toplevel-rooted store. The shell-out itself
is not worth testing.

### Phase 3: install

notez2 has no `install.sh`. Port the one from `notez-cli/install.sh`:

- `cargo build --release`
- copy to `~/.local/bin/notez`
- `codesign --force --sign -` on the installed binary. **Not optional on
  macOS ARM**: `cp` over an existing Mach-O invalidates the ad-hoc signature
  and the kernel SIGKILLs the binary on launch.
- symlink the aliases: `todoz zlog logz znote treez editz findz`

Install on both machines before either is used against migrated data. The old
binary rewrites legacy structures on `-g` commands, so leaving it in place on
machine B would undo the migration.

### Phase 4: migrate the vault

`migrate-from-legacy` already exists in `crates/notez-core/src/migrate.rs`
and is conservative by design: it previews a plan, merges entry by entry,
never overwrites, materializes private symlink targets into real files,
prunes dangling links, and reports collisions instead of guessing.

Sequencing across two machines:

1. On machine A: commit the current dirty state of `~/notez` (6 modified or
   untracked entries as of 2026-09-01) as a restore point, and push.
2. Run the migration in preview mode and read the plan.
3. Resolve the known duplicate numbered dirs by hand. `~/notez` contains
   `02_app2` and `11_app2`, `05_repoz` and `09_repoz`, `07_wireless-test-hub`
   and `14_wireless-test-hub`. These will surface as collisions.
4. Execute, review the diff, commit, push.
5. On machine B: `git pull`, then `notez attach` per project to rebuild that
   machine's registry. The registry is per-machine by design and is not
   synced; this step is expected, not a failure.
6. Verify from a subdirectory on both machines that `notez` resolves to the
   project root.

### Phase 5: docs and config

- The global `CLAUDE.md` names `~/Repos/notez-cli` as the source of the CLI.
  Update to `~/Repos/notez`.
- The `notez` skill documents the old command surface, including the cwd
  behavior this change removes. Update to the scope model.
- Rename the default branch from `master` to `main`, locally and on GitHub,
  and update the remote HEAD.

The repo has no `CLAUDE.md`, which the global convention requires for every
repo (house rules block, branch and PR policy). Adding one is out of scope
here but should follow shortly, since this repo becomes the survivor.

## Risks

| Risk | Mitigation |
| --- | --- |
| Notes pushed to the public code repo during the rename window | Repoint both machines' remotes before renaming notez2. Verify with `git remote -v` and a fetch on each. |
| Migration loses or merges notes wrongly | Commit and push the vault first. Preview the plan before executing. The migration never overwrites; collisions are reported. |
| Old binary on machine B rewrites migrated structure | Install the new binary on both machines before migrating. |
| New binary SIGKILLed after install on macOS ARM | `codesign --force --sign -` in `install.sh`, applied after the copy. |
| Duplicate numbered dirs merge into the wrong project | Resolved by hand in phase 4 step 3, before executing. |

## Out of scope

- `~/notes` and `Gaurgle/notes`. Left untouched.
- `repoz` and `fleetz`. Unaffected.
- The epoz CLI crate. It is the reason for the monorepo shape but is not
  built here.
- Any change to notez-cli beyond the README pointer. It is being archived.
