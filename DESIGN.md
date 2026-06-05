# notez2 design

A from-scratch rewrite of [notez-cli](https://github.com/Gaurgle/notez-cli) with cross-machine portability as a first-class concern. UX surface is preserved 1:1; only the storage layer is reworked.

## Goals

1. Same CLI and TUI surface as notez-cli. Same keybindings, same flags, same z-binary aliases (`todoz`, `zlog`, `znote`, `treez`, `logz`, `editz`, `findz`).
2. No OS symlinks. The current `~/notez/NN_<project>/` symlink scheme breaks when machines have different usernames or repo layouts.
3. Cross-machine sync of `~/notez/` works via plain `git pull/push`. No filesystem state assumes a specific home prefix.
4. Per-machine project registry is per-machine and never synced. Each machine learns where its own projects live.
5. No 100-project limit. No numbered directory allocation. Real names.
6. macOS and Linux first. Windows kept in mind but not blocking.

## Storage model

Four scopes. The TUI aggregates them into one view at runtime.

**Local** (`<project>/.notez/`, gitignored):
- Per-machine scratch. Truly private, never syncs anywhere.
- Falls back to `<cwd>/.notez/` even outside a git repo.
- For ephemeral notes that don't need to follow you.

**Personal** (`<notez_root>/personal/<project>/`, default scope):
- Your notes about this specific project, synced via your own notez remote.
- Invisible to teammates because they live in your `~/notez/` repo, not the
  project's repo.
- When you're inside a git project, this is the default for `notez add`.
- Outside a git project, falls back to global (no project subdir).

**Public** (`<project>/notez/`, committed):
- Public notes shared with the team via the project's git remote.
- Travels with the project automatically.

**Global** (`<notez_root>/`, has its own git remote):
- Cross-project notes, daily logs, todoz categories, scratch pad.
- Synced between machines via `notez sync` (wraps `git pull --rebase && git push`).
- Contains a `.notez-config.toml` metadata file (synced) with project display
  names, descriptions, tags.

### Why personal exists

In notez-cli, private project notes lived in `<project>/.notez/` and were
mirrored into `~/notez/<project>/` via OS symlinks. The symlinks stored
absolute paths, which broke when usernames or repo layouts differed between
machines.

Personal scope solves the same problem from the other direction: instead of
trying to mirror project-local files into your synced home, the notes live
in your synced home from the start. The project repo never knows about them.
No symlinks, no encryption, no extra remotes to set up per project.

### Scope flags

| Flag | Scope | Where |
|---|---|---|
| _(default)_ | Personal | `<notez_root>/personal/<project>/` |
| `-l` `--local` | Local | `<cwd>/.notez/` |
| `-p` `--public` | Public | `<cwd>/notez/` |
| `-g` `--global` | Global | `<notez_root>/` |

## Config files

**Per-machine, not synced**: `$XDG_CONFIG_HOME/notez/config.toml`

```toml
[paths]
notez_root = "~/notez"             # global notes root, stored as tilde-relative
quick_notes_dir = "00_quick-notes"
daily_logs_dir = "01_daily-logs"

[editor]
command = "nvim"
new_note_args = ["+4", "-c", "startinsert"]

[tools]
fzf = true
rg = true
yazi = true
```

All paths are stored tilde-relative and expanded at runtime via `dirs::home_dir()`. No absolute paths persist to disk in the config.

**Per-machine, not synced**: `$XDG_CONFIG_HOME/notez/registry.toml`

```toml
[projects.app2]
local_path = "~/repos/sigma/App2"

[projects.notez-cli]
local_path = "~/Repos/notez-cli"
```

This is the per-machine equivalent of the old `ProjectMapping`. It maps a project name (key) to its location on this machine. The path is stored tilde-relative; absolute resolution happens at every load.

**Synced, lives in ~/notez/**: `~/notez/.notez-config.toml`

```toml
[projects.app2]
display_name = "App2 (Android BLE testing)"
tags = ["sigma", "lia"]
order = 2

[projects.notez-cli]
display_name = "notez"
tags = ["personal"]
order = 4
```

This synced file holds project metadata that should be the same across machines: display names, tags, sort order. The per-machine `registry.toml` decides where the project actually lives on this specific machine. Lookups merge metadata with per-machine paths.

## Project discovery

`notez attach <name> [path]` registers a project on the current machine:
- If `name` is omitted, derives from current dir's git toplevel
- If `path` is omitted, uses current dir
- Stores tilde-relative path in `registry.toml`
- If `~/notez/.notez-config.toml` does not already have an entry for `<name>`, prompts for display name and tags, then writes it

`notez detach <name>` removes the registry entry; does not touch the project's notes.

`notez attach --scan` walks `~/repos/`, `~/Repos/`, and other configurable roots, finding directories with `.notez/` or `notez/` subdirs, and prompts the user to attach each one.

## TUI aggregation

When the user opens the global tree (`notez -g tree`) or global todoz (`todoz -g`), the TUI:

1. Loads `registry.toml` to get the local paths for each project
2. For each registered project, scans `<path>/.notez/` and `<path>/notez/`
3. Loads `~/notez/.notez-config.toml` for project metadata (display name, ordering)
4. Loads `~/notez/` itself for global notes (quick notes, daily logs, `_todos/<category>/TODO.md`)
5. Builds the tree as before, but with no filesystem symlinks involved

A project whose `local_path` does not exist on this machine renders as dimmed with a "(not attached)" marker, so the user knows the metadata exists but the source is unreachable. Notes for that project are not shown.

## Sync

`notez sync` is a thin wrapper:

1. Validates that `~/notez/` is a git repo with a remote
2. Runs `git pull --rebase`
3. Runs `git push`
4. On conflict, surfaces the git output and tells the user to resolve manually

The first-time setup walks the user through `git init && git remote add origin ...` if `~/notez/` does not have a remote yet.

## Tag system

Same as notez-cli:
- Five colored flags: important, prio, longterm, idea, blocked
- Persisted inline in TODO.md as `#important #prio #longterm #idea #blocked` suffixes
- Tree browser persists in `.tags` files (one per notes root), format `<relpath>:<flagbyte>` per line
- Filter syntax `#tagname` with prefix matching, `#13` for tags 1+3, AND across tokens, OR within

## Migration from notez-cli

`notez migrate-from-legacy` reads:
- `~/.config/notez/config` (old kv format)
- `~/.config/notez/projects` (old `name=path`)
- `~/notez/NN_*` symlinked directories

And produces:
- `~/.config/notez/config.toml` (new TOML config)
- `~/.config/notez/registry.toml` (new project registry)
- `~/notez/.notez-config.toml` (new synced metadata)

It does not touch the actual note files; it only re-writes the index layer. Old symlinks remain until the user deletes them. A `--clean` flag also removes the old symlinks after verifying every target is reachable via the new registry.

## Open questions and future work

### Calendar / deadlines for notes and todos (future)

Idea: surface a calendar dimension across notez and todoz. Each note/todo
could carry a **created date** (already implicit in filenames/mtime) and,
more importantly, an optional **deadline and/or event date**. A calendar
view would then link documents to dates — see what's due, what was created
when, and what events are coming up — across scopes and projects.

Open questions when this is picked up:
- Where does the date live? Frontmatter in the `.md` / inline metadata on a
  todo line (e.g. a `@2026-07-01` token) vs. a sidecar index.
- Deadline vs. event vs. created-date as distinct fields.
- Which UI library — Melt UI's `melt` package has **no** Calendar builder
  yet (only the legacy `@melt-ui/svelte` does), so this is a hand-roll or a
  legacy-pkg dependency when the time comes.
- Interaction with todoz tags (`#blocked`, `#longterm`) and sorting.

Deferred — not in the Melt UI refresh (Phase 1). Captured here so it isn't
lost.

### Scope migration (move notes/todos between scopes)

Sometimes a note that started as personal should become public, or a local
scratch should be lifted to personal so it follows you between machines.
Two operations, two complexity profiles:

**Note files**: a single `.md` file moves between scope directories.
Straightforward `notez mv <pattern> --to <scope>` command. Side effects to
think through:

- `personal -> public`: the file now lives in the project repo, will be
  committed next time the project is committed. Visible to teammates from
  that commit onward.
- `public -> personal`: the file leaves the project repo, but its history
  is still there in past commits. Removing from history is a destructive
  rewrite; offer `--rm-from-history` flag but only with strong warnings.
- `personal -> global`: removes the project association, lands in the
  user's cross-project notes.
- `* -> local`: file becomes per-machine again, stops syncing entirely.

**Todo entries**: each scope has its own `TODO.md`, with many entries per
file. Moving a single entry between scopes is fundamentally different from
moving a file - it is an edit operation, not a rename. Best done from
inside the todoz TUI:

- Press `m` on a selected todo: prompt "Move to: local / personal / public
  / global / cancel"
- Implementation: remove the line(s) from source TODO.md (subtree if it has
  subtasks), append to target TODO.md, preserve tags and indent depth

Both deferred until the TUI lands. The note-level `notez mv` could come
sooner if needed; the todo-level move only makes sense once todoz exists.

### TUI interactions for moving

The TUI should support both keyboard and mouse for the same operation, so
users can pick whichever fits their flow:

**Keyboard** (`m` keybind): opens a status-bar prompt
```
Move to: [l]ocal · [p]ersonal · [u]blic · [g]lobal · [esc] cancel
```
Single keystroke commits; cancel returns to navigation. Works in both
the tree browser and todoz, on the currently selected row.

**Mouse drag** (todoz, global view only): drag-and-drop already exists for
reorder. Extending it across section boundaries triggers a scope move
instead of a reorder when:

- The drop target is in a different section than the source
- That section corresponds to a different scope (local/personal/public/global)
  or a different project entirely

Visual cue: target section header highlights while dragging across it.
Cancel by dropping back into the source section.

For the tree browser, drag-and-drop across deeply-nested directory boundaries
is fragile. The tree TUI sticks to the keyboard `m` keybind only.

The two paths converge on the same internal move() function so behavior is
identical regardless of input modality.

## Test scenarios

A representative matrix to validate behavior end-to-end. These are not unit
tests; they exercise multi-machine, multi-user, multi-project realities the
storage layer has to handle. Each scenario lists the setup and the
properties that must hold.

### Cast

- **Alice**: works on two machines, **laptop** (username `alice`, home
  `/Users/alice`) and **desktop** (username `alice-desk`, home
  `/Users/alice-desk`). Has her own `~/notez/` repo on GitHub used as a
  private notez remote.
- **Bob**: works on a single machine **bobpc** (username `bob`). Has his
  own local `~/notez/` repo with no remote.
- **Carol** and **Dave**: each on one machine. Collaborate on a repo
  Alice and Bob are not part of.

### Repos

- `shared-repo-1` and `shared-repo-2`: GitHub repos shared between Alice
  and Bob (Carol and Dave have no access).
- `cd-repo`: GitHub repo shared between Carol and Dave (Alice and Bob have
  no access).

### Scenario A. Alice writes a personal note about shared-repo-1 on laptop

Setup: Alice has `shared-repo-1` cloned at `/Users/alice/repos/shared-repo-1`
on laptop, and at `/Users/alice-desk/Repos/shared-repo-1` (capital R) on
desktop. She has run `notez attach` inside the project on each machine.

Action: on laptop, `cd ~/repos/shared-repo-1 && notez add "API redesign idea"`

Verify:
- File lands at `/Users/alice/notez/personal/shared-repo-1/00_quick-notes/<date>-api-redesign-idea.md`.
- `~/notez/personal/shared-repo-1/` is tracked in Alice's notez repo.
- After `notez sync` on laptop and again on desktop, the file appears at
  `/Users/alice-desk/notez/personal/shared-repo-1/...` (note the different
  home path).
- The file is not visible inside `shared-repo-1`'s git tree on either
  machine, so Bob cannot see it via the project remote.

### Scenario B. Bob pushes a public note in shared-repo-1

Action: Bob runs `notez -p add "deploy steps"` inside `shared-repo-1`,
commits the new `notez/00_quick-notes/<date>-deploy-steps.md`, pushes.

Verify:
- Alice pulls the project on laptop and sees the file in
  `~/repos/shared-repo-1/notez/00_quick-notes/...`.
- `notez tree` on Alice's laptop shows the note under the Public scope for
  `shared-repo-1`, with the team-globe icon.
- Bob's same note is visible on Alice's desktop after she pulls there too.
- The file is not touched by `notez sync`; it travels via the project's
  git remote, not Alice's notez remote.

### Scenario C. Alice writes a local scratch note

Action: Alice on laptop runs `notez -l add "try this branch out"`.

Verify:
- File lands at `/Users/alice/repos/shared-repo-1/.notez/00_quick-notes/...`.
- The `.notez/` directory is gitignored (notez ensures this on first write).
- After `notez sync`: still only on laptop. Not synced to desktop, not
  visible to Bob.
- After `git pull` on desktop: not present (gitignored).

### Scenario D. Bob and Alice both have personal notes for the same project

Setup: both Bob and Alice have attached `shared-repo-1`. Each has their
own `~/notez/personal/shared-repo-1/` directory in their own notez remote.

Verify:
- Alice's `notez tree` shows her personal notes only.
- Bob's `notez tree` shows Bob's personal notes only.
- Neither sees the other's personal notes anywhere.
- Both can see the same public notes (Scenario B).

### Scenario E. Alice's two machines have shared-repo-1 at different paths

Setup: on laptop, registry has `shared-repo-1 = "~/repos/shared-repo-1"`.
On desktop, registry has `shared-repo-1 = "~/Repos/shared-repo-1"`
(capital R).

Verify:
- Registry differs between machines (it's per-machine and not synced).
- Personal notes resolve correctly on each machine via tilde expansion
  against the local home, producing different absolute paths but the
  same project name.
- No symlink ever points at a hardcoded absolute path.

### Scenario F. Carol and Dave's project is invisible to Alice and Bob

Setup: Carol and Dave each have `cd-repo` attached on their machines.
Their `~/notez/` is sync'd between them. Alice has not cloned `cd-repo`.

Verify:
- Alice's `notez list` does not show `cd-repo`.
- Alice's `notez tree` does not show notes from `cd-repo`.
- Carol's `notez tree` shows her personal + public + local notes for `cd-repo`.
- If Alice clones `cd-repo` and runs `notez attach`, it appears in her
  registry. The public notes from Carol show up immediately. Carol's
  personal notes do NOT appear (those live in Carol's notez remote, not
  the project remote).

### Scenario G. Carol shares cd-repo with Alice later

Setup: Carol invites Alice to `cd-repo`. Alice clones it and runs
`notez attach` to register it.

Verify:
- Alice sees all public notes Carol committed.
- Alice does not see any of Carol's personal notes (those are in Carol's
  own `~/notez/`, which Alice has no access to).
- Alice can write her own personal notes about `cd-repo`. These land in
  `~/notez/personal/cd-repo/` on Alice's machine and sync via Alice's
  notez remote.
- Carol cannot see Alice's personal notes about `cd-repo`.

### Scenario H. Alice deletes a personal note on laptop, syncs

Action: Alice deletes `~/notez/personal/shared-repo-1/<file>.md` on laptop
and runs `notez sync` (which commits and pushes the deletion).

Verify:
- After `notez sync` on desktop, the file is gone there too.
- Alice's project repo is not affected (the note was never in it).
- Bob is unaffected (it was never in his clone or notez remote).

### Scenario I. Project moved on one machine

Action: Alice moves `~/repos/shared-repo-1/` to `~/Code/work/shared-repo-1/`
on laptop. The registry still points at the old path.

Verify:
- `notez tree` warns about the missing project but does not crash.
- `notez attach --path ~/Code/work/shared-repo-1 shared-repo-1` updates
  the registry to the new path.
- After that, everything resolves correctly again.
- Other machines are unaffected (their registry is independent).

### Scenario J. Conflict during notez sync

Action: Alice writes a personal note on laptop and on desktop without
syncing in between, then runs `notez sync` on both.

Verify:
- First machine's push succeeds.
- Second machine's `git pull --rebase` surfaces a conflict.
- `notez sync` does not silently lose data; it tells the user to resolve
  the conflict manually in the notez repo and rerun.
- After manual resolution, both notes coexist.

### Scenario K. Public note moves to personal (future `notez mv`)

Action: Alice realizes a public note in `shared-repo-1` should not have
been shared. She runs `notez -p mv "leaked-thoughts" --to personal`.

Verify:
- File leaves `~/repos/shared-repo-1/notez/`.
- File arrives at `~/notez/personal/shared-repo-1/00_quick-notes/`.
- The history in `shared-repo-1`'s git is unchanged (Alice has to
  separately rewrite that history if she wants the leak removed from
  past commits; notez does not do this automatically).
- After `notez sync` + a commit in `shared-repo-1`, the file is gone from
  the public scope and synced via Alice's personal remote.

These scenarios are the acceptance criteria for the storage layer. Each
should eventually have an integration test (probably under `tests/`)
that spins up tempdir fixtures simulating two machines and two users.

## Status

This document describes the target design. Initial implementation focuses on:

1. Cargo.toml and module skeleton
2. Config and registry types with TOML round-trip tests
3. Core abstractions: Scope, Project, NoteSource
4. Commands: `add`, `log`, `mkdir`, `attach`, `detach`, `list` end-to-end
5. Stubs for `tree`, `todo`, `edit`, `search`, `nav`, `sync` (return not-implemented)
6. Panic-safe TUI entry/leave helpers (carried from notez-cli)

The TUI tree and todo browsers come next, ported from notez-cli with the storage layer swapped.
