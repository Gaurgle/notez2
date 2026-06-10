<script lang="ts">
  // Mock "ticketz" board — a quick Trello-style preview. Real tickets will be
  // repo-bound files synced via git (see DESIGN.md). Data here is placeholder.
  import Avatar from "$lib/components/Avatar.svelte";
  import MachineAvatar from "$lib/components/MachineAvatar.svelte";
  import MarkdownPreview from "$lib/components/MarkdownPreview.svelte";
  import Calendar from "$lib/components/Calendar.svelte";
  import Resizer from "$lib/components/Resizer.svelte";
  import { Plus, Filter, Eye, Pencil, PanelRight, CalendarDays } from "lucide-svelte";

  let { active = true }: { active?: boolean } = $props();

  const COLUMNS = [
    { key: "backlog", label: "Backlog" },
    { key: "progress", label: "In progress" },
    { key: "review", label: "Review" },
    { key: "done", label: "Done" },
  ] as const;

  type Lane = (typeof COLUMNS)[number]["key"];
  type Label = "feature" | "bug" | "design" | "chore";
  const LABELS: Label[] = ["feature", "bug", "design", "chore"];
  interface Ticket {
    id: number;
    title: string;
    lane: Lane;
    label: Label;
    assignee: string;
    project: string;
    points: number;
    body: string;
  }

  const LABEL_COLOR: Record<Label, string> = {
    feature: "var(--accent-public)",
    bug: "var(--danger)",
    design: "var(--accent-personal)",
    chore: "var(--accent-global)",
  };

  const POINT_SCALE = [1, 2, 3, 5, 8, 13];
  function pointTone(p: number): string {
    if (p <= 2) return "var(--accent-public)";
    if (p <= 5) return "var(--accent-global)";
    return "var(--danger)";
  }

  // Who is "in" each project — you can only assign a project's members.
  const PROJECT_MEMBERS: Record<string, string[]> = {
    notez2: ["you", "alex", "mira", "nora", "sam"],
    spaze: ["you", "sam", "mira", "kai"],
    repoz: ["sam", "kai", "you"],
    epoz: ["kai", "you", "alex"],
  };
  function membersFor(project: string): string[] {
    return PROJECT_MEMBERS[project] ?? ["you"];
  }

  let TICKETS = $state<Ticket[]>([
    { id: 101, title: "Calendar date encoding (@date token)", lane: "backlog", label: "feature", assignee: "you", project: "notez2", points: 5, body: "## Goal\nAn `@date` token in todo text that round-trips through the CLI.\n\n- [ ] parse `@2026-06-12`\n- [ ] serialize losslessly\n- [ ] calendar reads it" },
    { id: 102, title: "Ticket files synced via git", lane: "backlog", label: "feature", assignee: "alex", project: "notez2", points: 8, body: "Each ticket = a markdown file in the repo, synced like notes/todos." },
    { id: 103, title: "Extended task states: deferred / scrapped", lane: "backlog", label: "feature", assignee: "mira", project: "notez2", points: 3, body: "" },
    { id: 104, title: "repoz: broad repo status scan", lane: "backlog", label: "chore", assignee: "sam", project: "repoz", points: 5, body: "Scan all repos for **dirty trees** and **unpushed commits**." },
    { id: 201, title: "GitHub OAuth device flow", lane: "progress", label: "feature", assignee: "you", project: "spaze", points: 8, body: "1. `POST /device/code`\n2. poll for the token\n3. store in the keychain" },
    { id: 202, title: "Contextual inspector + cross counts", lane: "progress", label: "feature", assignee: "nora", project: "notez2", points: 5, body: "" },
    { id: 203, title: "epoz: git handling wrapper", lane: "progress", label: "feature", assignee: "kai", project: "epoz", points: 13, body: "The big one — `repoz`'s big brother." },
    { id: 301, title: "Unify toolbars on lucide icons", lane: "review", label: "design", assignee: "mira", project: "notez2", points: 3, body: "Even toolbars across notez/todoz/ticketz." },
    { id: 302, title: "Duplicate-path dedupe in collect_all", lane: "review", label: "bug", assignee: "you", project: "notez2", points: 2, body: "Personal notes double-walked into global → crash. Fixed + test." },
    { id: 401, title: "Split markdown preview pane", lane: "done", label: "feature", assignee: "alex", project: "notez2", points: 5, body: "" },
    { id: 402, title: "todoz tree connectors", lane: "done", label: "design", assignee: "nora", project: "notez2", points: 2, body: "" },
    { id: 403, title: "Workspace + serde restructure", lane: "done", label: "chore", assignee: "you", project: "notez2", points: 13, body: "" },
  ]);
  let nextId = 500;

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
  let visible = $derived(
    activeProject === null ? TICKETS : TICKETS.filter((t) => t.project === activeProject)
  );
  function inLane(lane: Lane): Ticket[] {
    return visible.filter((t) => t.lane === lane);
  }
  function laneLabel(lane: Lane): string {
    return COLUMNS.find((c) => c.key === lane)?.label ?? lane;
  }

  let selectedId = $state<number | null>(null);
  let hoveredId = $state<number | null>(null);
  let hoveredProject = $state<string | null>(null);
  // `selected` is the pinned ticket the editor acts on; `inspected` follows the
  // hover (a ticket, or the pinned one when nothing is hovered) so the inspector
  // and preview reveal info live, like the other views.
  let selected = $derived(selectedId !== null ? (TICKETS.find((t) => t.id === selectedId) ?? null) : null);
  let inspected = $derived(
    (hoveredId !== null ? TICKETS.find((t) => t.id === hoveredId) : undefined) ??
      (selectedId !== null ? TICKETS.find((t) => t.id === selectedId) : undefined) ??
      null
  );

  // Side panes. Inspector is open by default; preview stacks below it in the
  // same right zone. Edit and calendar are their own panes. Toggled p/e/i/c.
  let showPreview = $state(false);
  let showEdit = $state(true);
  let showInspector = $state(true);
  let showCalendar = $state(false);
  let editWidth = $state(320);
  let calendarWidth = $state(250);
  let zoneWidth = $state(310); // shared inspector + preview zone

  $effect(() => {
    function onKey(e: KeyboardEvent) {
      const el = e.target as HTMLElement | null;
      if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement || el instanceof HTMLSelectElement) {
        if (e.key === "Escape") el.blur();
        return;
      }
      if (!active) return;
      if (e.key === "p") showPreview = !showPreview;
      else if (e.key === "e") showEdit = !showEdit;
      else if (e.key === "i") showInspector = !showInspector;
      else if (e.key === "c") showCalendar = !showCalendar;
      else if (e.key === "Escape") {
        showPreview = showEdit = showInspector = showCalendar = false;
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  function newTicket(lane: Lane = "backlog") {
    const project = activeProject ?? PROJECTS[0]?.name ?? "notez2";
    const id = nextId++;
    TICKETS.push({
      id,
      title: "New ticket",
      lane,
      label: "feature",
      assignee: membersFor(project)[0] ?? "you",
      project,
      points: 3,
      body: "",
    });
    selectedId = id;
    showEdit = true;
  }

  // --- Pointer-based drag (HTML5 DnD is unreliable in WKWebView) ----------
  let pointer: { id: number; x0: number; y0: number; moved: boolean } | null = null;
  let dragId = $state<number | null>(null);
  let ghost = $state<{ x: number; y: number } | null>(null);
  let dropLane = $state<Lane | null>(null);
  let dropTargetId = $state<number | null>(null);
  let dropAfter = $state(false);
  let draggedTicket = $derived(dragId !== null ? (TICKETS.find((t) => t.id === dragId) ?? null) : null);

  function cardPointerDown(e: PointerEvent, t: Ticket) {
    if (e.button !== 0) return;
    pointer = { id: t.id, x0: e.clientX, y0: e.clientY, moved: false };
    window.addEventListener("pointermove", pointerMove);
    window.addEventListener("pointerup", pointerUp);
  }
  function pointerMove(e: PointerEvent) {
    if (!pointer) return;
    if (!pointer.moved && Math.hypot(e.clientX - pointer.x0, e.clientY - pointer.y0) < 5) return;
    pointer.moved = true;
    dragId = pointer.id;
    ghost = { x: e.clientX, y: e.clientY };
    const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null;
    dropLane = (el?.closest("[data-lane]") as HTMLElement | null)?.dataset.lane as Lane ?? null;
    const cardEl = el?.closest("[data-ticket]") as HTMLElement | null;
    if (cardEl) {
      dropTargetId = Number(cardEl.dataset.ticket);
      const r = cardEl.getBoundingClientRect();
      dropAfter = e.clientY > r.top + r.height / 2;
    } else {
      dropTargetId = null;
      dropAfter = false;
    }
  }
  function pointerUp() {
    window.removeEventListener("pointermove", pointerMove);
    window.removeEventListener("pointerup", pointerUp);
    if (pointer && !pointer.moved) {
      selectedId = pointer.id; // a tap selects
    } else if (dragId !== null) {
      performDrop();
    }
    pointer = null;
    dragId = null;
    ghost = null;
    dropLane = null;
    dropTargetId = null;
  }
  function performDrop() {
    const id = dragId;
    if (id === null) return;
    const from = TICKETS.findIndex((t) => t.id === id);
    if (from < 0) return;
    const [moved] = TICKETS.splice(from, 1);
    if (dropLane) moved.lane = dropLane;
    let at = TICKETS.length;
    if (dropTargetId !== null && dropTargetId !== id) {
      const ti = TICKETS.findIndex((t) => t.id === dropTargetId);
      if (ti >= 0) at = dropAfter ? ti + 1 : ti;
    }
    TICKETS.splice(at, 0, moved);
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
          onmouseenter={() => (hoveredProject = p.name)}
          onmouseleave={() => hoveredProject === p.name && (hoveredProject = null)}
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
      <button class="newbtn" onclick={() => newTicket()} title="New ticket (in Backlog)">
        <Plus size={15} /> New
      </button>
    </div>

    <div class="board">
      {#each COLUMNS as col (col.key)}
        <section class="lane" class:dragover={dropLane === col.key && dragId !== null} data-lane={col.key}>
          <header class="lane-head">
            <span class="lane-label">{col.label}</span>
            <span class="lane-count">{inLane(col.key).length}</span>
          </header>
          <div class="cards">
            {#each inLane(col.key) as t (t.id)}
              <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
              <div
                class="card"
                class:selected={selectedId === t.id}
                class:dragging={dragId === t.id}
                class:drop-before={dropTargetId === t.id && !dropAfter && dragId !== null && dragId !== t.id}
                class:drop-after={dropTargetId === t.id && dropAfter && dragId !== null && dragId !== t.id}
                data-ticket={t.id}
                onpointerdown={(e) => cardPointerDown(e, t)}
                onmouseenter={() => dragId === null && (hoveredId = t.id)}
                onmouseleave={() => hoveredId === t.id && (hoveredId = null)}
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
            <button class="add-card" onclick={() => newTicket(col.key)}>
              <Plus size={13} /> Add ticket
            </button>
          </div>
        </section>
      {/each}
    </div>

    <div class="statusbar">
      <span class="sb-path">{selected ? selected.title : "no ticket selected"}</span>
      <div class="sb-spacer"></div>
      <span class="sb-count">{visible.length} tickets</span>
      <div class="pane-toggles">
        <button class="pane-toggle" class:on={showPreview} onclick={() => (showPreview = !showPreview)} title="Preview (p)" aria-label="Toggle preview">
          <Eye size={14} />
        </button>
        <button class="pane-toggle" class:on={showEdit} onclick={() => (showEdit = !showEdit)} title="Edit (e)" aria-label="Toggle edit">
          <Pencil size={14} />
        </button>
        <button class="pane-toggle" class:on={showCalendar} onclick={() => (showCalendar = !showCalendar)} title="Calendar (c)" aria-label="Toggle calendar">
          <CalendarDays size={14} />
        </button>
        <button class="pane-toggle" class:on={showInspector} onclick={() => (showInspector = !showInspector)} title="Inspector (i)" aria-label="Toggle inspector">
          <PanelRight size={14} />
        </button>
      </div>
    </div>
  </div>

  {#if showEdit}
    <Resizer get={() => editWidth} set={(n) => (editWidth = n)} dir={-1} min={260} max={520} />
    <aside class="pane" style="width:{editWidth}px">
      {#if selected}
        <div class="pane-head"><Pencil size={13} /> Edit · #{selected.id}</div>
        <label class="field">
          <span class="flabel">Title</span>
          <input class="f-input" value={selected.title} oninput={(e) => selected && (selected.title = e.currentTarget.value)} />
        </label>
        <div class="field">
          <span class="flabel">Label</span>
          <div class="seg">
            {#each LABELS as l (l)}
              <button class="seg-btn" class:on={selected.label === l} style="--c:{LABEL_COLOR[l]}" onclick={() => selected && (selected.label = l)}>{l}</button>
            {/each}
          </div>
        </div>
        <div class="field">
          <span class="flabel">Status</span>
          <div class="seg">
            {#each COLUMNS as c (c.key)}
              <button class="seg-btn" class:on={selected.lane === c.key} onclick={() => selected && (selected.lane = c.key)}>{c.label}</button>
            {/each}
          </div>
        </div>
        <div class="field">
          <span class="flabel">Assignee · {selected.project} members</span>
          <div class="seg seg-wrap">
            {#each membersFor(selected.project) as m (m)}
              <button class="seg-btn person" class:on={selected.assignee === m} onclick={() => selected && (selected.assignee = m)}>
                <Avatar name={m} size={15} /> {m}
              </button>
            {/each}
          </div>
        </div>
        <div class="field">
          <span class="flabel">Story points</span>
          <div class="pt-picker">
            {#each POINT_SCALE as p (p)}
              <button class="pt-btn" class:on={selected.points === p} style="--pc:{pointTone(p)}" onclick={() => selected && (selected.points = p)}>{p}</button>
            {/each}
          </div>
        </div>
        <label class="field grow">
          <span class="flabel">Notes (markdown)</span>
          <textarea class="body-edit" value={selected.body} oninput={(e) => selected && (selected.body = e.currentTarget.value)} placeholder="Markdown notes for this ticket…"></textarea>
        </label>
      {:else}
        <div class="empty">Select a ticket to edit.</div>
      {/if}
    </aside>
  {/if}

  {#if showCalendar}
    <Resizer get={() => calendarWidth} set={(n) => (calendarWidth = n)} dir={-1} min={220} max={400} />
    <div class="calendar-col" style="width:{calendarWidth}px">
      <Calendar />
    </div>
  {/if}

  {#if showInspector || showPreview}
    <Resizer get={() => zoneWidth} set={(n) => (zoneWidth = n)} dir={-1} min={250} max={540} />
    <aside class="rightzone" style="width:{zoneWidth}px">
      {#if showInspector}
        <section class="zone-sec">
          <div class="pane-head"><PanelRight size={13} /> Inspector</div>
          {#if hoveredProject}
            {@const pc = PROJECTS.find((x) => x.name === hoveredProject)}
            <div class="insp-title-row">{hoveredProject}</div>
            <div class="insp-sub">project</div>
            <dl class="insp-rows">
              <dt>Tickets</dt>
              <dd>{pc?.count ?? 0}</dd>
              <dt>Members</dt>
              <dd>{membersFor(hoveredProject).length}</dd>
            </dl>
            <div class="insp-key">Members</div>
            <div class="people">
              {#each membersFor(hoveredProject) as m (m)}<Avatar name={m} size={20} />{/each}
            </div>
          {:else if inspected}
            <div class="insp-title-row">{inspected.title}</div>
            <div class="insp-badges">
              <span class="tag" style="--c:{LABEL_COLOR[inspected.label]}">{inspected.label}</span>
              <span class="lane-badge">{laneLabel(inspected.lane)}</span>
              <span class="pts" style="--pc:{pointTone(inspected.points)}">{inspected.points} <span class="sp">SP</span></span>
            </div>
            <dl class="insp-rows">
              <dt>Project</dt>
              <dd>{inspected.project}</dd>
              <dt>Status</dt>
              <dd>{laneLabel(inspected.lane)}</dd>
              <dt>Assignee</dt>
              <dd class="who"><Avatar name={inspected.assignee} size={16} /> {inspected.assignee}</dd>
              <dt>Points</dt>
              <dd>{inspected.points} SP</dd>
            </dl>
            <div class="insp-key">Project members</div>
            <div class="people">
              {#each membersFor(inspected.project) as m (m)}<Avatar name={m} size={20} />{/each}
            </div>
          {:else}
            <div class="empty">Hover or select a ticket.</div>
          {/if}
        </section>
      {/if}

      {#if showPreview}
        <section class="zone-sec preview-sec">
          <div class="pane-head"><Eye size={13} /> Preview</div>
          {#if inspected && inspected.body.trim()}
            <div class="pv-body"><MarkdownPreview content={inspected.body} /></div>
          {:else if inspected}
            <div class="empty">No notes — press <kbd>e</kbd> to edit.</div>
          {:else}
            <div class="empty">Hover or select a ticket.</div>
          {/if}
        </section>
      {/if}
    </aside>
  {/if}
</div>

{#if ghost && draggedTicket}
  <div class="ghost" style="left:{ghost.x}px; top:{ghost.y}px">{draggedTicket.title}</div>
{/if}

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
    min-height: 40px;
  }
  .card {
    position: relative;
    background: var(--mantle);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 0.6rem;
    cursor: grab;
    user-select: none;
    -webkit-user-select: none;
    touch-action: none;
    transition: border-color 0.12s, transform 0.12s, opacity 0.12s;
  }
  .card:hover {
    border-color: var(--border-strong);
    transform: translateY(-1px);
  }
  .card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }
  .card.dragging {
    opacity: 0.35;
  }
  .card.drop-before::before,
  .card.drop-after::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    height: 2px;
    border-radius: 2px;
    background: var(--accent);
  }
  .card.drop-before::before {
    top: -4px;
  }
  .card.drop-after::after {
    bottom: -4px;
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

  /* floating drag ghost */
  .ghost {
    position: fixed;
    z-index: 9999;
    pointer-events: none;
    transform: translate(-50%, -50%) rotate(-2deg);
    max-width: 220px;
    padding: 0.5rem 0.65rem;
    background: var(--mantle);
    border: 1px solid var(--accent);
    border-radius: 0.5rem;
    font-size: 0.82rem;
    color: var(--text);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.45);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* statusbar (footer) */
  .statusbar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.3rem 0.7rem;
    border-top: 1px solid var(--border);
    background: var(--mantle);
    flex-shrink: 0;
    font-size: 0.72rem;
  }
  .sb-path {
    color: var(--subtext);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 50%;
  }
  .sb-spacer {
    flex: 1;
  }
  .sb-count {
    color: var(--faint);
  }
  .pane-toggles {
    display: flex;
    gap: 0.2rem;
  }
  .pane-toggle {
    display: grid;
    place-items: center;
    width: 26px;
    height: 22px;
    border: none;
    border-radius: 0.4rem;
    background: none;
    color: var(--faint);
    opacity: 0.5;
    cursor: pointer;
    transition: opacity 0.12s, color 0.12s, background 0.12s;
  }
  .pane-toggle:hover {
    opacity: 0.9;
    background: var(--glass-hover);
  }
  .pane-toggle.on {
    opacity: 1;
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 14%, transparent);
  }

  /* preview / edit panes */
  .pane {
    flex-shrink: 0;
    border-left: 1px solid var(--border);
    background: var(--mantle);
    padding: 0.9rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }
  .pane-head {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.64rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--subtext);
  }
  .empty {
    color: var(--faint);
    font-size: 0.8rem;
    margin-top: 0.5rem;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .field.grow {
    flex: 1;
    min-height: 0;
  }
  .flabel {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--faint);
  }
  .f-input {
    width: 100%;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: 0.4rem;
    color: var(--text);
    font: inherit;
    font-size: 0.82rem;
    padding: 0.35rem 0.45rem;
  }
  .f-input:focus {
    outline: none;
    border-color: var(--accent);
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
  .body-edit {
    flex: 1;
    min-height: 150px;
    resize: none;
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
  .lane-badge {
    font-size: 0.62rem;
    color: var(--subtext);
    background: var(--surface-active);
    padding: 0.08rem 0.45rem;
    border-radius: 0.5rem;
  }
  .pv-body {
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  /* Combined inspector + preview zone (inspector on top, preview below) */
  .rightzone {
    flex-shrink: 0;
    border-left: 1px solid var(--border);
    background: var(--mantle);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }
  .zone-sec {
    padding: 0.85rem 0.9rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .zone-sec + .zone-sec {
    border-top: 1px solid var(--border);
  }
  .preview-sec {
    flex: 1;
    min-height: 0;
  }
  .insp-title-row {
    font-weight: 700;
    font-size: 0.95rem;
    color: var(--text);
    line-height: 1.3;
  }
  .insp-sub {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--faint);
  }
  .insp-badges {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
  }
  .insp-rows {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.35rem 0.75rem;
    margin: 0.2rem 0 0;
  }
  .insp-rows dt {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--faint);
  }
  .insp-rows dd {
    margin: 0;
    font-size: 0.8rem;
    color: var(--subtext);
  }
  .insp-rows dd.who {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }
  .insp-key {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--faint);
    margin-top: 0.3rem;
  }
  .people {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }
  kbd {
    font-family: ui-monospace, monospace;
    font-size: 0.7rem;
    background: var(--surface-active);
    border-radius: 0.3rem;
    padding: 0.02rem 0.3rem;
  }

  /* toolbar "New" button */
  .newbtn {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.3rem 0.6rem;
    border-radius: 0.5rem;
    border: 1px solid var(--border);
    background: var(--glass-hover);
    color: var(--text);
    cursor: pointer;
    font: inherit;
    font-size: 0.78rem;
    font-weight: 600;
  }
  .newbtn:hover {
    background: var(--glass-active);
    border-color: var(--accent);
    color: var(--accent);
  }

  /* per-lane add affordance */
  .add-card {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.3rem;
    width: 100%;
    padding: 0.4rem;
    border: 1px dashed var(--border);
    border-radius: 0.5rem;
    background: none;
    color: var(--faint);
    cursor: pointer;
    font: inherit;
    font-size: 0.72rem;
  }
  .add-card:hover {
    border-color: var(--accent);
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 8%, transparent);
  }

  /* segmented pill selectors (replace dropdowns) */
  .seg {
    display: flex;
    gap: 0.3rem;
  }
  .seg-wrap {
    flex-wrap: wrap;
  }
  .seg-btn {
    flex: 1;
    min-width: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.3rem;
    padding: 0.32rem 0.45rem;
    border: 1px solid var(--border);
    border-radius: 0.4rem;
    background: none;
    color: var(--subtext);
    cursor: pointer;
    font: inherit;
    font-size: 0.72rem;
    font-weight: 600;
    text-transform: capitalize;
    white-space: nowrap;
    transition: border-color 0.12s, color 0.12s, background 0.12s;
  }
  .seg-btn:hover {
    border-color: var(--border-strong);
    color: var(--text);
  }
  .seg-btn.on {
    color: var(--c, var(--accent));
    border-color: var(--c, var(--accent));
    background: color-mix(in srgb, var(--c, var(--accent)) 14%, transparent);
  }
  .seg-wrap .seg-btn {
    flex: 0 0 auto;
  }

  /* calendar pane */
  .calendar-col {
    flex-shrink: 0;
    min-width: 0;
    overflow-y: auto;
    border-left: 1px solid var(--border);
    background: var(--mantle);
  }
</style>
