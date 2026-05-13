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

In an ideal world the TUI exposes one consistent "move scope" affordance
that works on both notes (in the tree browser) and todos (in todoz),
keeping muscle memory unified across the two views.

## Status

This document describes the target design. Initial implementation focuses on:

1. Cargo.toml and module skeleton
2. Config and registry types with TOML round-trip tests
3. Core abstractions: Scope, Project, NoteSource
4. Commands: `add`, `log`, `mkdir`, `attach`, `detach`, `list` end-to-end
5. Stubs for `tree`, `todo`, `edit`, `search`, `nav`, `sync` (return not-implemented)
6. Panic-safe TUI entry/leave helpers (carried from notez-cli)

The TUI tree and todo browsers come next, ported from notez-cli with the storage layer swapped.
