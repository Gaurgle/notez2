<script lang="ts">
  import type { NoteListItem, ProjectInfo, Scope } from "$lib/types";
  import { SCOPE_META } from "$lib/types";

  let {
    notes,
    activeScope,
    activeProject,
    registeredProjects,
    onScope,
    onProject,
    onAttach,
    onDetach,
    onMigrate,
  }: {
    notes: NoteListItem[];
    activeScope: Scope | "all";
    activeProject: string | null;
    registeredProjects: ProjectInfo[];
    onScope: (s: Scope | "all") => void;
    onProject: (p: string | null) => void;
    onAttach: () => void;
    onDetach: (name: string) => void;
    onMigrate: () => void;
  } = $props();

  const scopes: (Scope | "all")[] = ["all", "personal", "public", "local", "global"];

  function countFor(scope: Scope | "all"): number {
    return scope === "all"
      ? notes.length
      : notes.filter((n) => n.scope === scope).length;
  }
</script>

<aside class="sidebar">
  <div class="brand">notez</div>

  <nav class="group">
    <div class="group-label">Scopes</div>
    {#each scopes as scope (scope)}
      <button
        class="item"
        class:active={activeScope === scope}
        onclick={() => onScope(scope)}
      >
        {#if scope !== "all"}<span class="dot {scope}"></span>{/if}
        <span class="item-label">
          {scope === "all" ? "All notes" : SCOPE_META[scope].label}
        </span>
        <span class="count">{countFor(scope)}</span>
      </button>
    {/each}
  </nav>

  <nav class="group">
    <div class="group-head">
      <span class="group-label">Projects</span>
      <button class="mini" title="attach project" aria-label="attach project" onclick={onAttach}>+</button>
    </div>
    {#if registeredProjects.length === 0}
      <div class="muted">none attached</div>
    {:else}
      <button
        class="item"
        class:active={activeProject === null}
        onclick={() => onProject(null)}
      >
        <span class="item-label">All projects</span>
      </button>
      {#each registeredProjects as p (p.name)}
        <div class="proj-row">
          <button
            class="item proj"
            class:active={activeProject === p.name}
            class:dim={!p.reachable}
            onclick={() => onProject(p.name)}
            title={p.local_path}
          >
            <span class="item-label">{p.name}</span>
          </button>
          <button class="x" title="detach" aria-label="detach" onclick={() => onDetach(p.name)}>×</button>
        </div>
      {/each}
    {/if}
    <button class="migrate-link" onclick={onMigrate}>migrate legacy…</button>
  </nav>
</aside>

<style>
  .sidebar {
    background: var(--glass);
    -webkit-backdrop-filter: var(--blur);
    backdrop-filter: var(--blur);
    border-right: 1px solid var(--border);
    padding: 0.5rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .brand {
    font-weight: 700;
    font-size: 1rem;
    color: var(--accent);
    padding: 0.4rem 0.5rem;
  }
  .group-label {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--subtext);
    padding: 0.25rem 0.5rem;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.35rem 0.5rem;
    background: none;
    border: none;
    border-radius: 0.4rem;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    font: inherit;
    font-size: 0.8rem;
  }
  .item:hover { background: var(--surface); }
  .item.active { background: var(--surface-active); }
  .item-label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .count { font-size: 0.68rem; color: var(--subtext); }
  .dot {
    width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0;
  }
  .dot.personal { background: var(--accent-personal); }
  .dot.public { background: var(--accent-public); }
  .dot.local { background: var(--accent-local); }
  .dot.global { background: var(--accent-global); }

  .group-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .mini {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: var(--glass-hover);
    color: var(--subtext);
    cursor: pointer;
    font-weight: 700;
    line-height: 1;
    padding: 0;
  }
  .mini:hover {
    background: var(--glass-active);
    color: var(--text);
  }
  .muted {
    font-size: 0.72rem;
    color: var(--faint);
    padding: 0.25rem 0.5rem;
  }
  .proj-row {
    display: flex;
    align-items: center;
  }
  .proj-row .item {
    flex: 1;
  }
  .proj.dim .item-label {
    color: var(--faint);
  }
  .x {
    background: none;
    border: none;
    color: var(--faint);
    cursor: pointer;
    font-size: 0.95rem;
    padding: 0 0.35rem;
    opacity: 0;
  }
  .proj-row:hover .x {
    opacity: 0.7;
  }
  .x:hover {
    opacity: 1;
    color: var(--danger);
  }
  .migrate-link {
    margin-top: 0.4rem;
    background: none;
    border: none;
    color: var(--faint);
    cursor: pointer;
    font: inherit;
    font-size: 0.7rem;
    text-align: left;
    padding: 0.25rem 0.5rem;
  }
  .migrate-link:hover {
    color: var(--accent);
    text-decoration: underline;
  }
</style>
