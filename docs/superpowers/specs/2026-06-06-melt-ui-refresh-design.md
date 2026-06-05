# Melt UI Refresh — Design

**Date:** 2026-06-06
**Branch:** `feat/melt-ui-refresh`
**Status:** Approved (Phase 1)

## Goal

Freshen the desktop app's UI by adopting [Melt UI](https://www.melt-ui.com/)
(the runes-based `melt` package) for behavior + accessibility, layered over the
existing `app.css` glass design system. Add an Apple-style translucent left
pane. Do it in phases, reviewing live via `npm run tauri dev` hot reload.

## Context (current state)

- **Svelte 5** (`svelte: ^5.0.0`), SvelteKit + Vite, Tauri 2 backend.
- Existing "liquid-glass" design system in `app/src/app.css` (CSS custom
  properties: `--glass`, `--accent-*`, `--blur`, etc.).
- The Tauri window is **already** `transparent: true` and `window-vibrancy`
  (`0.6`) is applied in `app/src-tauri/src/lib.rs` with material
  `NSVisualEffectMaterial::HudWindow`.
- **But** `app.css` paints an opaque `body { background: #14141e }`, which
  hides the native vibrancy. The current "glass" is a `backdrop-filter` blur
  against that opaque body, not true glass-over-desktop.
- Components are hand-built Svelte under `app/src/lib/components/`.

## Package strategy

Use **`melt`** (runes, headless) as the base — install with `npm i melt`.
It provides behavior + ARIA only; styling comes from the existing `app.css`
tokens, so components stay on-brand.

`melt` (runes rewrite) ships a **smaller set** than the legacy
`@melt-ui/svelte`. Confirmed available in `melt`: Avatar, Accordion,
Collapsible, Tabs, Toaster, Toggle, Tree, Progress, Popover, Tooltip (plus
Combobox, Dialog, Select, Slider, RadioGroup, PinInput, FileUpload,
SpatialMenu).

Missing from `melt` (legacy-only): Calendar, Checkbox, Switch, Dropdown Menu,
Menubar, Pagination, Tags Input, Toolbar, Toggle Group, Link Preview.

**Decision:** stay on `melt` + hand-roll the *trivial* missing primitives;
do **not** pull in the legacy package in Phase 1.
- **Switch** → build from `melt` `Toggle`.
- **Checkbox** → styled native `<input>` (todoz already owns tri-state logic).
- **Toggle Group** → `melt` `Toggle` × N.

The complex missing ones (Menubar, Pagination, Tags Input, Toolbar, Dropdown)
are deferred to a **Phase 2** decision: `Popover`-based hand-rolls vs.
selectively adding `@melt-ui/svelte`, decided per-item.

## Phase 1 scope

| Component | Source | Lands in |
|---|---|---|
| Glass left pane | CSS + vibrancy material swap | `Sidebar.svelte`, `app.css`, `lib.rs` |
| Switch (from Toggle) | `melt` | Inspector + footer toggles (vim, preview, inspector) |
| Tabs | `melt` | Notes ⇄ Todoz top-level switch |
| Toast (Toaster) | `melt` | Replace ad-hoc `flash()` in `NotesView.svelte` |
| Tree | `melt` | Todoz board (subtask tree) |
| Checkbox | native restyle | Todoz tri-state checkboxes |
| Avatar | `melt` | Sidebar header — machine identity |

### Glass pane (detail)

1. `lib.rs`: swap material `HudWindow` → `Sidebar` (the macOS sidebar material).
2. `app.css`: make `body` background transparent so vibrancy shows; keep the
   faint aurora gradient optional/over-transparent.
3. Content panes (note list, editor, todoz main, inspector) get **solid**
   backgrounds for readability.
4. Left **Sidebar** pane gets a tunable translucent background via a new var
   `--sidebar-glass-alpha` (default ≈ 0.92, "subtle ~90–95%").

### Avatar (detail)

- New Tauri command returning the machine hostname (and derived initials).
- `melt` `Avatar` in the sidebar header, initials as fallback, no image/OAuth.
- Ties into the per-machine identity concept (registry is per-machine).

## Out of scope (Phase 1)

- **Calendar** — captured as a future idea (see `DESIGN.md` → Future Ideas),
  not implemented. Not in `melt` yet; will be hand-rolled or legacy-pkg later.
- GitHub OAuth for the avatar.
- Menubar, Pagination, Tags Input, Toolbar, Dropdown — Phase 2.
- The notes-migration UX simplification (separate, user thinking on it).

## Approach / sequencing

1. Branch `feat/melt-ui-refresh`, `npm i melt`.
2. Glass pane first (visible, low-risk, validates the look).
3. Toast + Tabs (app-wide chrome).
4. Switch/Toggle in inspector + footer.
5. Tree + Checkbox in todoz.
6. Avatar in sidebar header.
7. Review live each step; scope Phase 2 from what's seen.

## Testing

- `npm run check` (svelte-check) must stay clean.
- `cargo build` for the `lib.rs`/command changes.
- Manual: verify hot-reload look per step; confirm vibrancy shows the desktop
  blur subtly behind the sidebar without hurting content readability.
- Existing `cargo test` (core) must stay green (no core logic touched).
