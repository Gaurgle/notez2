<script lang="ts">
  import { TAG_DEFS } from "$lib/types";
  import type { TodoTask } from "$lib/types";
  import Avatar from "$lib/components/Avatar.svelte";
  import { mockProjectAuthors } from "$lib/mock";

  let {
    task,
    selected,
    editing,
    prefix = [],
    last = true,
    onSelect,
    onToggle,
    onToggleFlag,
    onToggleCollapse,
    onEditStart,
    onEditCommit,
    onEditCancel,
    onDragStart,
    onDrop,
    onHover,
  }: {
    task: TodoTask;
    selected: boolean;
    editing: boolean;
    prefix?: boolean[];
    last?: boolean;
    onSelect: (id: number) => void;
    onToggle: (id: number) => void;
    onToggleFlag: (id: number, bit: number) => void;
    onToggleCollapse: (id: number) => void;
    onEditStart: (id: number) => void;
    onEditCommit: (id: number, text: string) => void;
    onEditCancel: () => void;
    onDragStart: (id: number) => void;
    onDrop: (id: number) => void;
    onHover: (id: number | null) => void;
  } = $props();

  let draft = $state("");
  let row = $state<HTMLElement>();

  $effect(() => {
    if (editing) draft = task.text;
  });
  $effect(() => {
    if (selected && row) row.scrollIntoView({ block: "nearest" });
  });

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  const mark = $derived(task.state === "checked" ? "✓" : task.state === "half" ? "–" : "");
</script>

<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
<div
  bind:this={row}
  class="row"
  class:header={task.is_header}
  class:selected
  class:done={task.state === "checked"}
  draggable={!task.is_header}
  onclick={() => onSelect(task.id)}
  onkeydown={(e) => {
    if (e.key === "Enter") onSelect(task.id);
  }}
  onmouseenter={() => onHover(task.id)}
  onmouseleave={() => onHover(null)}
  ondragstart={() => onDragStart(task.id)}
  ondragover={(e) => e.preventDefault()}
  ondrop={(e) => {
    e.preventDefault();
    onDrop(task.id);
  }}
>
  <span class="dotcol">
    {#each TAG_DEFS as d (d.bit)}
      {#if task.is_header}
        <span class="dot" class:on={(task.flags & d.bit) !== 0} style="--c:{d.color}"></span>
      {:else}
        <button
          class="dot"
          class:on={(task.flags & d.bit) !== 0}
          style="--c:{d.color}"
          title={d.label}
          aria-label={d.label}
          onclick={(e) => {
            e.stopPropagation();
            onToggleFlag(task.id, d.bit);
          }}
        ></button>
      {/if}
    {/each}
  </span>

  {#if task.is_header}
    <button class="caret" aria-label="collapse" onclick={() => onToggleCollapse(task.id)}>
      {task.collapsed ? "▸" : "▾"}
    </button>
    <span class="htitle">{task.text}</span>
    <span class="hdr-avatars">
      {#each mockProjectAuthors(task.text).slice(0, 4) as a (a)}
        <Avatar name={a} size={16} />
      {/each}
    </span>
  {:else}
    <span class="guides">
      {#each prefix as line, c (c)}
        <span class="g" class:line></span>
      {/each}
      {#if task.depth > 0}
        <span class="g conn" class:last></span>
      {/if}
    </span>

    {#if task.has_subtasks}
      <button class="caret" aria-label="collapse" onclick={() => onToggleCollapse(task.id)}>
        {task.collapsed ? "▸" : "▾"}
      </button>
    {:else}
      <span class="caret-spacer"></span>
    {/if}

    <button
      class="check {task.state}"
      role="checkbox"
      aria-checked={task.state === "checked" ? "true" : task.state === "half" ? "mixed" : "false"}
      aria-label="toggle done"
      onclick={() => onToggle(task.id)}
    >
      {mark}
    </button>

    {#if editing}
      <input
        class="edit"
        bind:value={draft}
        use:focusOnMount
        onblur={() => onEditCommit(task.id, draft)}
        onkeydown={(e) => {
          if (e.key === "Enter") onEditCommit(task.id, draft);
          if (e.key === "Escape") onEditCancel();
          e.stopPropagation();
        }}
      />
    {:else}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="text" ondblclick={() => onEditStart(task.id)}>{task.text}</span>
    {/if}
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0 0.85rem 0 0.6rem;
    /* Fixed height (not min-height) so row spacing is fully deterministic —
       it never shifts with content length, depth, or expand/collapse.
       Matches the notes list row (height + separator). */
    height: 34px;
    border-bottom: 1px solid var(--surface);
    font-size: 0.82rem;
    cursor: default;
  }
  .row:not(.header):hover {
    background: rgba(255, 255, 255, 0.035);
  }
  .row.selected {
    background: color-mix(in srgb, var(--accent) 16%, transparent);
  }

  /* Section headers — same type and height as a note row (just a touch
     bolder), so notes and todos read identically. */
  .row.header {
    height: 34px;
    padding: 0 0.85rem 0 0.6rem;
    color: var(--text);
    font-weight: 600;
    font-size: 0.82rem;
  }
  .row.header.selected {
    background: color-mix(in srgb, var(--accent) 14%, transparent);
  }
  .htitle {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .hdr-avatars {
    display: inline-flex;
    flex-shrink: 0;
    margin-right: 0.2rem;
  }
  .hdr-avatars :global(.avatar:not(:first-child)) {
    margin-left: -6px;
  }

  /* importance tags, left gutter */
  .dotcol {
    display: flex;
    gap: 0.28rem;
    flex-shrink: 0;
    width: 56px;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    border: none;
    background: var(--faint);
    cursor: pointer;
    padding: 0;
    opacity: 0.24;
    transition: opacity 0.12s, transform 0.12s, background 0.12s;
  }
  button.dot:hover {
    opacity: 0.85;
    background: var(--c);
    transform: scale(1.4);
  }
  .dot.on {
    background: var(--c);
    opacity: 1;
    box-shadow: 0 0 6px color-mix(in srgb, var(--c) 55%, transparent);
  }

  /* Continuous CSS tree guides (no ASCII). */
  .guides {
    display: flex;
    align-self: stretch;
    flex-shrink: 0;
  }
  .g {
    width: 17px;
    position: relative;
  }
  .g.line::before {
    content: "";
    position: absolute;
    left: 8px;
    top: 0;
    bottom: 0;
    border-left: 1px solid var(--guide);
  }
  .conn::before {
    /* vertical part of the connector */
    content: "";
    position: absolute;
    left: 8px;
    top: 0;
    bottom: 0;
    border-left: 1px solid var(--guide);
  }
  .conn.last::before {
    bottom: auto;
    height: 50%;
  }
  .conn::after {
    /* horizontal elbow into the row */
    content: "";
    position: absolute;
    left: 8px;
    top: 50%;
    width: 9px;
    border-top: 1px solid var(--guide);
  }

  .caret,
  .caret-spacer {
    width: 22px;
    flex-shrink: 0;
    text-align: center;
  }
  .caret {
    background: none;
    border: none;
    color: var(--subtext);
    cursor: pointer;
    font-size: 1.6rem;
    line-height: 1;
    padding: 0;
  }
  .caret:hover {
    color: var(--accent);
  }
  .row.header .caret {
    color: var(--text);
  }

  .check {
    width: 16px;
    height: 16px;
    border: 1.5px solid var(--surface-active);
    border-radius: 5px;
    background: rgba(0, 0, 0, 0.2);
    color: #11131a;
    cursor: pointer;
    display: grid;
    place-items: center;
    font-size: 0.66rem;
    font-weight: 800;
    flex-shrink: 0;
    padding: 0;
    transition: background 0.12s, border-color 0.12s;
  }
  .check:hover {
    border-color: var(--accent-public);
  }
  .check.checked {
    background: var(--accent-public);
    border-color: var(--accent-public);
  }
  .check.half {
    background: var(--accent-global);
    border-color: var(--accent-global);
  }

  .text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: text;
  }
  .row.done .text {
    color: var(--faint);
  }
  .edit {
    flex: 1;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid var(--accent);
    border-radius: 5px;
    color: var(--text);
    font: inherit;
    font-size: 0.86rem;
    padding: 0.1rem 0.45rem;
  }
  .edit:focus {
    outline: none;
  }
</style>
