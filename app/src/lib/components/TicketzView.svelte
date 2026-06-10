<script lang="ts">
  // Mock "ticketz" board — a quick Trello-style preview. Real tickets will be
  // repo-bound files synced via git (see DESIGN.md). Data here is placeholder.
  import Avatar from "$lib/components/Avatar.svelte";
  import MachineAvatar from "$lib/components/MachineAvatar.svelte";
  import MarkdownPreview from "$lib/components/MarkdownPreview.svelte";
  import Resizer from "$lib/components/Resizer.svelte";
  import { Plus, Filter, Pencil, Eye } from "lucide-svelte";

  let { active = true }: { active?: boolean } = $props();

  const COLUMNS = [
    { key: "backlog", label: "Backlog" },
    { key: "progress", label: "In progress" },
    { key: "review", label: "Review" },
    { key: "done", label: "Done" },
  ] as const;

  type Lane = (typeof COLUMNS)[number]["key"];
  type Label = "feature" | "bug" | "design" | "chore";
  interface Ticket {
    id: number;
    title: string;
    lane: Lane;
    label: Label;
    assignee: string;
    project: string;
    points: number; // story points, Fibonacci scale (effort/size)
    body: string; // optional markdown notes
  }

  const LABEL_COLOR: Record<Label, string> = {
    feature: "var(--accent-public)",
    bug: "var(--danger)",
    design: "var(--accent-personal)",
    chore: "var(--accent-global)",
  };

  const POINT_SCALE = [1, 2, 3, 5, 8, 13];
  function pointTone(p: number): string {
    if (p <= 2) return "var(--accent-public)"; // small
    if (p <= 5) return "var(--accent-global)"; // medium
    return "var(--danger)"; // large
  }

  let TICKETS = $state<Ticket[]>([
    { id: 101, title: "Calendar date encoding (@date token)", lane: "backlog", label: "feature", assignee: "you", project: "notez2", points: 5, body: "## Goal\nAn `@date` token in todo text that round-trips through the CLI.\n\n- [ ] parse `@2026-06-12`\n- [ ] serialize losslessly\n- [ ] calendar reads it\n\nBlocked by the storage-format decision." },
    { id: 102, title: "Ticket files synced via git", lane: "backlog", label: "feature", assignee: "alex", project: "notez2", points: 8, body: "Each ticket = a markdown file in the repo, synced like notes/todos. Same identity, same git push." },
    { id: 103, title: "Extended task states: deferred / scrapped", lane: "backlog", label: "feature", assignee: "mira", project: "notez2", points: 3, body: "" },
    { id: 104, title: "repoz: broad repo status scan", lane: "backlog", label: "chore", assignee: "sam", project: "repoz", points: 5, body: "Scan all repos for **dirty trees** and **unpushed commits**, summarise per repo." },
    { id: 201, title: "GitHub OAuth device flow", lane: "progress", label: "feature", assignee: "you", project: "spaze", points: 8, body: "1. `POST /device/code`\n2. poll for the token\n3. store in the OS keychain\n\nUnlocks identity everywhere (avatars, authorship)." },
    { id: 202, title: "Contextual inspector + cross counts", lane: "progress", label: "feature", assignee: "nora", project: "notez2", points: 5, body: "" },
    { id: 203, title: "epoz: git handling wrapper", lane: "progress", label: "feature", assignee: "kai", project: "epoz", points: 13, body: "The big one. Wraps common git flows; `repoz`'s big brother." },
    { id: 301, title: "Unify toolbars on lucide icons", lane: "review", label: "design", assignee: "mira", project: "notez2", points: 3, body: "Even, consistent toolbars across notez/todoz/ticketz." },
    { id: 302, title: "Duplicate-path dedupe in collect_all", lane: "review", label: "bug", assignee: "you", project: "notez2", points: 2, body: "Personal notes were double-walked into the global scope → duplicate keys → crash. Fixed + test." },
    { id: 401, title: "Split markdown preview pane", lane: "done", label: "feature", assignee: "alex", project: "notez2", points: 5, body: "" },
    { id: 402, title: "todoz tree connectors", lane: "done", label: "design", assignee: "nora", project: "notez2", points: 2, body: "" },
    { id: 403, title: "Workspace + serde restructure", lane: "done", label: "chore", assignee: "you", project: "notez2", points: 13, body: "" },
  ]);

  const PROJECTS = (() => {
    const order: string[] = [];
    const map: Record<string, number> = {};
    for (const t of TICKETS) {
      if (!(t.project in map)) {
        map[t.project] = 0;
        order.push(t.project);
      }
      map[t.project]++;
    }
    return order.map((name) => ({ name, count: map[name] }));
  })();

  let activeProject = $state<string | null>(null);
  let sidebarWidth = $state(185);
  let inspectorWidth = $state(310);
  let visible = $derived(
    activeProject === null ? TICKETS : TICKETS.filter((t) => t.project === activeProject)
  );
  function inLane(lane: Lane): Ticket[] {
    return visible.filter((t) => t.lane === lane);
  }
  function laneLabel(lane: Lane): string {
    return COLUMNS.find((c) => c.key === lane)?.label ?? lane;
  }

  // Inspector source: hovering a card previews it; clicking pins it. When the
  // pointer is off the cards (e.g. in the inspector), the pinned one shows.
  let selectedId = $state<number | null>(null);
  let hoveredId = $state<number | null>(null);
  let editingBody = $state(false);
  let inspected = $derived(
    (hoveredId !== null ? TICKETS.find((t) => t.id === hoveredId) : undefined) ??
      (selectedId !== null ? TICKETS.find((t) => t.id === selectedId) : undefined) ??
      null
  );

  // Drag to move between lanes and reorder within a lane.
  let draggingId = $state<number | null>(null);
  let dragLane = $state<Lane | null>(null);
  let dragBeforeId = $state<number | null>(null);

  function clearDrag() {
    draggingId = null;
    dragLane = null;
    dragBeforeId = null;
  }
  function performDrop() {
    if (draggingId === null) return clearDrag();
    const from = TICKETS.findIndex((t) => t.id === draggingId);
    if (from < 0) return clearDrag();
    const beforeId = dragBeforeId !== draggingId ? dragBeforeId : null;
    const targetLane = dragLane;
    const [moved] = TICKETS.splice(from, 1);
    if (targetLane) moved.lane = targetLane;
    let at = TICKETS.length;
    if (beforeId !== null) {
      const bi = TICKETS.findIndex((t) => t.id === beforeId);
      if (bi >= 0) at = bi;
    }
    TICKETS.splice(at, 0, moved);
    clearDrag();
  }
</script>

<div class="ticketz">
  <aside class="sidebar" style="width:{sidebarWidth}px">
    <div class="brand">
      <MachineAvatar />
      <span class="brand-name">ticketz</span>
    </div>
    <nav class="group">
      <div class="group-label">Projects</div>
      <button class="item" class:active={activeProject === null} onclick={() => (activeProject = null)}>
        <span class="item-label">All projects</span>
        <span class="count">{TICKETS.length}</span>
      </button>
      {#each PROJECTS as p (p.name)}
        <button
          class="item"
          class:active={activeProject === p.name}
          onclick={() => (activeProject = p.name)}
        >
          <span class="item-label">{p.name}</span>
          <span class="count">{p.count}</span>
        </button>
      {/each}
    </nav>
  </aside>
  <Resizer get={() => sidebarWidth} set={(n) => (sidebarWidth = n)} dir={1} min={160} max={320} />

  <div class="main">
    <div class="viewbar">
      <span class="title">Ticketz</span>
      <span class="counts">{visible.length} tickets{activeProject ? ` · ${activeProject}` : ""} · mock</span>
      <div class="spacer"></div>
      <button class="ghost iconbtn icononly" title="Filter" aria-label="Filter"><Filter size={15} /></button>
      <button class="ghost iconbtn icononly" title="New ticket" aria-label="New ticket"><Plus size={16} /></button>
    </div>

    <div class="board">
      {#each COLUMNS as col (col.key)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <section
          class="lane"
          class:dragover={dragLane === col.key}
          ondragover={(e) => {
            e.preventDefault();
            dragLane = col.key;
            dragBeforeId = null;
          }}
          ondrop={(e) => {
            e.preventDefault();
            performDrop();
          }}
        >
          <header class="lane-head">
            <span class="lane-label">{col.label}</span>
            <span class="lane-count">{inLane(col.key).length}</span>
          </header>
          <div class="cards">
            {#each inLane(col.key) as t (t.id)}
              <div
                class="card"
                class:selected={selectedId === t.id}
                class:dragging={draggingId === t.id}
                class:drop-before={dragBeforeId === t.id && draggingId !== null && draggingId !== t.id}
                draggable="true"
                tabindex="0"
                role="button"
                onclick={() => (selectedId = t.id)}
                onkeydown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    selectedId = t.id;
                  }
                }}
                onmouseenter={() => (hoveredId = t.id)}
                onmouseleave={() => (hoveredId = null)}
                ondragstart={(e) => {
                  draggingId = t.id;
                  e.dataTransfer?.setData("text/plain", String(t.id));
                  if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
                }}
                ondragend={clearDrag}
                ondragover={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  dragLane = t.lane;
                  dragBeforeId = t.id;
                }}
                ondrop={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  performDrop();
                }}
              >
                <div class="card-top">
                  <span class="top-left">
                    <span class="tag" style="--c:{LABEL_COLOR[t.label]}">{t.label}</span>
                    <span class="num">#{t.id}</span>
                  </span>
                  <span class="pts" style="--pc:{pointTone(t.points)}" title="{t.points} story points">
                    {t.points} <span class="sp">SP</span>
                  </span>
                </div>
                <div class="card-title">{t.title}</div>
                <div class="card-foot">
                  <span class="proj">{t.project}</span>
                  <span class="assignee">
                    {#if t.body.trim()}<span class="has-notes" title="Has notes">●</span>{/if}
                    <Avatar name={t.assignee} size={18} />
                  </span>
                </div>
              </div>
            {/each}
          </div>
        </section>
      {/each}
    </div>
  </div>

  <Resizer get={() => inspectorWidth} set={(n) => (inspectorWidth = n)} dir={-1} min={240} max={520} />
  <aside class="inspector" style="width:{inspectorWidth}px">
    {#if inspected}
      <input
        class="insp-title"
        value={inspected.title}
        oninput={(e) => inspected && (inspected.title = e.currentTarget.value)}
        placeholder="Ticket title"
      />
      <div class="insp-tags">
        <span class="tag" style="--c:{LABEL_COLOR[inspected.label]}">{inspected.label}</span>
        <span class="proj">{inspected.project}</span>
        <span class="lane-badge">{laneLabel(inspected.lane)}</span>
        <span class="id">#{inspected.id}</span>
      </div>

      <div class="insp-row">
        <span class="insp-key">Assignee</span>
        <span class="insp-val"><Avatar name={inspected.assignee} size={18} /> {inspected.assignee}</span>
      </div>

      <div class="insp-row">
        <span class="insp-key">Story points</span>
        <div class="pt-picker">
          {#each POINT_SCALE as p (p)}
            <button
              class="pt-btn"
              class:on={inspected.points === p}
              style="--pc:{pointTone(p)}"
              onclick={() => inspected && (inspected.points = p)}
            >
              {p}
            </button>
          {/each}
        </div>
      </div>

      <div class="insp-body">
        <div class="insp-body-head">
          <span class="insp-key">Notes</span>
          <button class="mini-toggle" onclick={() => (editingBody = !editingBody)}>
            {#if editingBody}<Eye size={12} /> Preview{:else}<Pencil size={12} /> Edit{/if}
          </button>
        </div>
        {#if editingBody}
          <textarea
            class="body-edit"
            value={inspected.body}
            oninput={(e) => inspected && (inspected.body = e.currentTarget.value)}
            placeholder="Markdown notes for this ticket…"
          ></textarea>
        {:else if inspected.body.trim()}
          <div class="body-preview"><MarkdownPreview content={inspected.body} /></div>
        {:else}
          <div class="body-empty">
            No notes yet. <button class="link" onclick={() => (editingBody = true)}>Add some →</button>
          </div>
        {/if}
      </div>
    {:else}
      <div class="empty">Hover or select a ticket to inspect it.</div>
    {/if}
  </aside>
</div>

<style>
  .ticketz {
    display: flex;
    height: 100%;
    background: var(--base);
  }
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
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
  .item .count {
    font-size: 0.68rem;
    color: var(--subtext);
  }
  .title {
    font-weight: 700;
    color: var(--accent);
    font-size: 0.95rem;
  }
  .board {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 0.75rem;
    padding: 0.85rem;
    overflow-x: auto;
  }
  .lane {
    display: flex;
    flex-direction: column;
    min-width: 250px;
    width: 250px;
    flex-shrink: 0;
    background: rgba(255, 255, 255, 0.018);
    border: 1px solid var(--border);
    border-radius: 0.6rem;
    transition: background 0.12s, border-color 0.12s;
  }
  .lane.dragover {
    background: color-mix(in srgb, var(--accent) 7%, transparent);
    border-color: color-mix(in srgb, var(--accent) 40%, transparent);
  }
  .lane-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.55rem 0.7rem;
    border-bottom: 1px solid var(--border);
  }
  .lane-label {
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--subtext);
  }
  .lane-count {
    font-size: 0.68rem;
    color: var(--faint);
  }
  .cards {
    flex: 1;
    overflow-y: auto;
    padding: 0.55rem;
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
  }
  .card {
    position: relative;
    background: var(--mantle);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 0.6rem;
    cursor: grab;
    transition: border-color 0.12s, transform 0.12s, opacity 0.12s;
  }
  .card:hover {
    border-color: var(--border-strong);
    transform: translateY(-1px);
  }
  .card:active {
    cursor: grabbing;
  }
  .card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }
  .card.dragging {
    opacity: 0.4;
  }
  .card.drop-before::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: -4px;
    height: 2px;
    border-radius: 2px;
    background: var(--accent);
  }
  .card-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
    margin-bottom: 0.45rem;
  }
  .top-left {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    min-width: 0;
  }
  .tag {
    font-size: 0.6rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--c);
    background: color-mix(in srgb, var(--c) 16%, transparent);
    padding: 0.08rem 0.4rem;
    border-radius: 0.5rem;
    flex-shrink: 0;
  }
  .num {
    font-size: 0.64rem;
    color: var(--faint);
    font-variant-numeric: tabular-nums;
  }
  /* story points — a square chip, clearly different from the round avatar */
  .pts {
    flex-shrink: 0;
    font-size: 0.66rem;
    font-weight: 800;
    color: var(--pc);
    background: color-mix(in srgb, var(--pc) 16%, transparent);
    padding: 0.1rem 0.4rem;
    border-radius: 0.4rem;
    font-variant-numeric: tabular-nums;
  }
  .pts .sp {
    font-size: 0.54rem;
    font-weight: 700;
    opacity: 0.8;
  }
  .card-title {
    font-size: 0.84rem;
    line-height: 1.35;
    color: var(--text);
    margin-bottom: 0.55rem;
  }
  .card-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .proj {
    font-size: 0.66rem;
    color: var(--faint);
    background: var(--glass-hover);
    padding: 0.05rem 0.4rem;
    border-radius: 0.5rem;
  }
  .assignee {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }
  .has-notes {
    font-size: 0.5rem;
    color: var(--accent-local);
  }

  /* Inspector */
  .inspector {
    flex-shrink: 0;
    border-left: 1px solid var(--border);
    background: var(--mantle);
    padding: 0.9rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }
  .empty {
    color: var(--faint);
    font-size: 0.8rem;
    margin-top: 1rem;
  }
  .insp-title {
    width: 100%;
    background: none;
    border: none;
    border-bottom: 1px solid transparent;
    color: var(--text);
    font: inherit;
    font-size: 1rem;
    font-weight: 700;
    line-height: 1.3;
    padding: 0.1rem 0;
  }
  .insp-title:hover {
    border-bottom-color: var(--border);
  }
  .insp-title:focus {
    outline: none;
    border-bottom-color: var(--accent);
  }
  .insp-tags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
  }
  .lane-badge {
    font-size: 0.62rem;
    color: var(--subtext);
    background: var(--surface-active);
    padding: 0.08rem 0.45rem;
    border-radius: 0.5rem;
  }
  .id {
    font-size: 0.64rem;
    color: var(--faint);
    margin-left: auto;
    font-variant-numeric: tabular-nums;
  }
  .insp-row {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .insp-key {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--faint);
  }
  .insp-val {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.82rem;
    color: var(--text);
  }
  .pt-picker {
    display: flex;
    gap: 0.3rem;
  }
  .pt-btn {
    flex: 1;
    padding: 0.3rem 0;
    border: 1px solid var(--border);
    border-radius: 0.4rem;
    background: none;
    color: var(--subtext);
    cursor: pointer;
    font: inherit;
    font-size: 0.78rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    transition: all 0.12s;
  }
  .pt-btn:hover {
    border-color: var(--pc);
    color: var(--pc);
  }
  .pt-btn.on {
    color: var(--pc);
    border-color: var(--pc);
    background: color-mix(in srgb, var(--pc) 16%, transparent);
  }
  .insp-body {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    flex: 1;
    min-height: 0;
  }
  .insp-body-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .mini-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.66rem;
    color: var(--subtext);
    background: var(--glass-hover);
    border: 1px solid var(--border);
    border-radius: 0.4rem;
    padding: 0.1rem 0.4rem;
    cursor: pointer;
    font-family: inherit;
  }
  .mini-toggle:hover {
    color: var(--text);
    background: var(--glass-active);
  }
  .body-edit {
    flex: 1;
    min-height: 160px;
    resize: vertical;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    color: var(--text);
    font-family: ui-monospace, "SF Mono", monospace;
    font-size: 0.78rem;
    line-height: 1.5;
    padding: 0.55rem 0.6rem;
  }
  .body-edit:focus {
    outline: none;
    border-color: var(--accent);
  }
  .body-preview {
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    overflow: hidden;
  }
  .body-empty {
    font-size: 0.78rem;
    color: var(--faint);
  }
  .link {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font: inherit;
    font-size: 0.78rem;
    padding: 0;
  }
  .link:hover {
    text-decoration: underline;
  }
</style>
