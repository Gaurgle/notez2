<script lang="ts">
  import type { NoteListItem } from "$lib/types";
  import { SCOPE_META } from "$lib/types";

  let {
    notes,
    selectedPath,
    onSelect,
  }: {
    notes: NoteListItem[];
    selectedPath: string | null;
    onSelect: (note: NoteListItem) => void;
  } = $props();
</script>

<div class="list">
  {#if notes.length === 0}
    <div class="empty">No notes here yet.</div>
  {:else}
    {#each notes as note (note.path)}
      <button
        class="row"
        class:active={note.path === selectedPath}
        onclick={() => onSelect(note)}
      >
        <span class="dot {note.scope}" title={SCOPE_META[note.scope].label}></span>
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
    border-right: 1px solid var(--surface);
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
  .project {
    font-size: 0.68rem;
    color: var(--subtext);
    background: var(--surface);
    padding: 0.05rem 0.35rem;
    border-radius: 0.6rem;
    flex-shrink: 0;
  }
</style>
