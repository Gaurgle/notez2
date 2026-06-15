<script lang="ts">
  // "ticketz" board, backed by real GitHub issues across the active repos
  // (personal + orgs; the selection lives in the shared repoStore). Issues map
  // to lanes (closed = Done; review/in-progress labels drive the middle lanes;
  // everything else is Backlog). Drag-to-organize is local for now; GitHub
  // write-back for moves is a follow-up (see DESIGN.md). New creates a real issue.
  import { onMount } from "svelte";
  import Avatar from "$lib/components/Avatar.svelte";
  import MachineAvatar from "$lib/components/MachineAvatar.svelte";
  import MarkdownPreview from "$lib/components/MarkdownPreview.svelte";
  import Calendar from "$lib/components/Calendar.svelte";
  import Resizer from "$lib/components/Resizer.svelte";
  import { githubUser, githubCreateIssue } from "$lib/ipc";
  import { repoStore } from "$lib/repos.svelte";
  import { ensureActivity, activityCache, invalidate, loadingRepos } from "$lib/activity.svelte";
  import { SvelteSet } from "svelte/reactivity";
  import type { GhIssue } from "$lib/types";
  import { Plus, Eye, Pencil, PanelRight, CalendarDays, ChevronDown, ChevronRight } from "lucide-svelte";

  /** Short repo name from a full owner/repo key. */
  const shortName = (full: string) => full.split("/").pop() ?? full;

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
    id: number; // synthetic, stable for this load (the board key)
    number: number; // real GitHub issue number
    repo: string;
    url: string;
    title: string;
    lane: Lane;
    label: Label; // colored category, derived from the real labels
    labels: string[]; // real GitHub label names
    assignee: string; // first assignee login, else the author
    avatar: string | null;
    author: string;
    project: string; // == repo (keeps the sidebar code generic)
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

  // --- real issues across the active repos --------------------------------
  let repoNames = $state<string[]>([]); // full owner/repo names (the active set)
  let TICKETS = $state<Ticket[]>([]);
  let me = $state("you");
  let loadError = $state<string | null>(null);
  let synth = 0;
  let fetchToken = 0;

  function laneFor(iss: GhIssue): Lane {
    if (iss.state === "closed") return "done";
    const l = iss.labels.map((x) => x.toLowerCase());
    if (l.some((x) => x.includes("review") || x.includes("qa"))) return "review";
    if (l.some((x) => x.includes("progress") || x.includes("wip") || x.includes("doing")))
      return "progress";
    return "backlog";
  }
  function categoryFor(labels: string[]): Label {
    const l = labels.map((x) => x.toLowerCase());
    if (l.some((x) => x.includes("bug") || x.includes("fix"))) return "bug";
    if (l.some((x) => x.includes("design") || x.includes("ui") || x.includes("ux"))) return "design";
    if (l.some((x) => x.includes("chore") || x.includes("docs") || x.includes("refactor")))
      return "chore";
    return "feature";
  }
  function toTicket(iss: GhIssue): Ticket {
    return {
      id: ++synth,
      number: iss.number,
      repo: iss.repo,
      url: iss.url,
      title: iss.title,
      lane: laneFor(iss),
      label: categoryFor(iss.labels),
      labels: iss.labels,
      assignee: iss.assignees[0] ?? iss.author ?? "you",
      avatar: iss.avatar_url,
      author: iss.author,
      project: iss.repo,
      points: iss.points ?? 3,
      body: iss.body ?? "",
    };
  }
  onMount(async () => {
    try {
      me = (await githubUser()).login;
    } catch {
      /* offline */
    }
    await repoStore.ensure(me);
  });

  // Build the board from the shared activity cache — but only while ticketz is
  // visible (all views stay mounted), debounced, through the shared pool. Cached
  // repos rebuild instantly; new ones stream in via ensureActivity.
  $effect(() => {
    if (!active) return;
    const names = repoStore.activeNames;
    if (repoStore.loading) return;
    repoNames = names;
    const token = ++fetchToken;
    const timer = setTimeout(async () => {
      await ensureActivity(names);
      if (token === fetchToken) rebuildTickets(names);
    }, 250);
    return () => clearTimeout(timer);
  });

  function rebuildTickets(names: string[]) {
    synth = 0;
    loadError = repoStore.error;
    TICKETS = names.flatMap((n) => activityCache.get(n)?.issues ?? []).map(toTicket);
  }

  // True while any active repo is still loading its first activity.
  let loading = $derived(
    repoStore.loading || repoNames.some((n) => loadingRepos.has(n) || !activityCache.has(n))
  );

  // Project members = whoever is actually assigned/authoring in that repo.
  function membersFor(project: string): string[] {
    const set = new Set<string>();
    for (const t of TICKETS) {
      if (t.project !== project) continue;
      if (t.assignee) set.add(t.assignee);
      if (t.author) set.add(t.author);
    }
    if (set.size === 0) set.add(me);
    return [...set];
  }

  let PROJECTS = $derived(
    repoNames.map((full) => ({
      full,
      name: shortName(full),
      count: TICKETS.filter((t) => t.project === full).length,
    }))
  );

  // Sidebar projects grouped by owner (personal first, then orgs), collapsible.
  let projectGroups = $derived.by(() => {
    const byOwner = new Map<string, typeof PROJECTS>();
    for (const p of PROJECTS) {
      const owner = p.full.split("/")[0];
      const list = byOwner.get(owner) ?? [];
      list.push(p);
      byOwner.set(owner, list);
    }
    const groups = [...byOwner.entries()].map(([owner, repos]) => ({
      owner,
      isMe: owner === me,
      repos,
    }));
    groups.sort((a, b) => (a.isMe ? 0 : 1) - (b.isMe ? 0 : 1) || a.owner.localeCompare(b.owner));
    return groups;
  });
  const collapsedOwners = new SvelteSet<string>();
  function toggleOwner(o: string) {
    if (collapsedOwners.has(o)) collapsedOwners.delete(o);
    else collapsedOwners.add(o);
  }

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

  // New-issue composer. Creating writes a real GitHub issue (user-initiated),
  // then reloads the board so the new card reflects the source of truth.
  let draft = $state<{ repo: string; title: string; body: string } | null>(null);
  let creating = $state(false);
  function startNew() {
    draft = { repo: activeProject ?? repoNames[0] ?? "", title: "", body: "" };
    selectedId = null;
    showEdit = true;
  }
  async function createDraft() {
    if (!draft || !draft.title.trim() || !draft.repo) return;
    creating = true;
    try {
      const repo = draft.repo;
      await githubCreateIssue(repo, draft.title.trim(), draft.body);
      draft = null;
      invalidate(repo); // force a refetch of just that repo
      await ensureActivity(repoNames);
      rebuildTickets(repoNames);
    } catch (e) {
      loadError = String(e);
    } finally {
      creating = false;
    }
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
    document.body.style.userSelect = "none"; // no text highlight while dragging
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
    document.body.style.userSelect = "";
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
      {#each projectGroups as g (g.owner)}
        {@const open = !collapsedOwners.has(g.owner)}
        <div class="owner-row">
          <button class="owner-name" onclick={() => toggleOwner(g.owner)} title="Collapse / expand">
            {#if open}<ChevronDown size={12} />{:else}<ChevronRight size={12} />{/if}
            <span class="owner-text">{g.isMe ? "personal" : g.owner}</span>
          </button>
        </div>
        {#if open}
          {#each g.repos as p (p.full)}
            <button
              class="item"
              class:active={activeProject === p.full}
              class:loading={loadingRepos.has(p.full)}
              onclick={() => (activeProject = p.full)}
              onmouseenter={() => (hoveredProject = p.full)}
              onmouseleave={() => hoveredProject === p.full && (hoveredProject = null)}
            >
              <span class="item-label">{p.name}</span>
              <span class="count">{p.count}</span>
            </button>
          {/each}
        {/if}
      {/each}
      {#if projectGroups.length === 0}
        <div class="proj-empty">{loading ? "loading…" : "no active repos — pick some on Home"}</div>
      {/if}
    </nav>
  </aside>
  <Resizer get={() => sidebarWidth} set={(n) => (sidebarWidth = n)} dir={1} min={160} max={320} />

  <div class="main">
    <div class="viewbar">
      <span class="title">Ticketz</span>
      <span class="counts">
        {visible.length} issues{activeProject ? ` · ${shortName(activeProject)}` : ""} ·
        {loading ? "loading…" : loadError ? "offline" : `${repoNames.length} repos`}
      </span>
      <div class="spacer"></div>
      <button class="newbtn" onclick={startNew} title="New issue">
        <Plus size={15} /> New
      </button>
    </div>

    <div class="panes">
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
                    <span class="num">{t.repo} #{t.number}</span>
                  </span>
                  <span class="pts" style="--pc:{pointTone(t.points)}" title="{t.points} story points">
                    {t.points} <span class="sp">SP</span>
                  </span>
                </div>
                <div class="card-title">{t.title}</div>
                <div class="card-foot">
                  <span class="proj">{shortName(t.project)}</span>
                  <span class="assignee">
                    {#if t.body.trim()}<span class="has-notes" title="Has notes">●</span>{/if}
                    <Avatar name={t.assignee} src={t.avatar} size={18} />
                  </span>
                </div>
              </div>
            {/each}
            {#if inLane(col.key).length === 0}
              <div class="lane-empty">{loading ? "…" : "empty"}</div>
            {/if}
            <button class="add-card" onclick={startNew}>
              <Plus size={13} /> New issue
            </button>
          </div>
        </section>
      {/each}
      </div>

      {#if showEdit}
    <Resizer get={() => editWidth} set={(n) => (editWidth = n)} dir={-1} min={260} max={520} />
    <aside class="pane" style="width:{editWidth}px">
      {#if draft}
        <div class="pane-head"><Plus size={13} /> New issue</div>
        <div class="field">
          <span class="flabel">Repository</span>
          <div class="seg seg-wrap">
            {#each repoNames as r (r)}
              <button class="seg-btn" class:on={draft.repo === r} onclick={() => draft && (draft.repo = r)}>{shortName(r)}</button>
            {/each}
          </div>
        </div>
        <label class="field">
          <span class="flabel">Title</span>
          <input class="f-input" bind:value={draft.title} placeholder="Issue title" />
        </label>
        <label class="field grow">
          <span class="flabel">Body (markdown)</span>
          <textarea class="body-edit" bind:value={draft.body} placeholder="Describe the issue…"></textarea>
        </label>
        <div class="draft-actions">
          <button class="ghost-btn" onclick={() => (draft = null)} disabled={creating}>Cancel</button>
          <button class="newbtn" onclick={createDraft} disabled={creating || !draft.title.trim() || !draft.repo}>
            {creating ? "Creating…" : `Create in ${draft.repo ? shortName(draft.repo) : "…"}`}
          </button>
        </div>
        {#if loadError}<div class="draft-err">{loadError}</div>{/if}
      {:else if selected}
        <div class="pane-head"><Pencil size={13} /> {selected.repo} #{selected.number} <span class="local-hint">local edits</span></div>
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
          <span class="flabel">Assignee · {shortName(selected.project)} members</span>
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
            {@const pc = PROJECTS.find((x) => x.full === hoveredProject)}
            <div class="insp-title-row">{shortName(hoveredProject)}</div>
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
              <dd>{shortName(inspected.project)}</dd>
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
  /* owner group header in the projects sidebar */
  .owner-row {
    display: flex;
    align-items: center;
    padding: 0.4rem 0.5rem 0.15rem;
  }
  .owner-name {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    font: inherit;
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--subtext);
    font-weight: 700;
  }
  .owner-name:hover {
    color: var(--text);
  }
  .owner-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  /* per-repo loading bar */
  .item.loading {
    position: relative;
    overflow: hidden;
  }
  .item.loading::after {
    content: "";
    position: absolute;
    left: 0;
    bottom: 0;
    height: 2px;
    width: 40%;
    background: var(--accent);
    border-radius: 2px;
    animation: repo-load 0.9s ease-in-out infinite;
  }
  @keyframes repo-load {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(350%);
    }
  }
  .title {
    font-weight: 700;
    color: var(--accent);
    font-size: 0.95rem;
  }
  /* board + side panes share a row; the statusbar sits below it, full width */
  .panes {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .board {
    flex: 1;
    min-width: 0;
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
  /* .pane-toggles / .pane-toggle come from app.css (shared with notes/todoz) */

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

  /* empty lane hint + new-issue composer */
  .lane-empty {
    text-align: center;
    color: var(--faint);
    font-size: 0.7rem;
    padding: 0.5rem 0;
    opacity: 0.7;
  }
  .proj-empty {
    color: var(--faint);
    font-size: 0.72rem;
    padding: 0.4rem 0.5rem;
    line-height: 1.4;
  }
  .draft-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
    margin-top: 0.5rem;
  }
  .ghost-btn {
    padding: 0.3rem 0.6rem;
    border-radius: 0.5rem;
    border: 1px solid var(--border);
    background: none;
    color: var(--subtext);
    cursor: pointer;
    font: inherit;
    font-size: 0.78rem;
  }
  .ghost-btn:hover {
    color: var(--text);
    border-color: var(--border-strong);
  }
  .newbtn:disabled,
  .ghost-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .draft-err {
    margin-top: 0.5rem;
    color: var(--danger);
    font-size: 0.72rem;
    word-break: break-word;
  }
  .local-hint {
    margin-left: auto;
    font-size: 0.6rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--faint);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 0.05rem 0.35rem;
  }
</style>
