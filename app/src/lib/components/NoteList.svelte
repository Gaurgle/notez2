<script lang="ts">
  import type { NoteListItem } from "$lib/types";
  import { SCOPE_META, TAG_DEFS } from "$lib/types";

  let {
    notes,
    selectedPath,
    onSelect,
    onHover,
  }: {
    notes: NoteListItem[];
    selectedPath: string | null;
    onSelect: (note: NoteListItem) => void;
    onHover: (note: NoteListItem | null) => void;
  } = $props();

  let listEl = $state<HTMLElement>();

  $effect(() => {
    selectedPath; // re-run when selection changes
    listEl?.querySelector(".row.active")?.scrollIntoView({ block: "nearest" });
  });
</script>

<div class="list" bind:this={listEl}>
  {#if notes.length === 0}
    <div class="empty">No notes here yet.</div>
  {:else}
    {#each notes as note (note.path)}
      <button
        class="row"
        class:active={note.path === selectedPath}
        onclick={() => onSelect(note)}
        onmouseenter={() => onHover(note)}
        onmouseleave={() => onHover(null)}
      >
        <span class="dot {note.scope}" title={SCOPE_META[note.scope].label}></span>
        <span class="tagdots">
          {#each TAG_DEFS as d (d.bit)}
            <span
              class="tdot"
              class:on={(note.flags & d.bit) !== 0}
              style="--c:{d.color}"
              title={d.label}
            ></span>
          {/each}
        </span>
        <span class="name">{note.name}</span>
        {#if note.project}
          <span class="project">{note.project}</span>
        {/if}
      </button>
    {/each}
  {/if}
</div>

<style>
  .list {
    overflow-y: auto;
    height: 100%;
    background: rgba(18, 18, 28, 0.92);
    border-right: 1px solid var(--border);
  }
  .empty {
    padding: 1rem;
    color: var(--subtext);
    font-size: 0.82rem;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: none;
    border: none;
    border-bottom: 1px solid var(--surface);
    color: var(--text);
    text-align: left;
    cursor: pointer;
    font: inherit;
  }
  .row:hover {
    background: var(--surface);
  }
  .row.active {
    background: var(--surface-active);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .dot.personal { background: var(--accent-personal); }
  .dot.public { background: var(--accent-public); }
  .dot.local { background: var(--accent-local); }
  .dot.global { background: var(--accent-global); }
  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.82rem;
  }
  .tagdots {
    display: flex;
    gap: 0.22rem;
    flex-shrink: 0;
    width: 56px;
  }
  .tdot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--faint);
    opacity: 0.25;
  }
  .tdot.on {
    background: var(--c);
    opacity: 1;
    box-shadow: 0 0 5px color-mix(in srgb, var(--c) 55%, transparent);
  }
  .project {
    font-size: 0.68rem;
    color: var(--subtext);
    background: var(--surface);
    padding: 0.05rem 0.35rem;
    border-radius: 0.6rem;
    flex-shrink: 0;
  }
</style>
