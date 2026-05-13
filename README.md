# notez2

A local-first CLI note-taking tool. Cross-machine portable rewrite of [notez-cli](https://github.com/Gaurgle/notez-cli).

## What it is

- Take notes that travel with your projects via the project's own git remote (`-p` public scope) or stay only on your machines (`-l` local scope).
- Default scope (`personal`) puts notes in your own private notez repo, syncing across your machines without ever touching the project repo. Teammates never see them.
- One central CLI and TUI that surfaces everything: local scratch, personal-per-project, public-with-team, and global cross-project notes.
- A full todoz interactive todo manager: tags, subtasks, drag-to-reorder, code TODO scanning. (TUI ports in progress.)
- Cross-machine portable. No OS symlinks. No absolute paths persisted. Per-machine project registry, per-user encrypted-only-by-being-in-your-own-repo private notes.

## Status

Early development. The CLI surface mirrors notez-cli's, but the TUI ports (`tree`, `todoz`) are not yet ready. Working today:

```
notez add        notez log         notez mkdir
notez attach     notez detach      notez list
notez sync       notez setup       notez completions
notez init       notez --help
```

Coming next:

- `notez tree` and `treez` interactive browser
- `notez todo` and `todoz` interactive manager
- `notez mv` for scope migrations
- Migration script from notez-cli

See [DESIGN.md](DESIGN.md) for the full architecture, scope model, and the test-scenario matrix.

## Build

```bash
cargo build --release
./target/release/notez --help
```

133 tests passing.

## License

MIT.
