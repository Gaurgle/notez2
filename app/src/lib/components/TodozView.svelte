<script lang="ts">
  import { onMount } from "svelte";
  import TodoItem from "$lib/components/todo/TodoItem.svelte";
  import FilterStrip from "$lib/components/todo/FilterStrip.svelte";
  import {
    loadTodoBoard,
    toggleTask,
    setTaskFlags,
    editTask,
    addTodo,
    removeTodo,
    reorderTask,
    toggleCollapse,
  } from "$lib/ipc";
  import type { TodoTask } from "$lib/types";

  let items = $state<TodoTask[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let filterText = $state("");
  let filterBits = $state(0);
  let dragId: number | null = null;

  onMount(async () => {
    await reload();
    loading = false;
  });

  async function reload() {
    try {
      items = (await loadTodoBoard()).items;
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  // Collapse-aware visibility, mirroring notez-core::todo::get_visible_indices.
  function collapseVisible(list: TodoTask[]): TodoTask[] {
    const out: TodoTask[] = [];
    let skipSection = false;
    let collapseDepth: number | null = null;
    for (const item of list) {
      if (item.is_header) {
        out.push(item);
        skipSection = item.collapsed;
        collapseDepth = null;
        continue;
      }
      if (skipSection) continue;
      if (collapseDepth !== null) {
        if (item.depth > collapseDepth) continue;
        collapseDepth = null;
      }
      out.push(item);
      if (item.has_subtasks && item.collapsed) collapseDepth = item.depth;
    }
    return out;
  }

  let visible = $derived.by(() => {
    const active = filterText.trim() !== "" || filterBits !== 0;
    if (!active) return collapseVisible(items);
    const tokens = filterText.toLowerCase().split(/\s+/).filter(Boolean);
    return items.filter((item) => {
      if (item.is_header) return true;
      const textOk = tokens.every((t) => item.text.toLowerCase().includes(t));
      const tagOk = (item.flags & filterBits) === filterBits;
      return textOk && tagOk;
    });
  });

  async function run(p: Promise<{ items: TodoTask[] }>) {
    try {
      items = (await p).items;
    } catch (e) {
      error = String(e);
    }
  }

  const onToggle = (id: number) => run(toggleTask(id));
  const onToggleCollapse = (id: number) => run(toggleCollapse(id));
  const onRemove = (id: number) => run(removeTodo(id));
  const onEdit = (id: number, text: string) => run(editTask(id, text));
  const onAdd = (afterId: number, depth: number) => run(addTodo(afterId, depth, "new task"));

  function onToggleFlag(id: number, bit: number) {
    const t = items.find((i) => i.id === id);
    if (t) run(setTaskFlags(id, t.flags ^ bit));
  }

  const onDragStart = (id: number) => {
    dragId = id;
  };
  function onDrop(targetId: number) {
    if (dragId !== null && dragId !== targetId) run(reorderTask(dragId, targetId));
    dragId = null;
  }
</script>

<div class="todoz">
  <FilterStrip
    text={filterText}
    activeBits={filterBits}
    onText={(t) => (filterText = t)}
    onToggleBit={(bit) => (filterBits ^= bit)}
  />

  <div class="board">
    {#if loading}
      <div class="status">Loading todos…</div>
    {:else if error}
      <div class="status error">{error}</div>
    {:else if visible.length === 0}
      <div class="status">No todos.</div>
    {:else}
      {#each visible as task (task.id)}
        <TodoItem
          {task}
          {onToggle}
          {onToggleFlag}
          {onToggleCollapse}
          {onRemove}
          {onEdit}
          {onAdd}
          {onDragStart}
          {onDrop}
        />
      {/each}
    {/if}
  </div>
</div>

<style>
  .todoz {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .board {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }
  .status {
    padding: 1rem;
    color: var(--subtext);
    font-size: 0.85rem;
  }
  .status.error {
    color: #f38ba8;
    white-space: pre-wrap;
  }
</style>
