# notez2

A local-first note-taking tool with a CLI/TUI **and** a native desktop app. Cross-machine portable rewrite of [notez-cli](https://github.com/Gaurgle/notez-cli).

## What it is

- Take notes that travel with your projects via the project's own git remote (`-p` public scope) or stay only on your machines (`-l` local scope).
- Default scope (`personal`) puts notes in your own private notez repo, syncing across your machines without ever touching the project repo. Teammates never see them.
- One model that surfaces everything from the CLI or the desktop app: local scratch, personal-per-project, public-with-team, and global cross-project notes.
- A full todoz todo manager: tags, subtasks, drag-to-reorder.
- Cross-machine portable. No OS symlinks. No absolute paths persisted. Per-machine project registry; private notes stay private by living in your own repo.

## Repository layout

A Cargo workspace plus a Tauri/Svelte frontend:

```
crates/notez-core/   # scopes, aggregation, todoz model, tags (GUI-agnostic)
crates/notez-cli/    # the `notez` binary (+ todoz/zlog symlink dispatch)
app/                 # desktop app: Tauri (Rust) backend + SvelteKit frontend
```

## Desktop app

A native desktop app (Tauri + SvelteKit + TypeScript, CodeMirror 6 editor) that reads and writes the same files as the CLI, byte-for-byte, so notes round-trip through `notez`, `nvim`, and the GUI without spurious diffs.

**Notes**
- Sidebar of scopes (Personal / Public / Local / Global) and registered projects; filter by one scope **or** one project, or view everything.
- Note list with importance-tag dots and a scope/project pill.
- Markdown editor with optional **vim mode** (toggle with the footer pill or `Ctrl+;`, with a NORMAL/INSERT/VISUAL badge) and a separate, resizable, togglable **live preview** pane.
- Right-side inspector (scope, project, path, tags) and a status-bar footer.
- Sort by latest touch / oldest / name, across any scope or project.
- Importance tags (`1`–`5`), vim + arrow-key navigation, `/` and `Cmd/Ctrl+F` to search.

**Todoz**
- Interactive tree todo board: tri-state checkboxes, 5 importance tags, subtasks, sections, drag-to-reorder.
- Full keyboard control mirroring the todoz TUI, `#tag` / `#1` filtering.

All panes are resizable; preview and inspector are independently togglable (`p` / `i`).

### Run / build the app

```bash
cd app
npm install
npm run tauri dev      # dev build with hot reload
npm run tauri build    # production bundle
```

## CLI

The CLI surface mirrors notez-cli's. Working today:

```
notez add        notez log         notez mkdir
notez attach     notez detach      notez list
notez sync       notez setup       notez completions
notez init       notez --help
```

```bash
cargo build --release
./target/release/notez --help
```

See [DESIGN.md](DESIGN.md) for the architecture, scope model, and test-scenario matrix. Core logic is covered by unit tests (`cargo test`).

## License

MIT.
