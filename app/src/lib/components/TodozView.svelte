<script lang="ts">
  import { onMount } from "svelte";
  import TodoItem from "$lib/components/todo/TodoItem.svelte";
  import Inspector from "$lib/components/Inspector.svelte";
  import {
    loadTodoBoard,
    toggleTask,
    setTaskState,
    setTaskFlags,
    editTask,
    addTodo,
    removeTodo,
    reorderTask,
    toggleCollapse,
    moveTodo,
    collapseAll,
    createCategory,
  } from "$lib/ipc";
  import type { TodoTask } from "$lib/types";
  import { TAG_DEFS } from "$lib/types";

  let { active = true }: { active?: boolean } = $props();

  /** Resolve a `#token` (without `#`) into a tag bitset, like the CLI:
   *  empty → all, digits → those 1-based tags (OR), else name-prefix match. */
  function tagTokenBits(tok: string): number {
    if (tok === "") return 0b11111;
    if (/^\d+$/.test(tok)) {
      let s = 0;
      for (const ch of tok) {
        const i = Number(ch);
        if (i >= 1 && i <= 5) s |= 1 << (i - 1);
      }
      return s;
    }
    let s = 0;
    for (const d of TAG_DEFS) if (d.key.startsWith(tok)) s |= d.bit;
    return s;
  }

  let items = $state<TodoTask[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let filterText = $state("");
  let filterBits = $state(0);

  let selPos = $state(0);
  let editingId = $state<number | null>(null);
  let allCollapsed = $state(false);

  let showHelp = $state(false);
  let confirmDeleteId = $state<number | null>(null);
  let categoryMode = $state(false);
  let categoryDraft = $state("");
  let focusedSource = $state<string | null>(null);

  let dragId: number | null = null;
  let hoveredId = $state<number | null>(null);
  const onHover = (id: number | null) => (hoveredId = id);

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

  function computeVisible(
    list: TodoTask[],
    text: string,
    bits: number,
    focus: string | null
  ): TodoTask[] {
    let base = focus ? list.filter((t) => t.source === focus) : list;

    // Split the buffer into free-text tokens and `#tag` sets (AND across
    // tokens, OR within a token). Toggled filter dots add per-bit sets.
    const textTokens: string[] = [];
    const tagSets: number[] = [];
    for (const raw of text.split(/\s+/).filter(Boolean)) {
      if (raw.startsWith("#")) tagSets.push(tagTokenBits(raw.slice(1).toLowerCase()));
      else textTokens.push(raw.toLowerCase());
    }
    for (const d of TAG_DEFS) if (bits & d.bit) tagSets.push(d.bit);

    const active = textTokens.length > 0 || tagSets.length > 0;
    if (active) {
      return base.filter((item) => {
        if (item.is_header) return true;
        const textOk = textTokens.every((t) => item.text.toLowerCase().includes(t));
        const tagOk = tagSets.every((set) => (item.flags & set) !== 0);
        return textOk && tagOk;
      });
    }
    const out: TodoTask[] = [];
    let skipSection = false;
    let collapseDepth: number | null = null;
    for (const item of base) {
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

  let visible = $derived(computeVisible(items, filterText, filterBits, focusedSource));
  let selectedTask = $derived(visible[selPos] ?? null);

  // Tree connector metadata per visible row: vertical rails for each ancestor
  // level that continues below, and whether this row is its parent's last child.
  // Is `list[idx]` the last child among its siblings?
  function isLast(list: TodoTask[], idx: number, depth: number): boolean {
    for (let j = idx + 1; j < list.length; j++) {
      if (list[j].is_header || list[j].depth < depth) return true;
      if (list[j].depth === depth) return false;
    }
    return true;
  }

  // For each row: `prefix[c]` = draw a vertical guide at ancestor column c
  // (the ancestor there still has siblings below), and `last` = is this row
  // its parent's last child (└ vs ├). Columns: 0..depth-2 ancestors + the
  // connector at depth-1.
  function connectorsFor(list: TodoTask[]) {
    const res: { prefix: boolean[]; last: boolean }[] = [];
    for (let i = 0; i < list.length; i++) {
      const it = list[i];
      if (it.is_header) {
        res.push({ prefix: [], last: true });
        continue;
      }
      const d = it.depth;
      const last = isLast(list, i, d);

      // Walk up the ancestor chain, recording each ancestor's last-child state.
      const ancestorLast: Record<number, boolean> = {};
      let j = i;
      let curDepth = d;
      while (curDepth > 0) {
        let p = j - 1;
        while (p >= 0 && (list[p].is_header || list[p].depth >= curDepth)) p--;
        if (p < 0 || list[p].depth !== curDepth - 1) break;
        ancestorLast[curDepth - 1] = isLast(list, p, curDepth - 1);
        j = p;
        curDepth--;
      }

      const prefix: boolean[] = [];
      for (let c = 0; c <= d - 2; c++) prefix.push(!(ancestorLast[c + 1] ?? true));
      res.push({ prefix, last });
    }
    return res;
  }
  let connectors = $derived(connectorsFor(visible));

  // Section list for the left navigator (mirrors the notes scope sidebar).
  function sectionStats(list: TodoTask[]) {
    const secs: { source: string; label: string; pending: number }[] = [];
    for (let i = 0; i < list.length; i++) {
      if (!list[i].is_header) continue;
      let pending = 0;
      for (let j = i + 1; j < list.length; j++) {
        if (list[j].is_header) break;
        if (list[j].depth === 0 && list[j].state !== "checked") pending++;
      }
      secs.push({ source: list[i].source, label: list[i].text, pending });
    }
    return secs;
  }
  let sections = $derived(sectionStats(items));

  async function toolbarNew() {
    const sel = selectedTask ?? visible[0];
    if (!sel) return;
    const depth = sel.is_header ? 0 : sel.depth;
    items = (await addTodo(sel.id, depth, "new task")).items;
    selectId(sel.id + 1);
    editingId = sel.id + 1;
  }

  let inspected = $derived(
    (hoveredId !== null ? (items.find((i) => i.id === hoveredId) ?? null) : null) ?? selectedTask
  );
  let inspectorRows = $derived(
    inspected
      ? [
          { label: "Section", value: inspected.section },
          { label: "State", value: inspected.is_header ? "section" : inspected.state },
          { label: "Depth", value: inspected.is_header ? "—" : String(inspected.depth) },
          { label: "File", value: inspected.source.split("/").pop() ?? inspected.source },
        ]
      : []
  );
  let pending = $derived(
    items.filter((i) => !i.is_header && i.depth === 0 && i.state !== "checked").length
  );
  let done = $derived(
    items.filter((i) => !i.is_header && i.depth === 0 && i.state === "checked").length
  );

  $effect(() => {
    const n = visible.length;
    if (selPos >= n) selPos = Math.max(0, n - 1);
    if (selPos < 0) selPos = 0;
  });

  async function apply(p: Promise<{ items: TodoTask[] }>) {
    try {
      items = (await p).items;
    } catch (e) {
      error = String(e);
    }
  }

  function selectId(id: number) {
    const p = computeVisible(items, filterText, filterBits, focusedSource).findIndex((t) => t.id === id);
    if (p >= 0) selPos = p;
  }

  const onSelect = (id: number) => {
    const p = visible.findIndex((t) => t.id === id);
    if (p >= 0) selPos = p;
  };
  const onToggle = (id: number) => apply(toggleTask(id));
  const onToggleCollapse = (id: number) => apply(toggleCollapse(id));
  const onToggleFlag = (id: number, bit: number) => {
    const t = items.find((i) => i.id === id);
    if (t) apply(setTaskFlags(id, t.flags ^ bit));
  };
  const onEditStart = (id: number) => (editingId = id);
  const onEditCancel = () => (editingId = null);
  async function onEditCommit(id: number, text: string) {
    editingId = null;
    if (text.trim()) await apply(editTask(id, text.trim()));
  }
  const onDragStart = (id: number) => (dragId = id);
  function onDrop(targetId: number) {
    if (dragId !== null && dragId !== targetId) apply(reorderTask(dragId, targetId));
    dragId = null;
  }

  async function newTask(subtask: boolean) {
    const sel = selectedTask;
    if (!sel) return;
    const depth = sel.is_header ? 0 : subtask ? Math.min(sel.depth + 1, 2) : sel.depth;
    items = (await addTodo(sel.id, depth, "new task")).items;
    const newId = sel.id + 1;
    selectId(newId);
    editingId = newId;
  }

  async function submitCategory() {
    const name = categoryDraft.trim();
    categoryMode = false;
    categoryDraft = "";
    if (name) await apply(createCategory(name));
  }

  function clearTransient(): boolean {
    if (showHelp) { showHelp = false; return true; }
    if (confirmDeleteId !== null) { confirmDeleteId = null; return true; }
    if (categoryMode) { categoryMode = false; categoryDraft = ""; return true; }
    if (focusedSource) { focusedSource = null; return true; }
    return false;
  }

  async function handleKey(e: KeyboardEvent) {
    if (!active) return; // only the visible view handles keys
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    // Modal states intercept everything.
    if (showHelp) {
      showHelp = false;
      e.preventDefault();
      return;
    }
    if (confirmDeleteId !== null) {
      if (e.key === "y" || e.key === "Enter") {
        await apply(removeTodo(confirmDeleteId));
        confirmDeleteId = null;
      } else if (e.key === "n" || e.key === "Escape") {
        confirmDeleteId = null;
      }
      e.preventDefault();
      return;
    }
    if (e.key === "Escape") {
      if (clearTransient()) e.preventDefault();
      return;
    }

    const sel = selectedTask;
    const k = e.key;

    if (k === "j" || k === "ArrowDown") {
      selPos = Math.min(selPos + 1, visible.length - 1);
      e.preventDefault();
    } else if (k === "k" || k === "ArrowUp") {
      selPos = Math.max(selPos - 1, 0);
      e.preventDefault();
    } else if (k === "g") {
      selPos = 0;
    } else if (k === "G") {
      selPos = visible.length - 1;
    } else if (k === "/") {
      document.getElementById("todo-filter")?.focus();
      e.preventDefault();
    } else if (k === "?") {
      showHelp = true;
      e.preventDefault();
    } else if (k === "v") {
      allCollapsed = !allCollapsed;
      await apply(collapseAll(allCollapsed));
    } else if (k === "N") {
      categoryMode = true;
      e.preventDefault();
    } else if (!sel) {
      return;
    } else if (k === " " || k === "x" || k === "Enter") {
      if (sel.is_header || sel.has_subtasks) await apply(toggleCollapse(sel.id));
      else await apply(toggleTask(sel.id));
      e.preventDefault();
    } else if (k === "a") {
      if (!sel.is_header) await apply(setTaskState(sel.id, "half"));
    } else if (k === "e") {
      if (!sel.is_header) editingId = sel.id;
      e.preventDefault();
    } else if (k === "n") {
      await newTask(false);
      e.preventDefault();
    } else if (k === "s") {
      await newTask(true);
      e.preventDefault();
    } else if (k === "d") {
      if (!sel.is_header) confirmDeleteId = sel.id;
    } else if (k === "f") {
      focusedSource = focusedSource ? null : sel.source;
      selPos = 0;
    } else if (k === "h" || k === "ArrowLeft") {
      if ((sel.is_header || sel.has_subtasks) && !sel.collapsed) await apply(toggleCollapse(sel.id));
    } else if (k === "l" || k === "ArrowRight") {
      if ((sel.is_header || sel.has_subtasks) && sel.collapsed) await apply(toggleCollapse(sel.id));
    } else if (k === "J") {
      if (!sel.is_header) {
        await apply(moveTodo(sel.id, false));
        selectId(sel.id + 1);
      }
    } else if (k === "K") {
      if (!sel.is_header) {
        await apply(moveTodo(sel.id, true));
        selectId(sel.id - 1);
      }
    } else if (k >= "1" && k <= "5") {
      if (!sel.is_header) {
        const bit = 1 << (Number(k) - 1);
        await apply(setTaskFlags(sel.id, sel.flags ^ bit));
      }
    }
  }

  $effect(() => {
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  });

  const HELP = [
    ["j / k / ↑ ↓", "navigate"],
    ["g / G", "top / bottom"],
    ["space / x / enter", "toggle done (collapse on a section)"],
    ["a", "mark half-done"],
    ["e", "edit text"],
    ["n / s", "new todo / subtask"],
    ["d", "delete (confirm)"],
    ["1 – 5", "toggle importance tags"],
    ["J / K", "move up / down"],
    ["h / l", "collapse / expand"],
    ["v", "collapse / expand all"],
    ["f", "focus this section"],
    ["N", "new category"],
    ["/", "filter"],
    ["tab", "switch notes / todos"],
    ["esc", "clear filter / focus / dialog"],
  ];
</script>

<div class="todoz">
  <aside class="sidebar">
    <div class="brand">todoz</div>
    <nav class="group">
      <div class="group-label">Sections</div>
      <button
        class="item"
        class:active={focusedSource === null}
        onclick={() => (focusedSource = null)}
      >
        <span class="item-label">All sections</span>
        <span class="count">{pending}</span>
      </button>
      {#each sections as s (s.source)}
        <button
          class="item"
          class:active={focusedSource === s.source}
          onclick={() => {
            focusedSource = s.source;
            selPos = 0;
          }}
        >
          <span class="item-label">{s.label}</span>
          <span class="count">{s.pending}</span>
        </button>
      {/each}
    </nav>
  </aside>

  <div class="main">
    <div class="viewbar">
      <input
        id="todo-filter"
        class="searchbar"
        placeholder="Search… text, #prio, #1, #13"
        value={filterText}
        oninput={(e) => (filterText = (e.target as HTMLInputElement).value)}
      />
      <span class="tagdots">
        {#each TAG_DEFS as d (d.bit)}
          <button
            class="fdot"
            class:on={(filterBits & d.bit) !== 0}
            style="--c:{d.color}"
            title={d.label}
            aria-label={d.label}
            onclick={() => (filterBits ^= d.bit)}
          ></button>
        {/each}
      </span>
      <span class="counts">{pending} pending · {done} done</span>
      <div class="spacer"></div>
      <button class="primary" onclick={toolbarNew}>+ New</button>
      <button class="ghost icon" title="keybindings" onclick={() => (showHelp = true)}>?</button>
    </div>

    {#if categoryMode}
      <form
        class="category"
        onsubmit={(e) => {
          e.preventDefault();
          submitCategory();
        }}
      >
        <span class="lbl">new category</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          bind:value={categoryDraft}
          autofocus
          placeholder="name…"
          onkeydown={(e) => {
            if (e.key === "Escape") {
              categoryMode = false;
              categoryDraft = "";
            }
          }}
        />
      </form>
    {/if}

    {#if confirmDeleteId !== null}
      <div class="confirm">delete this todo? <kbd>y</kbd> yes · <kbd>n</kbd> no</div>
    {/if}

    <div class="body">
      <div class="board">
        {#if loading}
          <div class="status">Loading todos…</div>
        {:else if error}
          <div class="status error">{error}</div>
        {:else if visible.length === 0}
          <div class="status">No todos.</div>
        {:else}
          {#each visible as task, i (task.id)}
            <TodoItem
              {task}
              selected={i === selPos}
              editing={editingId === task.id}
              prefix={connectors[i]?.prefix ?? []}
              last={connectors[i]?.last ?? true}
              {onSelect}
              {onToggle}
              {onToggleFlag}
              {onToggleCollapse}
              {onEditStart}
              {onEditCommit}
              {onEditCancel}
              {onDragStart}
              {onDrop}
              {onHover}
            />
          {/each}
        {/if}
      </div>

      <Inspector title={inspected ? inspected.text : null} flags={inspected?.flags ?? 0} rows={inspectorRows} />
    </div>
  </div>

  {#if showHelp}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div
      class="overlay"
      role="presentation"
      onclick={(e) => {
        if (e.target === e.currentTarget) showHelp = false;
      }}
    >
      <div class="help">
        <h2>Keybindings</h2>
        <div class="rows">
          {#each HELP as [keys, desc] (keys)}
            <kbd>{keys}</kbd>
            <span>{desc}</span>
          {/each}
        </div>
        <div class="dismiss">press any key to close</div>
      </div>
    </div>
  {/if}
</div>

<style>
  .todoz {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  /* Sections navigator — mirrors the notes scope sidebar. */
  .sidebar {
    width: 200px;
    flex-shrink: 0;
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
  .item:hover {
    background: var(--surface);
  }
  .item.active {
    background: var(--surface-active);
  }
  .item-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .count {
    font-size: 0.68rem;
    color: var(--subtext);
  }

  .main {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
  .tagdots {
    display: flex;
    gap: 0.35rem;
    flex-shrink: 0;
  }
  .fdot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1px solid var(--c);
    background: transparent;
    cursor: pointer;
    padding: 0;
    opacity: 0.45;
  }
  .fdot.on {
    background: var(--c);
    opacity: 1;
  }
  .category,
  .confirm {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.45rem 0.85rem;
    border-bottom: 1px solid var(--border);
    font-size: 0.78rem;
  }
  .category .lbl {
    color: var(--accent-public);
    text-transform: uppercase;
    font-size: 0.62rem;
    letter-spacing: 0.05em;
  }
  .category input {
    flex: 1;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--accent);
    border-radius: 0.3rem;
    color: var(--text);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.25rem 0.5rem;
  }
  .category input:focus {
    outline: none;
  }
  .confirm {
    color: var(--accent-global);
    background: rgba(250, 179, 135, 0.08);
  }
  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  .board {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
    background: rgba(18, 18, 28, 0.92);
    padding-bottom: 1rem;
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

  kbd {
    font-family: ui-monospace, monospace;
    font-size: 0.68rem;
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid var(--border);
    border-radius: 0.3rem;
    padding: 0.05rem 0.35rem;
    color: var(--text);
    white-space: nowrap;
  }
  .help {
    background: var(--glass-strong);
    -webkit-backdrop-filter: var(--blur);
    backdrop-filter: var(--blur);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    padding: 1.25rem 1.5rem;
    width: 380px;
    max-width: 90vw;
  }
  .help h2 {
    margin: 0 0 0.85rem;
    font-size: 0.95rem;
    color: var(--accent);
  }
  .help .rows {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 0.85rem;
    align-items: center;
    font-size: 0.78rem;
    color: var(--subtext);
  }
  .help .dismiss {
    margin-top: 1rem;
    font-size: 0.68rem;
    color: var(--faint);
    text-align: center;
  }
</style>
