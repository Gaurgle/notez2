<script lang="ts">
  import type { NoteListItem, SearchHit } from "$lib/types";
  import { SCOPE_META, TAG_DEFS } from "$lib/types";

  let {
    notes,
    selectedPath,
    onSelect,
    onHover,
    inProjectMode = false,
    snippets = null,
  }: {
    notes: NoteListItem[];
    selectedPath: string | null;
    onSelect: (note: NoteListItem) => void;
    onHover: (note: NoteListItem | null) => void;
    inProjectMode?: boolean;
    /** Content search hits keyed by path; matching rows show their snippet. */
    snippets?: Map<string, SearchHit> | null;
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
    {#each notes as note, i (note.path + " " + i)}
      <button
        class="row"
        class:active={note.path === selectedPath}
        class:has-snippet={!!snippets?.get(note.path)?.snippet}
        onclick={() => onSelect(note)}
        onmouseenter={() => onHover(note)}
        onmouseleave={() => onHover(null)}
      >
        {#if snippets}
          <!-- Search mode: name + where the term was found; scope and
               urgency live in the inspector. Pills stack on the right. -->
          <span class="searchrow">
            <span class="searchmain">
              <span class="name">{note.name}</span>
              {#if snippets.get(note.path)?.snippet}
                {@const hit = snippets.get(note.path)!}
                <span class="hitinfo">
                  found at line {hit.line}{#if hit.match_count > 1}
                    · +{hit.match_count - 1} more{/if}
                </span>
              {/if}
            </span>
            <span class="pillstack">
              {#if note.kind === "doc"}
                <span class="pill docs">docs</span>
              {/if}
              {#if inProjectMode}
                <span class="pill {note.scope}">{SCOPE_META[note.scope].pill}</span>
              {:else if note.project}
                <span class="project">{note.project}</span>
              {/if}
            </span>
          </span>
        {:else}
          <span class="rowmain">
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
            {#if note.kind === "doc"}
              <span class="pill docs">docs</span>
            {/if}
            {#if inProjectMode}
              <span class="pill {note.scope}">{SCOPE_META[note.scope].pill}</span>
            {:else if note.project}
              <span class="project">{note.project}</span>
            {/if}
          </span>
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
    flex-direction: column;
    gap: 0.15rem;
    width: 100%;
    padding: 0.6rem 0.75rem;
    background: none;
    border: none;
    border-bottom: 1px solid var(--surface);
    color: var(--text);
    text-align: left;
    cursor: pointer;
    font: inherit;
  }
  .rowmain {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
  }
  .row.has-snippet {
    padding-top: 0.65rem;
    padding-bottom: 0.65rem;
    gap: 0.3rem;
  }
  .searchrow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    width: 100%;
  }
  .searchmain {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    flex: 1;
    min-width: 0;
  }
  .hitinfo {
    font-size: 0.7rem;
    color: var(--subtext);
    font-variant-numeric: tabular-nums;
  }
  .pillstack {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.2rem;
    flex-shrink: 0;
  }
  .row:hover {
    background: var(--surface);
  }
  .row.active {
    background: color-mix(in srgb, var(--accent) 15%, transparent);
    box-shadow: inset 2px 0 0 var(--accent);
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
  .pill {
    font-size: 0.6rem;
    padding: 0.05rem 0.42rem;
    border-radius: 0.6rem;
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .pill.personal {
    color: var(--accent-personal);
    background: color-mix(in srgb, var(--accent-personal) 16%, transparent);
  }
  .pill.public {
    color: var(--accent-public);
    background: color-mix(in srgb, var(--accent-public) 16%, transparent);
  }
  .pill.local {
    color: var(--accent-local);
    background: color-mix(in srgb, var(--accent-local) 16%, transparent);
  }
  .pill.global {
    color: var(--accent-global);
    background: color-mix(in srgb, var(--accent-global) 16%, transparent);
  }
  .pill.docs {
    color: var(--accent-public);
    background: none;
    border: 1px dashed color-mix(in srgb, var(--accent-public) 45%, transparent);
  }
  .project {
    font-size: 0.68rem;
    color: var(--subtext);
    background: var(--surface);
    padding: 0.05rem 0.35rem;
    border-radius: 0.6rem;
    flex-shrink: 0;
    max-width: 9rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
