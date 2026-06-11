# Multi-user sync — research notes (parked for later)

**Status:** exploration, not a committed design. Revisit when we start building
notez2 for **multiple collaborators**. Today the app is effectively single-user
(your own machines syncing via your own remotes); this captures the problem and
the option space so we don't re-derive it later.

---

## The problem

All shared data lives in the repos. **Public**-scope notes/todos live in
`<project>/notez/`, committed into the *project's* git repo. When two people
collaborate via branches, they don't see each other's notes/todos until the
branches merge.

> "if users work on different branches, they will not be able to see other
> notes or todos, since they don't have the updates."

Concretely, putting team notes inside the code tree means they inherit the
code's branching model:

- On a feature branch you don't see public notes added on `main` until you merge.
- A note committed on a branch disappears when you switch branches.
- Uncommitted notes block branch switches / get stashed.
- Two people editing `notez/TODO.md` get a **merge conflict** on team data,
  tangled up with code merges.

## Diagnosis: lifecycle coupling

Code and team-knowledge have **different lifecycles**:

- **Code** — branch-bound, reviewed, merged deliberately.
- **Team notes/todos** — should be *always-current*, visible to all
  collaborators regardless of which branch they're on.

Public scope couples them by storing notes inside the code tree. **That coupling
is the root cause** — not "sync" in the abstract.

Per-scope status (only one scope is actually broken):

| Scope | Lives in | Branch-coupled? | Multi-user intent | Status |
|---|---|---|---|---|
| Local | `<proj>/.notez/` (gitignored) | n/a | never shared | fine |
| Personal | `~/notez/personal/<proj>/` | no (own remote) | private to you | fine |
| Global | `~/notez/` | no (own remote) | private to you | fine |
| **Public** | `<proj>/notez/` (committed) | **yes** | **shared with team** | **broken** |

So the surface area is small — only **Public** scope. But "sync data projects
**and users**" also drags in a second concern the current model barely has:
**identity** — who the collaborators are, and how a note knows who wrote it.

## Three separable sub-problems

Conflating these is what makes it feel huge. They can be solved in stages.

1. **Transport / branch-decoupling** — how shared data travels without
   inheriting code's branches.
2. **Conflict model** — what happens when two people edit the same shared list.
3. **Identity & presence** — who is a "user," attribution, who's online (the
   "and users" half of the ask).

---

## 1. Transport options

### A. Dedicated git ref / orphan branch *(pure git)*

Team data lives on a ref like `refs/notez/data` — an orphan branch with no
shared history with code. The app/CLI read & write it **without checking it
out** into the working tree (a hidden git worktree pinned to that ref, or
reading blobs directly with gitoxide/libgit2). Sync = fetch/push *just that
ref*, regardless of which code branch is checked out.

- ✅ Fully decoupled from code branches; **same remote, same permissions, same
  auth** as the repo; offline / local-first; **no new infra**. Matches DESIGN.md
  goal #3 (sync via plain git).
- ❌ Not real-time (pull-based — could fetch on a timer). Needs ref plumbing the
  CLI doesn't have today; slightly unusual git usage (devs may be surprised by a
  hidden ref).

### B. Companion notes repo per project *(pure git, simplest plumbing)*

Each project gets a sibling repo `<project>-notez`. Team data is a normal `main`
branch there; sync is **identical to how `~/notez` already syncs** — no ref
trickery. The registry maps project → notes-repo path/remote.

- ✅ Dead-simple mechanism (just another clone); reuses existing sync code; easy
  to reason about.
- ❌ One extra repo to **create + grant access per project** (friction; could be
  auto-created via the GitHub API). Two clones to keep track of; discovery.

### C. Sync server / "spaze" *(real-time)*

The backend we already have GitHub device-flow auth for holds team notes/todos;
clients sync over http/ws. Git stops transporting team data.

- ✅ Real-time + presence + the activity feed already planned; conflict
  resolution is far easier server-side (per-field last-write-wins, or CRDT);
  no branch involvement at all.
- ❌ Hosting, auth, uptime, backups; offline becomes "local cache + reconcile";
  biggest departure from the local-first goal and the most lock-in.

### Leaning

**A as the substrate now, shaped so C can front it later.** A removes the daily
pain while keeping every DESIGN.md goal intact (plain git, no infra, one remote,
inherits GitHub permissions). If the *data representation* (below) is designed
well, the spaze server can later serve the **same** data for real-time, with git
as the offline fallback — no rewrite. **B** is the fast-to-ship fallback if ref
plumbing feels too fiddly. **C alone** is the most work and the most lock-in;
only worth it once real-time/presence is a proven need.

---

## 2. Conflict model

The hard tension: **the human-editable Markdown file IS the database.** `todoz`
reads `notez/TODO.md` directly; the desktop app polls it from disk. Two people
editing one Markdown list is a conflict *fundamentally*, no matter the transport.

Options, weakest → strongest:

- **Keep `TODO.md` as source of truth, accept rare conflicts.** Just decouple it
  from code branches (transport A/B). Concurrent edits to the same file can still
  conflict and need manual resolution — but it's isolated to notes, never tangled
  with code merges. Smallest change; honest about the limitation.
- **Per-author partitioning, folded on read.** Each author writes their own file
  (`notez/by/<user>/todos.*`); the shared view is computed by folding all
  authors' files. Disjoint authors → disjoint files → git auto-merges with **no
  conflict**. `TODO.md` becomes a *generated projection* for the CLI/human view,
  not the DB. Editing/checking the *same* item still needs a merge rule
  (last-write-wins per item id).
- **Full CRDT** (per-item LWW-register + an OR-set for list membership/order).
  Strongest — concurrent edits to the same item converge automatically. Most
  code; arguably overkill for a notes tool.

**Knock-on requirement:** anything past option 1 needs **stable IDs per todo
item**. Markdown checklists have no ids today — adding them (e.g. a trailing
`<!-- id:... -->` or a sidecar) is itself a small design task, and the CLI must
learn to preserve them.

**The CLI is the real constraint here.** If storage moves from Markdown to
structured per-author records, `todoz`/`notez` (file-based, read `TODO.md`
directly) must change in lockstep, or read a generated `TODO.md` projection.
Keep CLI/desktop parity front-of-mind for any change past option 1.

---

## 3. Identity & users

Today notez has **no real user model** — dashboard attribution is mock data.
Multi-user needs:

- **Who is a collaborator** (read/write public scope for a project). The git
  approaches (A/B) get this **for free** — repo access *is* the ACL, inherited
  from GitHub. A server (C) has to build its own authz.
- **Stable author identity for attribution.** GitHub identity (the device-flow
  auth already planned in spaze) is the natural source; stamp it on each record
  / per-author file.
- **Presence / live activity** (who's online, recent edits). Only the server
  (C) gives true real-time; the git approaches give coarse "last-synced"
  activity from commit/ref timestamps.

This is why **transport and identity are linked**: pure-git approaches piggyback
on GitHub repo permissions (no authz to build); a server means building authz
and identity from scratch (but unlocks presence).

---

## A phased path (one way to sequence it)

Each step is independently shippable and useful on its own:

1. **Decouple Public from code branches** via a dedicated ref (A). Keep
   `TODO.md` as-is; accept rare conflicts. Removes the daily pain with the
   smallest change.
2. **Stable item ids + per-author partitioning** so concurrent todo edits stop
   conflicting; `TODO.md` becomes a projection. CLI updated in lockstep.
3. **GitHub identity for attribution** (reuse spaze device-flow); real author
   names replace the mock data.
4. **(Only if real-time/presence becomes a real need)** introduce the spaze
   server fronting the *same* data model; git stays the offline path.

## Open questions for when we pick this up

- Do collaborators always share the project's GitHub remote, or do some need
  **notes-only** access? (decides ref **A** vs companion-repo **B** — a
  notes-only repo can have different collaborators than the code.)
- Is real-time actually needed, or is **fetch-on-interval** enough? (decides
  whether the server **C** is ever worth its cost.)
- Should personal notes ever be **promoted to public** (personal → shared), and
  how does that interact with attribution and history?
- How do the **CLI (file-based) and desktop app** stay consistent if storage
  moves from Markdown to structured per-author records? The CLI reads `TODO.md`
  directly — it's the binding constraint.
- **Windows**: behavior of a hidden-ref worktree / gitoxide blob reads on
  Windows (DESIGN.md keeps Windows in mind but non-blocking).
- Does "sync **projects**" (the registry / `.notez-config.toml`) also need to
  become multi-user, or stays per-user as today? Project *metadata* sharing is a
  smaller sibling of the notes problem and might ride the same transport.
