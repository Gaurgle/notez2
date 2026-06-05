# Melt UI Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Freshen the notez2 desktop UI by adopting the runes-based `melt` package for behavior/accessibility, add an Apple-style translucent left pane, all reviewed live via hot reload.

**Architecture:** `melt` is headless — its builder classes expose getters that return attribute objects you spread onto elements (`{...toggle.trigger}`) plus reactive state (`toggle.value`). Styling stays on the existing `app/src/app.css` glass tokens. The glass pane is a CSS + macOS vibrancy-material change; the window is already transparent with `window-vibrancy` applied.

**Tech Stack:** Svelte 5 (runes), SvelteKit, Vite, Tauri 2 (Rust), `melt` (npm), `window-vibrancy` (Rust crate, already a dep).

**Verification note:** This repo has no frontend test runner; frontend tasks are verified with `npm run check` (svelte-check, must stay clean) plus manual observation in `npm run tauri dev` hot reload. Rust tasks are verified with `cargo build` and existing `cargo test`. This is intentional — we don't add a test harness in this pass.

**Reference — real `melt` builder API (from package source):**
- `new Toggle({ value: () => x, onValueChange: (v) => x = v })` → `.value` (bool), spread `{...toggle.trigger}`.
- `new Toaster()` → `.toasts` (array), `.addToast({ data })`; each toast: `{...toast.root}`, `{...toast.content}`, `{...toast.title}`, `{...toast.close}`.
- `new Avatar({ src: () => url })` → `{...avatar.image}`, `{...avatar.fallback}`, `.loadingStatus`.
- `new Tabs({ value: () => x, onValueChange: (v) => x = v })` → `{...tabs.triggerList}`, `tabs.getTrigger(v)`, `tabs.getContent(v)`.
- `new Tree({ items, selected, expanded })` → `{...tree.root}`, `tree.children` (each child `.attrs`, `.item`).

---

## File Structure

- `app/package.json` — add `melt` dependency.
- `app/src-tauri/src/lib.rs` — swap vibrancy material; register `machine_name` command.
- `app/src-tauri/src/commands.rs` — add `machine_name` command.
- `app/src/lib/ipc.ts` — bind `machineName()`.
- `app/src/app.css` — transparent body, `--sidebar-glass-alpha` var, melt-based component styling hooks.
- `app/src/lib/components/NotesView.svelte` — Toaster, Toggle switches (Preview/Inspector/Vim).
- `app/src/lib/components/Sidebar.svelte` — Avatar in brand header, glass background.
- `app/src/routes/+page.svelte` — rail → melt Tabs (vertical).
- `app/src/lib/components/todo/TodoItem.svelte` — checkbox a11y.
- `app/src/lib/components/TodozView.svelte` — Tree keyboard-nav spike (optional, gated).

---

## Task 1: Install `melt` and verify it builds

**Files:**
- Modify: `app/package.json`

- [ ] **Step 1: Install the package**

Run from `app/`:
```bash
cd ~/Repos/notez2/app && npm i melt
```
Expected: `melt` added to `dependencies` in `package.json`.

- [ ] **Step 2: Smoke-test the import in a scratch check**

Add a temporary import at the top of `app/src/routes/+page.svelte` `<script>`:
```ts
import { Toggle } from "melt/builders";
const _smoke = new Toggle();
```

- [ ] **Step 3: Verify type-check passes**

Run: `cd ~/Repos/notez2/app && npm run check`
Expected: 0 errors (the import resolves).

- [ ] **Step 4: Remove the smoke import**

Delete the two `_smoke` lines from `+page.svelte`.

- [ ] **Step 5: Commit**

```bash
git add app/package.json app/package-lock.json
git commit -m "build(app): add melt UI package"
```

---

## Task 2: Glass left pane (vibrancy material + transparent body)

**Files:**
- Modify: `app/src-tauri/src/lib.rs:22`
- Modify: `app/src/app.css:56-68`
- Modify: `app/src/lib/components/Sidebar.svelte` (`.sidebar` background)

- [ ] **Step 1: Switch the macOS vibrancy material to Sidebar**

In `app/src-tauri/src/lib.rs`, change the material from `HudWindow` to `Sidebar`:
```rust
apply_vibrancy(
    &window,
    NSVisualEffectMaterial::Sidebar,
    Some(NSVisualEffectState::Active),
    None,
)
.expect("failed to apply window vibrancy");
```

- [ ] **Step 2: Make the body transparent so vibrancy shows**

In `app/src/app.css`, replace the opaque base background block (currently `html, body { background: #14141e; }` and the `body` gradient block) with:
```css
/* Transparent base so the native macOS vibrancy shows through. */
html,
body {
  background: transparent;
}

body {
  position: relative;
  background-image:
    radial-gradient(48% 58% at 12% 14%, rgba(203, 166, 247, 0.06), transparent 70%),
    radial-gradient(48% 58% at 88% 12%, rgba(137, 180, 250, 0.05), transparent 70%);
  background-attachment: fixed;
}
```

- [ ] **Step 3: Add the tunable sidebar glass alpha token**

In `app/src/app.css` `:root`, add near the surface tokens:
```css
  /* Left pane translucency over native vibrancy. 1 = opaque, lower = glassier. */
  --sidebar-glass-alpha: 0.92;
```

- [ ] **Step 4: Apply the alpha to the sidebar background**

In `app/src/lib/components/Sidebar.svelte` `<style>`, set the `.sidebar` background to:
```css
  .sidebar {
    /* existing rules unchanged; only the background line changes */
    background: rgba(20, 20, 32, var(--sidebar-glass-alpha));
    -webkit-backdrop-filter: var(--blur);
    backdrop-filter: var(--blur);
  }
```
(Keep the rest of `.sidebar`'s existing rules — border, padding, layout.)

- [ ] **Step 5: Give content panes solid backgrounds (readability)**

Verify the note list / editor / todoz columns read on an opaque surface. In `app/src/app.css`, ensure `.viewbar` and main content keep `var(--glass)` (already ~0.94 opaque). If the editor column shows desktop bleed-through, add to the relevant column rule in `NotesView.svelte`: `background: var(--base);`.

- [ ] **Step 6: Verify build + look**

Run: `cd ~/Repos/notez2/app && npm run check` → 0 errors.
Run: `npm run tauri dev` and observe: the left sidebar subtly shows the blurred desktop behind it; note text stays fully readable. Tweak `--sidebar-glass-alpha` (0.85–0.95) live until it matches the "~92%" target.

- [ ] **Step 7: Commit**

```bash
git add app/src-tauri/src/lib.rs app/src/app.css app/src/lib/components/Sidebar.svelte
git commit -m "feat(app): translucent sidebar over native macOS vibrancy"
```

---

## Task 3: Replace ad-hoc toast with melt Toaster

**Files:**
- Modify: `app/src/lib/components/NotesView.svelte` (toast state `:113`, viewbar toast span `:384`, `flash()` helper)

- [ ] **Step 1: Add the Toaster builder**

In `NotesView.svelte` `<script>`, add the import and instance, and replace the `let toast = $state<string | null>(null);` line:
```ts
import { Toaster } from "melt/builders";
const toaster = new Toaster<{ message: string }>({ closeDelay: 2500 });
```

- [ ] **Step 2: Point `flash()` at the toaster**

Find the existing `flash(...)` helper (the one that sets `toast`) and replace its body with:
```ts
function flash(message: string) {
  toaster.addToast({ data: { message } });
}
```

- [ ] **Step 3: Remove the inline toast span**

Delete the viewbar line `{#if toast}<span class="toast">{toast}</span>{/if}` (around `:384`).

- [ ] **Step 4: Render the toast region**

Near the end of the template (after the closing `</div>` of `.notes`, before the dialogs), add:
```svelte
<div class="toaster" {...toaster.root}>
  {#each toaster.toasts as toast (toast.id)}
    <div class="toast-card" {...toast.content}>
      <span {...toast.title}>{toast.data.message}</span>
      <button class="toast-x" {...toast.close} aria-label="dismiss">×</button>
    </div>
  {/each}
</div>
```

- [ ] **Step 5: Style the toaster**

In `NotesView.svelte` `<style>` (or `app.css`), add:
```css
  .toaster {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    z-index: 50;
  }
  .toast-card {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background: var(--glass-strong);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow);
    padding: 0.55rem 0.8rem;
    font-size: 0.8rem;
    color: var(--text);
  }
  .toast-x {
    background: none;
    border: none;
    color: var(--faint);
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
  }
```

- [ ] **Step 6: Verify**

Run: `npm run check` → 0 errors.
In `npm run tauri dev`: trigger a sync/save/attach and confirm a toast slides in bottom-right and auto-dismisses; the old inline viewbar text is gone.

- [ ] **Step 7: Commit**

```bash
git add app/src/lib/components/NotesView.svelte app/src/app.css
git commit -m "feat(app): toast notifications via melt Toaster"
```

---

## Task 4: Preview / Inspector / Vim toggles as melt switches

**Files:**
- Modify: `app/src/lib/components/NotesView.svelte` (viewbar buttons `:385-390`, vim pill `:458-465`)

- [ ] **Step 1: Create Toggle builders bound to existing state**

In `NotesView.svelte` `<script>`, add:
```ts
import { Toggle } from "melt/builders";
const previewToggle = new Toggle({ value: () => showPreview, onValueChange: (v) => (showPreview = v) });
const inspectorToggle = new Toggle({ value: () => showInspector, onValueChange: (v) => (showInspector = v) });
const vimToggle = new Toggle({ value: () => vimMode, onValueChange: (v) => (vimMode = v) });
```

- [ ] **Step 2: Replace the Preview button**

Replace the existing Preview `<button class="ghost" class:on={showPreview} ...>` with:
```svelte
<button class="switch" class:on={previewToggle.value} {...previewToggle.trigger} title="Preview (p)">
  <span class="knob"></span>Preview
</button>
```

- [ ] **Step 3: Replace the Inspector button**

Replace the Inspector `<button class="ghost" class:on={showInspector} ...>` with:
```svelte
<button class="switch" class:on={inspectorToggle.value} {...inspectorToggle.trigger} title="Inspector (i)">
  <span class="knob"></span>Inspector
</button>
```

- [ ] **Step 4: Replace the Vim pill**

Replace the `<button class="vim-pill" ...>` in the statusbar with:
```svelte
<button class="switch" class:on={vimToggle.value} {...vimToggle.trigger} title="Toggle vim mode (Ctrl+;)">
  <span class="knob"></span>VIM
</button>
```

- [ ] **Step 5: Add switch styling**

In `app.css`, add a reusable switch pill:
```css
  button.switch {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    border-radius: 999px;
    padding: 0.32rem 0.7rem 0.32rem 0.4rem;
    font: inherit;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    background: var(--glass-hover);
    border: 1px solid var(--border);
    color: var(--subtext);
    transition: background 0.15s, color 0.15s;
  }
  button.switch .knob {
    width: 26px;
    height: 15px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.14);
    position: relative;
    transition: background 0.15s;
  }
  button.switch .knob::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: var(--text);
    transition: transform 0.15s;
  }
  button.switch.on {
    color: var(--accent);
    border-color: color-mix(in srgb, var(--accent) 40%, transparent);
  }
  button.switch.on .knob {
    background: color-mix(in srgb, var(--accent) 70%, transparent);
  }
  button.switch.on .knob::after {
    transform: translateX(11px);
  }
```

- [ ] **Step 6: Verify**

Run: `npm run check` → 0 errors.
In dev: each switch flips its knob, toggles the pane/mode, keeps keyboard shortcuts (`p`, `i`, `Ctrl+;`) working, and exposes `aria-pressed` (melt's `trigger`). Confirm `Space`/`Enter` on a focused switch toggles it.

- [ ] **Step 7: Commit**

```bash
git add app/src/lib/components/NotesView.svelte app/src/app.css
git commit -m "feat(app): preview/inspector/vim controls as accessible switches"
```

---

## Task 5: Machine-identity Avatar in the sidebar header

**Files:**
- Modify: `app/src-tauri/src/commands.rs`
- Modify: `app/src-tauri/src/lib.rs` (register command)
- Modify: `app/src/lib/ipc.ts`
- Modify: `app/src/lib/components/Sidebar.svelte` (brand header `:37`)

- [ ] **Step 1: Add the `machine_name` Tauri command**

In `app/src-tauri/src/commands.rs`, add:
```rust
/// The current machine's hostname, used as a per-machine identity label.
#[tauri::command]
pub fn machine_name() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "this machine".to_string())
}
```
If the `hostname` crate is not already a dependency, instead read the env without a new dep:
```rust
#[tauri::command]
pub fn machine_name() -> String {
    std::env::var("HOST")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "this machine".to_string())
}
```
Prefer the env version to avoid adding a dependency (ask the user before adding `hostname`).

- [ ] **Step 2: Register the command**

In `app/src-tauri/src/lib.rs`, add `commands::machine_name,` to the `tauri::generate_handler![...]` list.

- [ ] **Step 3: Bind it in ipc.ts**

In `app/src/lib/ipc.ts`, add:
```ts
export const machineName = () => invoke<string>("machine_name");
```

- [ ] **Step 4: Render the Avatar in the brand header**

In `Sidebar.svelte` `<script>`, add:
```ts
import { Avatar } from "melt/builders";
import { machineName } from "$lib/ipc";
import { onMount } from "svelte";
let host = $state("");
const avatar = new Avatar({ src: () => "" }); // no image — initials fallback
const initials = $derived(
  host.replace(/\.local$/, "").slice(0, 2).toUpperCase() || "··"
);
onMount(async () => { host = await machineName(); });
```
Replace `<div class="brand">notez</div>` with:
```svelte
<div class="brand">
  <span class="avatar" {...avatar.fallback} title={host}>{initials}</span>
  <span class="brand-name">notez</span>
</div>
```

- [ ] **Step 5: Style the avatar**

In `Sidebar.svelte` `<style>`:
```css
  .brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .avatar {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 0.66rem;
    font-weight: 700;
    color: #1a1626;
    background: linear-gradient(180deg, var(--accent), #b48ceb);
    flex-shrink: 0;
  }
```

- [ ] **Step 6: Verify**

Run: `cd app/src-tauri && cargo build` → compiles.
Run: `cd ~/Repos/notez2/app && npm run check` → 0 errors.
In dev: the sidebar header shows a circular avatar with this machine's initials; hovering shows the full hostname.

- [ ] **Step 7: Commit**

```bash
git add app/src-tauri/src/commands.rs app/src-tauri/src/lib.rs app/src/lib/ipc.ts app/src/lib/components/Sidebar.svelte
git commit -m "feat(app): machine-identity avatar in sidebar header"
```

---

## Task 6: Convert the Notes/Todos rail to melt Tabs (vertical) — evaluate

**Files:**
- Modify: `app/src/routes/+page.svelte`

- [ ] **Step 1: Add a Tabs builder bound to `mode`**

In `+page.svelte` `<script>`:
```ts
import { Tabs } from "melt/builders";
const tabs = new Tabs<"notes" | "todoz">({
  value: () => mode,
  onValueChange: (v) => (mode = v),
  orientation: "vertical",
});
```

- [ ] **Step 2: Replace the rail buttons with tab triggers**

Replace the two `<button class="rail-btn" ...>` elements inside `<nav class="rail">` with:
```svelte
<div {...tabs.triggerList}>
  <button class="rail-btn" class:active={mode === "notes"} {...tabs.getTrigger("notes")} title="Notes">
    <span class="glyph">✎</span><span class="label">Notes</span>
  </button>
  <button class="rail-btn" class:active={mode === "todoz"} {...tabs.getTrigger("todoz")} title="Todos">
    <span class="glyph">☑</span><span class="label">Todos</span>
  </button>
</div>
```
Keep the existing `Ctrl+Tab` handler — melt adds arrow-key roving focus on top.

- [ ] **Step 3: Verify**

Run: `npm run check` → 0 errors.
In dev: clicking and arrow-keys switch Notes/Todos; `Ctrl+Tab` still works; the active style is intact.

- [ ] **Step 4: Decision gate**

If vertical Tabs add real value (keyboard nav, ARIA tablist) without regressions, keep. If it feels like ceremony over the working rail, revert this task's diff (`git checkout -- app/src/routes/+page.svelte`) and note it as "skipped — rail was already sufficient". Either way, record the decision in the PR description.

- [ ] **Step 5: Commit (if kept)**

```bash
git add app/src/routes/+page.svelte
git commit -m "feat(app): notes/todos rail as accessible vertical tabs"
```

---

## Task 7: Accessible tri-state checkbox in todoz

**Files:**
- Modify: `app/src/lib/components/todo/TodoItem.svelte:121-123`

- [ ] **Step 1: Add ARIA to the existing check control**

The existing `.check` button already renders tri-state (`checked`/`half`/empty). Add proper checkbox semantics. Replace the check button:
```svelte
<button
  class="check {task.state}"
  role="checkbox"
  aria-checked={task.state === "checked" ? "true" : task.state === "half" ? "mixed" : "false"}
  aria-label="toggle done"
  onclick={() => onToggle(task.id)}
>
  {mark}
</button>
```

- [ ] **Step 2: Verify**

Run: `npm run check` → 0 errors.
In dev: toggling still cycles state; a screen reader / the accessibility inspector reports `checkbox` with `mixed` for half state. Visuals unchanged.

- [ ] **Step 3: Commit**

```bash
git add app/src/lib/components/todo/TodoItem.svelte
git commit -m "a11y(app): tri-state todo checkbox semantics"
```

---

## Task 8: melt Tree keyboard navigation in todoz — spike (optional, gated)

**Files:**
- Modify: `app/src/lib/components/TodozView.svelte`

> **Risk:** TodozView owns rich custom state (drag-reorder, per-row flags, inline edit, collapse). A full swap to melt `Tree` rendering would jeopardize those. This task is a **non-destructive spike**: wire melt `Tree` for keyboard navigation + expand/collapse semantics only, keeping the existing `TodoItem` rendering and drag.

- [ ] **Step 1: Read the current TodozView structure**

Read `app/src/lib/components/TodozView.svelte` fully. Identify: the flat task array, how `selected`/collapse are tracked, and the keydown handler. Map melt `Tree` items to the existing tasks (`id` as the tree item id, `has_subtasks`/`collapsed` → expanded set).

- [ ] **Step 2: Construct a Tree from the task list**

```ts
import { Tree } from "melt/builders";
const tree = new Tree({
  items: () => taskTreeItems,           // derived: [{ id: String(t.id), children: [...] }]
  selected: () => selectedId == null ? undefined : String(selectedId),
  onSelectedChange: (v) => (selectedId = v == null ? null : Number(v)),
  expanded: () => expandedIds,          // Set<string> of non-collapsed header/subtask ids
});
```
Build `taskTreeItems` as a `$derived` nested structure from the flat list using `depth`/`has_subtasks`.

- [ ] **Step 3: Attach tree semantics without replacing rendering**

Spread `{...tree.root}` on the scroll container and, for each rendered `TodoItem` row wrapper, the matching `tree.children[i].attrs` for roving tabindex + ARIA `treeitem`. Keep `TodoItem`'s own click/drag handlers.

- [ ] **Step 4: Verify**

Run: `npm run check` → 0 errors.
In dev: arrow keys move selection through the tree; left/right collapse/expand headers; existing click, drag-reorder, flag dots, and inline edit all still work.

- [ ] **Step 5: Decision gate**

If the spike integrates cleanly and improves keyboard nav, keep it. If it fights the existing state model, revert (`git checkout -- app/src/lib/components/TodozView.svelte`) and record "Tree deferred to Phase 2 — conflicts with custom drag/edit state". Do not force it.

- [ ] **Step 6: Commit (if kept)**

```bash
git add app/src/lib/components/TodozView.svelte
git commit -m "feat(app): keyboard tree navigation in todoz via melt Tree"
```

---

## Phase 2 (not in this plan — scope after live review)

Decide per-item between `Popover`-based hand-rolls and selectively adding `@melt-ui/svelte`: Menubar, Pagination, Tags Input, Toolbar, Dropdown Menu, Accordion (scope-as-accordion option), Progress (sync progress), Collapsible. Calendar stays deferred (see `DESIGN.md` → Future Ideas).

## Done criteria

- `npm run check` clean; `cargo build` + `cargo test` green.
- Sidebar visibly translucent over the desktop, content readable.
- Toasts, switches, avatar working; tabs/tree/checkbox kept-or-explicitly-deferred with the decision recorded.
