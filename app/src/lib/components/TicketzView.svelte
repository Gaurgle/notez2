<script lang="ts">
  // Mock "ticketz" board — a quick Trello-style preview. Real tickets will be
  // repo-bound files synced via git (see DESIGN.md). Data here is placeholder.
  import Avatar from "$lib/components/Avatar.svelte";
  import MachineAvatar from "$lib/components/MachineAvatar.svelte";
  import Resizer from "$lib/components/Resizer.svelte";
  import { Plus, Filter } from "lucide-svelte";

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
  }

  const LABEL_COLOR: Record<Label, string> = {
    feature: "var(--accent-public)",
    bug: "var(--danger)",
    design: "var(--accent-personal)",
    chore: "var(--accent-global)",
  };

  // Fibonacci story points (Scrum-style): bigger number = bigger task.
  const POINT_SCALE = [1, 2, 3, 5, 8, 13];
  function pointTone(p: number): string {
    if (p <= 2) return "var(--accent-public)"; // small
    if (p <= 5) return "var(--accent-global)"; // medium
    return "var(--danger)"; // large
  }
  function bumpPoints(t: Ticket, dir: 1 | -1) {
    const i = POINT_SCALE.indexOf(t.points);
    const next = Math.min(POINT_SCALE.length - 1, Math.max(0, (i < 0 ? 2 : i) + dir));
    t.points = POINT_SCALE[next];
  }

  let TICKETS = $state<Ticket[]>([
    { id: 101, title: "Calendar date encoding (@date token)", lane: "backlog", label: "feature", assignee: "you", project: "notez2", points: 5 },
    { id: 102, title: "Ticket files synced via git", lane: "backlog", label: "feature", assignee: "alex", project: "notez2", points: 8 },
    { id: 103, title: "Extended task states: deferred / scrapped", lane: "backlog", label: "feature", assignee: "mira", project: "notez2", points: 3 },
    { id: 104, title: "repoz: broad repo status scan", lane: "backlog", label: "chore", assignee: "sam", project: "repoz", points: 5 },
    { id: 201, title: "GitHub OAuth device flow", lane: "progress", label: "feature", assignee: "you", project: "spaze", points: 8 },
    { id: 202, title: "Contextual inspector + cross counts", lane: "progress", label: "feature", assignee: "nora", project: "notez2", points: 5 },
    { id: 203, title: "epoz: git handling wrapper", lane: "progress", label: "feature", assignee: "kai", project: "epoz", points: 13 },
    { id: 301, title: "Unify toolbars on lucide icons", lane: "review", label: "design", assignee: "mira", project: "notez2", points: 3 },
    { id: 302, title: "Duplicate-path dedupe in collect_all", lane: "review", label: "bug", assignee: "you", project: "notez2", points: 2 },
    { id: 401, title: "Split markdown preview pane", lane: "done", label: "feature", assignee: "alex", project: "notez2", points: 5 },
    { id: 402, title: "todoz tree connectors", lane: "done", label: "design", assignee: "nora", project: "notez2", points: 2 },
    { id: 403, title: "Workspace + serde restructure", lane: "done", label: "chore", assignee: "you", project: "notez2", points: 13 },
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
  let sidebarWidth = $state(190);
  let visible = $derived(
    activeProject === null ? TICKETS : TICKETS.filter((t) => t.project === activeProject)
  );

  function inLane(lane: Lane): Ticket[] {
    return visible.filter((t) => t.lane === lane);
  }

  // Drag a ticket between lanes.
  let draggingId = $state<number | null>(null);
  let dragOverLane = $state<Lane | null>(null);

  function dropOn(lane: Lane) {
    const t = TICKETS.find((x) => x.id === draggingId);
    if (t) t.lane = lane;
    draggingId = null;
    dragOverLane = null;
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
        class:dragover={dragOverLane === col.key}
        ondragover={(e) => {
          e.preventDefault();
          dragOverLane = col.key;
        }}
        ondragleave={(e) => {
          if (!e.currentTarget.contains(e.relatedTarget as Node)) dragOverLane = null;
        }}
        ondrop={(e) => {
          e.preventDefault();
          dropOn(col.key);
        }}
      >
        <header class="lane-head">
          <span class="lane-label">{col.label}</span>
          <span class="lane-count">{inLane(col.key).length}</span>
        </header>
        <div class="cards">
          {#each inLane(col.key) as t (t.id)}
            <article
              class="card"
              class:dragging={draggingId === t.id}
              draggable="true"
              ondragstart={() => (draggingId = t.id)}
              ondragend={() => {
                draggingId = null;
                dragOverLane = null;
              }}
            >
              <div class="card-top">
                <span class="tag" style="--c:{LABEL_COLOR[t.label]}">{t.label}</span>
                <span class="num">#{t.id}</span>
              </div>
              <div class="card-title">{t.title}</div>
              <div class="card-foot">
                <span class="proj">{t.project}</span>
                <span class="foot-right">
                  <button
                    class="pts"
                    style="--pc:{pointTone(t.points)}"
                    title="Story points — click to size up, right-click down"
                    onclick={(e) => {
                      e.stopPropagation();
                      bumpPoints(t, 1);
                    }}
                    oncontextmenu={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      bumpPoints(t, -1);
                    }}
                  >
                    {t.points}
                  </button>
                  <Avatar name={t.assignee} size={20} />
                </span>
              </div>
            </article>
          {/each}
        </div>
      </section>
    {/each}
    </div>
  </div>
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
    background: color-mix(in srgb, var(--accent) 8%, transparent);
    border-color: color-mix(in srgb, var(--accent) 45%, transparent);
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
  .card.dragging {
    opacity: 0.45;
    border-color: var(--accent);
  }
  .card-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.4rem;
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
  }
  .num {
    font-size: 0.66rem;
    color: var(--faint);
  }
  .card-title {
    font-size: 0.84rem;
    line-height: 1.35;
    color: var(--text);
    margin-bottom: 0.5rem;
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
  .foot-right {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
  /* story-point bubble — color + value convey task size */
  .pts {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 19px;
    height: 19px;
    padding: 0 0.25rem;
    border-radius: 50%;
    border: none;
    font: inherit;
    font-size: 0.64rem;
    font-weight: 800;
    color: var(--pc);
    background: color-mix(in srgb, var(--pc) 18%, transparent);
    cursor: pointer;
    font-variant-numeric: tabular-nums;
  }
  .pts:hover {
    background: color-mix(in srgb, var(--pc) 30%, transparent);
  }
</style>
