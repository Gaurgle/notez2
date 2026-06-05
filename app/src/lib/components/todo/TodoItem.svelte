<script lang="ts">
  import { TAG_DEFS } from "$lib/types";
  import type { TodoTask } from "$lib/types";

  let {
    task,
    onToggle,
    onToggleFlag,
    onToggleCollapse,
    onRemove,
    onEdit,
    onAdd,
    onDragStart,
    onDrop,
  }: {
    task: TodoTask;
    onToggle: (id: number) => void;
    onToggleFlag: (id: number, bit: number) => void;
    onToggleCollapse: (id: number) => void;
    onRemove: (id: number) => void;
    onEdit: (id: number, text: string) => void;
    onAdd: (afterId: number, depth: number) => void;
    onDragStart: (id: number) => void;
    onDrop: (id: number) => void;
  } = $props();

  let editing = $state(false);
  let draft = $state("");

  function startEdit() {
    if (task.is_header) return;
    draft = task.text;
    editing = true;
  }
  function commit() {
    editing = false;
    if (draft.trim() && draft.trim() !== task.text) onEdit(task.id, draft.trim());
  }
  function focusOnMount(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  const mark = $derived(
    task.state === "checked" ? "✔" : task.state === "half" ? "◓" : ""
  );
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="row"
  class:header={task.is_header}
  style="padding-left: {0.5 + task.depth * 1.1}rem"
  draggable={!task.is_header}
  ondragstart={() => onDragStart(task.id)}
  ondragover={(e) => e.preventDefault()}
  ondrop={(e) => {
    e.preventDefault();
    onDrop(task.id);
  }}
>
  {#if task.is_header}
    <button class="caret" aria-label="collapse" onclick={() => onToggleCollapse(task.id)}>
      {task.collapsed ? "▸" : "▾"}
    </button>
    <span class="htitle">{task.text}</span>
    <span class="dots">
      {#each TAG_DEFS as d (d.bit)}
        {#if (task.flags & d.bit) !== 0}
          <span class="dot on" style="--c:{d.color}"></span>
        {/if}
      {/each}
    </span>
    <button class="add" title="add task" aria-label="add task" onclick={() => onAdd(task.id, 0)}>+</button>
  {:else}
    {#if task.has_subtasks}
      <button class="caret" aria-label="collapse" onclick={() => onToggleCollapse(task.id)}>
        {task.collapsed ? "▸" : "▾"}
      </button>
    {:else}
      <span class="caret-spacer"></span>
    {/if}

    <button class="check {task.state}" aria-label="toggle done" onclick={() => onToggle(task.id)}>
      {mark}
    </button>

    {#if editing}
      <input
        class="edit"
        bind:value={draft}
        use:focusOnMount
        onblur={commit}
        onkeydown={(e) => {
          if (e.key === "Enter") commit();
          if (e.key === "Escape") editing = false;
        }}
      />
    {:else}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="text" class:done={task.state === "checked"} ondblclick={startEdit}>
        {task.text}
      </span>
    {/if}

    <span class="dots">
      {#each TAG_DEFS as d (d.bit)}
        <button
          class="dot"
          class:on={(task.flags & d.bit) !== 0}
          style="--c:{d.color}"
          title={d.label}
          aria-label={d.label}
          onclick={() => onToggleFlag(task.id, d.bit)}
        ></button>
      {/each}
    </span>

    <button
      class="add"
      title="add subtask"
      aria-label="add subtask"
      onclick={() => onAdd(task.id, Math.min(task.depth + 1, 2))}
    >+</button>
    <button class="del" title="remove" aria-label="remove" onclick={() => onRemove(task.id)}>✕</button>
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.3rem 0.75rem;
    border-bottom: 1px solid color-mix(in srgb, var(--surface) 50%, transparent);
    font-size: 0.82rem;
  }
  .row:hover {
    background: color-mix(in srgb, var(--surface) 40%, transparent);
  }
  .row.header {
    background: var(--mantle);
    font-weight: 600;
    color: var(--subtext);
    text-transform: uppercase;
    font-size: 0.7rem;
    letter-spacing: 0.04em;
  }
  .caret,
  .caret-spacer {
    width: 16px;
    flex-shrink: 0;
  }
  .caret {
    background: none;
    border: none;
    color: var(--subtext);
    cursor: pointer;
    font-size: 0.7rem;
    padding: 0;
  }
  .htitle {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .check {
    width: 18px;
    height: 18px;
    border: 1.5px solid var(--surface-active);
    border-radius: 0.3rem;
    background: var(--base);
    color: var(--accent-public);
    cursor: pointer;
    display: grid;
    place-items: center;
    font-size: 0.7rem;
    flex-shrink: 0;
    padding: 0;
  }
  .check.checked {
    border-color: var(--accent-public);
  }
  .check.half {
    color: var(--accent-global);
    border-color: var(--accent-global);
  }
  .text {
    flex: 1;
    cursor: text;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .text.done {
    color: var(--subtext);
    text-decoration: line-through;
  }
  .edit {
    flex: 1;
    background: var(--base);
    border: 1px solid var(--accent);
    border-radius: 0.3rem;
    color: var(--text);
    font: inherit;
    font-size: 0.82rem;
    padding: 0.15rem 0.35rem;
  }
  .edit:focus {
    outline: none;
  }
  .dots {
    display: flex;
    gap: 0.25rem;
    flex-shrink: 0;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 1px solid var(--c);
    background: transparent;
    cursor: pointer;
    padding: 0;
    opacity: 0.4;
  }
  .dot.on {
    background: var(--c);
    opacity: 1;
  }
  .add,
  .del {
    background: none;
    border: none;
    color: var(--subtext);
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0 0.2rem;
    opacity: 0;
    flex-shrink: 0;
  }
  .row:hover .add,
  .row:hover .del {
    opacity: 0.7;
  }
  .add:hover,
  .del:hover {
    opacity: 1;
    color: var(--text);
  }
</style>
