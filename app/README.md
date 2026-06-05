# notez desktop app

Native desktop app for [notez2](../README.md): Tauri (Rust) backend reusing `notez-core`, SvelteKit + TypeScript frontend, CodeMirror 6 editor. It reads and writes the same files as the `notez` CLI, so notes and todos round-trip without spurious diffs.

## Develop

```bash
npm install
npm run tauri dev      # dev build, hot reload
npm run check          # svelte-check (types + a11y)
npm run tauri build    # production bundle
```

The Rust backend lives in `src-tauri/` (crate `notez-app`); the frontend in `src/`.

## Layout

```
src/lib/ipc.ts                  typed wrappers around Tauri commands
src/lib/types.ts                DTO mirrors of the Rust wire format
src/lib/components/             NotesView, TodozView, NoteEditor, MarkdownPreview,
                                Sidebar, Inspector, Resizer, NoteList, todo/…
src/routes/+page.svelte         shell: Notes / Todos tabs (both kept mounted)
src-tauri/src/commands.rs        #[tauri::command] handlers
src-tauri/src/dto.rs             serde wire DTOs (paths as strings)
```

See the root [README](../README.md) for features and the scope model.
