<script lang="ts">
  import type { NoteListItem, Scope } from "$lib/types";
  import { SCOPE_META } from "$lib/types";

  let {
    notes,
    activeScope,
    activeProject,
    onScope,
    onProject,
  }: {
    notes: NoteListItem[];
    activeScope: Scope | "all";
    activeProject: string | null;
    onScope: (s: Scope | "all") => void;
    onProject: (p: string | null) => void;
  } = $props();

  const scopes: (Scope | "all")[] = ["all", "personal", "public", "local", "global"];

  let projects = $derived(
    [...new Set(notes.map((n) => n.project).filter((p): p is string => !!p))].sort()
  );

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

  {#if projects.length > 0}
    <nav class="group">
      <div class="group-label">Projects</div>
      <button
        class="item"
        class:active={activeProject === null}
        onclick={() => onProject(null)}
      >
        <span class="item-label">All projects</span>
      </button>
      {#each projects as project (project)}
        <button
          class="item"
          class:active={activeProject === project}
          onclick={() => onProject(project)}
        >
          <span class="item-label">{project}</span>
        </button>
      {/each}
    </nav>
  {/if}
</aside>

<style>
  .sidebar {
    background: var(--mantle);
    border-right: 1px solid var(--surface);
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
</style>
