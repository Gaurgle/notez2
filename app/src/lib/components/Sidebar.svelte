<script lang="ts">
  import MachineAvatar from "$lib/components/MachineAvatar.svelte";
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
    onHover = () => {},
    width = 220,
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
    onHover?: (item: { kind: "scope" | "project"; name: string } | null) => void;
    width?: number;
  } = $props();

  function countFor(scope: Scope | "all"): number {
    return scope === "all"
      ? notes.length
      : notes.filter((n) => n.scope === scope).length;
  }

  // Hover handlers (spreadable) so hovering a scope/project drives the inspector.
  const navHov = (kind: "scope" | "project", name: string) => ({
    onmouseenter: () => onHover({ kind, name }),
    onmouseleave: () => onHover(null),
  });
</script>

<aside class="sidebar" style="width:{width}px">
  <div class="brand">
    <MachineAvatar />
    <span class="brand-name">notez</span>
  </div>

  <nav class="group">
    <div class="group-label">Scopes</div>
    <button
      class="item"
      class:active={activeProject === null && activeScope === "all"}
      onclick={() => onScope("all")}
      {...navHov("scope", "all")}
    >
      <span class="item-label">All notes</span>
      <span class="count">{countFor("all")}</span>
    </button>
    <button
      class="item"
      class:active={activeProject === null && activeScope === "personal"}
      onclick={() => onScope("personal")}
      {...navHov("scope", "personal")}
    >
      <span class="dot personal"></span>
      <span class="item-label">{SCOPE_META.personal.label}</span>
      <span class="count">{countFor("personal")}</span>
    </button>
    <button
      class="item"
      class:active={activeProject === null && activeScope === "public"}
      onclick={() => onScope("public")}
      {...navHov("scope", "public")}
    >
      <span class="dot public"></span>
      <span class="item-label">{SCOPE_META.public.label}</span>
      <span class="count">{countFor("public")}</span>
    </button>
    <button
      class="item"
      class:active={activeProject === null && activeScope === "local"}
      onclick={() => onScope("local")}
      {...navHov("scope", "local")}
    >
      <span class="dot local"></span>
      <span class="item-label">{SCOPE_META.local.label}</span>
      <span class="count">{countFor("local")}</span>
    </button>
    <button
      class="item"
      class:active={activeProject === null && activeScope === "global"}
      onclick={() => onScope("global")}
      {...navHov("scope", "global")}
    >
      <span class="dot global"></span>
      <span class="item-label">{SCOPE_META.global.label}</span>
      <span class="count">{countFor("global")}</span>
    </button>
  </nav>

  <nav class="group">
    <div class="group-head">
      <span class="group-label">Projects</span>
      <button class="mini" title="attach project" aria-label="attach project" onclick={onAttach}>+</button>
    </div>
    {#if registeredProjects.length === 0}
      <div class="muted">none attached</div>
    {:else}
      {#each registeredProjects as p (p.name)}
        <div class="proj-row">
          <button
            class="item proj"
            class:active={activeProject === p.name}
            class:dim={!p.reachable}
            onclick={() => onProject(p.name)}
            {...navHov("project", p.name)}
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
    flex-shrink: 0;
    background: rgba(20, 20, 32, var(--sidebar-glass-alpha));
    -webkit-backdrop-filter: var(--blur);
    backdrop-filter: var(--blur);
    border-right: 1px solid var(--border);
    padding: 0.5rem;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 700;
    font-size: 1rem;
    color: var(--accent);
    padding: 0.4rem 0.5rem;
  }
  .brand-name {
    color: var(--accent);
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
