<script lang="ts">
  // Mock "home" dashboard on a draggable/resizable Gridstack board.
  // The clock, the calendar month, and the weather (Open-Meteo) are real.
  import { onMount, onDestroy } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { GridStack } from "gridstack";
  import "gridstack/dist/gridstack.min.css";
  import Avatar from "$lib/components/Avatar.svelte";
  import MachineAvatar from "$lib/components/MachineAvatar.svelte";
  import WeatherWidget from "$lib/components/WeatherWidget.svelte";
  import { hashStr } from "$lib/mock";
  import {
    CloudSun,
    GitCommitHorizontal,
    FolderGit2,
    Users,
    Megaphone,
    FileText,
    ListChecks,
    KanbanSquare,
    CalendarDays,
    Flame,
    RotateCcw,
  } from "lucide-svelte";

  let { active = true }: { active?: boolean } = $props();

  let now = $state(new Date());
  $effect(() => {
    const id = setInterval(() => (now = new Date()), 1000);
    return () => clearInterval(id);
  });

  let greeting = $derived(
    now.getHours() < 12 ? "Good morning" : now.getHours() < 18 ? "Good afternoon" : "Good evening"
  );
  let clock = $derived(
    now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" })
  );
  let dateStr = $derived(
    now.toLocaleDateString([], { weekday: "long", month: "long", day: "numeric" })
  );

  function spark(seed: string): number[] {
    return Array.from({ length: 9 }, (_, i) => 22 + (hashStr(seed + ":" + i) % 78));
  }
  const STATS = [
    { label: "Notes", value: 128, trend: "+6 this week", icon: FileText, color: "var(--accent-local)" },
    { label: "Todos", value: 47, trend: "12 done", icon: ListChecks, color: "var(--accent-public)" },
    { label: "Tickets", value: 12, trend: "3 in review", icon: KanbanSquare, color: "var(--accent-personal)" },
    { label: "Commits", value: 142, trend: "this quarter", icon: GitCommitHorizontal, color: "var(--accent-global)" },
  ];

  const WD = ["M", "T", "W", "T", "F", "S", "S"];
  let monthName = $derived(now.toLocaleDateString([], { month: "long", year: "numeric" }));
  let calCells = $derived.by(() => {
    const y = now.getFullYear();
    const m = now.getMonth();
    const lead = (new Date(y, m, 1).getDay() + 6) % 7;
    const days = new Date(y, m + 1, 0).getDate();
    const cells: { day: number | null; event: boolean }[] = [];
    for (let i = 0; i < lead; i++) cells.push({ day: null, event: false });
    for (let d = 1; d <= days; d++) cells.push({ day: d, event: hashStr(`${m}:${d}`) % 5 === 0 });
    return cells;
  });
  let todayNum = $derived(now.getDate());

  const grid: number[][] = Array.from({ length: 16 }, (_, w) =>
    Array.from({ length: 7 }, (_, d) => {
      const v = hashStr(`${w}:${d}`) % 11;
      return v > 8 ? 4 : v > 6 ? 3 : v > 4 ? 2 : v > 1 ? 1 : 0;
    })
  );

  const REPOS = [
    { name: "notez2", status: "dirty", detail: "3 uncommitted" },
    { name: "spaze", status: "ahead", detail: "2 ahead" },
    { name: "repoz", status: "clean", detail: "up to date" },
    { name: "epoz", status: "clean", detail: "up to date" },
  ];
  const STATUS_COLOR: Record<string, string> = {
    clean: "var(--accent-public)",
    dirty: "var(--accent-global)",
    ahead: "var(--accent-local)",
  };
  const REPO_COLOR: Record<string, string> = {
    notez2: "var(--accent-personal)",
    spaze: "var(--accent-local)",
    repoz: "var(--accent-public)",
    epoz: "var(--accent-global)",
  };
  // Per-repo metadata for the repoz-style commit listing: working-tree path
  // and how many commits the local branch trails its upstream.
  const REPO_META: Record<string, { path: string; behind: number }> = {
    notez2: { path: "~/Repos/notez2", behind: 11 },
    spaze: { path: "~/Repos/spaze", behind: 2 },
    repoz: { path: "~/Repos/repoz", behind: 0 },
    epoz: { path: "~/Repos/epoz", behind: 1 },
  };

  const TEAM = [
    { name: "you", online: true },
    { name: "alex", online: true },
    { name: "mira", online: false },
    { name: "sam", online: true },
    { name: "nora", online: false },
    { name: "kai", online: true },
  ];

  const NEWS = [
    { title: "notez2 desktop alpha is live", time: "2h", tag: "release" },
    { title: "Spaze self-hosting guide published", time: "1d", tag: "docs" },
    { title: "All-hands Friday 15:00", time: "2d", tag: "team" },
  ];

  const COMMITS = [
    { hash: "a3f9c2e", msg: "fix(core): stop personal notes duplicating into global scope", repo: "notez2", by: "you", time: "2h" },
    { hash: "6f68822", msg: "feat(app): unify toolbars, footer indicators, + button", repo: "notez2", by: "alex", time: "5h" },
    { hash: "cfc5eae", msg: "feat(app): contextual inspector with contributors", repo: "notez2", by: "you", time: "7h" },
    { hash: "1b4d09a", msg: "feat(app): split preview, resizable panes, vim mode", repo: "notez2", by: "mira", time: "1d" },
    { hash: "9c2a7f1", msg: "feat(client): sidebar nav with hjkl + Enter", repo: "spaze", by: "sam", time: "1d" },
    { hash: "4e7b1d8", msg: "feat(server): GitHub device-flow auth", repo: "spaze", by: "you", time: "1d" },
    { hash: "e8b3d40", msg: "chore: broad dirty-tree scan across repos", repo: "repoz", by: "sam", time: "2d" },
    { hash: "7a1f5c3", msg: "feat: epoz git-handling wrapper skeleton", repo: "epoz", by: "kai", time: "2d" },
    { hash: "b2d6e90", msg: "docs(design): calendar + ticket roadmap", repo: "notez2", by: "you", time: "2d" },
    { hash: "3c9a0f7", msg: "fix(todoz): tree connectors off-by-one", repo: "notez2", by: "nora", time: "3d" },
    { hash: "5f8c12b", msg: "feat(app): markdown read view + code highlighting", repo: "notez2", by: "alex", time: "3d" },
    { hash: "d41a6e2", msg: "refactor: split notez2 into a cargo workspace", repo: "notez2", by: "you", time: "4d" },
    { hash: "8b3f7a0", msg: "feat(commands): slash command parser", repo: "spaze", by: "mira", time: "4d" },
    { hash: "2e5d9c4", msg: "test: todoz parse/serialize round-trip", repo: "notez2", by: "sam", time: "5d" },
    { hash: "f07b3a1", msg: "chore(repoz): color unpushed-commit summary", repo: "repoz", by: "kai", time: "5d" },
    { hash: "6d2c8e5", msg: "feat(app): projects, sync, migration, note tags", repo: "notez2", by: "you", time: "6d" },
    { hash: "a9e4b07", msg: "feat(storage): SQLite FTS5 search", repo: "spaze", by: "alex", time: "1w" },
    { hash: "c13f6d8", msg: "docs: README + scope model", repo: "notez2", by: "nora", time: "1w" },
  ];

  type Commit = (typeof COMMITS)[number] & { add: number; del: number };
  type CommitGroup = { repo: string; path: string; behind: number; commits: Commit[] };
  // Derive a stable +added/−removed churn for each commit from its hash so the
  // repoz-style diff stats stay constant across renders without real git data.
  function diffStat(hash: string): { add: number; del: number } {
    const add = 1 + (hashStr(hash + ":add") % 420);
    // Roughly a third of commits are pure additions (no deletions), like the CLI.
    const del = hashStr(hash) % 3 === 0 ? 0 : 1 + (hashStr(hash + ":del") % 260);
    return { add, del };
  }
  const COMMIT_GROUPS: CommitGroup[] = (() => {
    const order: string[] = [];
    const map: Record<string, Commit[]> = {};
    for (const c of COMMITS) {
      if (!map[c.repo]) {
        map[c.repo] = [];
        order.push(c.repo);
      }
      map[c.repo].push({ ...c, ...diffStat(c.hash) });
    }
    return order.map((repo) => ({
      repo,
      path: REPO_META[repo]?.path ?? `~/Repos/${repo}`,
      behind: REPO_META[repo]?.behind ?? 0,
      commits: map[repo],
    }));
  })();

  // --- Project visibility (sidebar) --------------------------------------
  // Which projects are visualized in the dashboard, persisted across sessions.
  const ALL_PROJECTS = ["notez2", "spaze", "repoz", "epoz"];
  const PROJ_KEY = "notez-dash-projects-v1";
  function loadSelectedProjects(): string[] {
    try {
      const s = localStorage.getItem(PROJ_KEY);
      if (s) return JSON.parse(s);
    } catch {
      /* ignore */
    }
    return [...ALL_PROJECTS];
  }
  const selectedProjects = new SvelteSet<string>(loadSelectedProjects());
  $effect(() => {
    localStorage.setItem(PROJ_KEY, JSON.stringify([...selectedProjects]));
  });
  function toggleProject(p: string) {
    if (selectedProjects.has(p)) selectedProjects.delete(p);
    else selectedProjects.add(p);
  }

  let visibleCommitGroups = $derived(COMMIT_GROUPS.filter((g) => selectedProjects.has(g.repo)));
  let visibleRepos = $derived(REPOS.filter((r) => selectedProjects.has(r.name)));
  let totalBehind = $derived(visibleCommitGroups.reduce((n, g) => n + g.behind, 0));

  // --- Gridstack ---------------------------------------------------------
  // Spread the gs-* layout attributes via a helper (keeps them off the typed
  // element props so svelte-check doesn't reject the custom attributes).
  function gs(o: {
    id: string;
    x: number;
    y: number;
    w: number;
    h: number;
    minW?: number;
    minH?: number;
    maxH?: number;
  }): Record<string, string | number> {
    const a: Record<string, string | number> = {
      "gs-id": o.id,
      "gs-x": o.x,
      "gs-y": o.y,
      "gs-w": o.w,
      "gs-h": o.h,
    };
    if (o.minW !== undefined) a["gs-min-w"] = o.minW;
    if (o.minH !== undefined) a["gs-min-h"] = o.minH;
    if (o.maxH !== undefined) a["gs-max-h"] = o.maxH;
    return a;
  }

  const LAYOUT_KEY = "notez-dash-layout-v2";
  let gridEl: HTMLElement;
  let gridStack: GridStack | undefined;

  onMount(() => {
    gridStack = GridStack.init(
      {
        column: 12,
        cellHeight: 76,
        margin: 11,
        float: false, // gravity-pack: dragging never leaves holes behind
        animate: true, // smooth reflow of neighbours
        draggable: { cancel: ".no-drag" },
        resizable: { handles: "n, e, s, w, se, sw, ne, nw" },
        columnOpts: { breakpoints: [{ w: 760, c: 1 }] },
      },
      gridEl
    );

    const saved = localStorage.getItem(LAYOUT_KEY);
    if (saved) {
      try {
        gridStack.load(JSON.parse(saved), false);
      } catch {
        /* ignore a corrupt saved layout */
      }
    }
    gridStack.on("change", () => {
      try {
        localStorage.setItem(LAYOUT_KEY, JSON.stringify(gridStack?.save(false)));
      } catch {
        /* storage may be unavailable */
      }
    });
  });

  onDestroy(() => gridStack?.destroy(false));

  function resetLayout() {
    localStorage.removeItem(LAYOUT_KEY);
    location.reload();
  }
</script>

<div class="dash">
  <aside class="sidebar">
    <div class="brand">
      <MachineAvatar />
      <span class="brand-name">home</span>
    </div>
    <nav class="group">
      <div class="group-label">Projects</div>
      {#each ALL_PROJECTS as p (p)}
        <button class="item" class:active={selectedProjects.has(p)} onclick={() => toggleProject(p)}>
          <span class="cbox" class:on={selectedProjects.has(p)}></span>
          <span class="item-label">{p}</span>
        </button>
      {/each}
    </nav>
  </aside>

  <div class="main">
    <div class="inner">
      <header class="hero">
      <div class="greet">
        <div class="hello">{greeting}, you</div>
        <div class="date">{dateStr}</div>
      </div>
      <div class="clock">{clock}</div>
      <button class="reset" onclick={resetLayout} title="Reset widget layout" aria-label="Reset layout">
        <RotateCcw size={14} />
      </button>
    </header>

    <div class="grid-stack" bind:this={gridEl}>
      {#each STATS as s, i (s.label)}
        <div
          class="grid-stack-item"
          {...gs({ id: `tile-${s.label}`, x: i * 3, y: 0, w: 3, h: 1, minW: 2, minH: 1, maxH: 2 })}
        >
          <div class="grid-stack-item-content card tile">
            <div class="tile-ico" style="--c:{s.color}"><s.icon size={17} /></div>
            <div class="tile-text">
              <div class="tile-num">{s.value}</div>
              <div class="tile-label">{s.label} · <span class="trend">{s.trend}</span></div>
            </div>
            <div class="spark" style="--c:{s.color}">
              {#each spark(s.label) as h (h)}<span style="height:{h}%"></span>{/each}
            </div>
          </div>
        </div>
      {/each}

      <div class="grid-stack-item" {...gs({ id: "calendar", x: 0, y: 1, w: 4, h: 3, minW: 3, minH: 3 })}>
        <div class="grid-stack-item-content card">
          <div class="card-head"><CalendarDays size={13} /> {monthName}</div>
          <div class="cal-wd">{#each WD as w, i (i)}<span>{w}</span>{/each}</div>
          <div class="cal-grid">
            {#each calCells as c, i (i)}
              <div class="cal-cell" class:empty={c.day === null} class:today={c.day === todayNum}>
                {#if c.day}{c.day}{#if c.event}<span class="cal-ev"></span>{/if}{/if}
              </div>
            {/each}
          </div>
        </div>
      </div>

      <div class="grid-stack-item" {...gs({ id: "git", x: 4, y: 1, w: 4, h: 3, minW: 3, minH: 2 })}>
        <div class="grid-stack-item-content card">
          <div class="card-head"><GitCommitHorizontal size={13} /> Git activity</div>
          <div class="heatmap">
            {#each grid as week, w (w)}
              <div class="week">{#each week as lvl, d (d)}<span class="cell lvl{lvl}"></span>{/each}</div>
            {/each}
          </div>
          <div class="git-foot">
            <span><Flame size={12} /> 7-day streak</span>
            <span class="legend">Less <span class="cell lvl1"></span><span class="cell lvl2"></span><span class="cell lvl3"></span><span class="cell lvl4"></span> More</span>
          </div>
        </div>
      </div>

      <div class="grid-stack-item" {...gs({ id: "weather", x: 8, y: 1, w: 4, h: 3, minW: 3, minH: 2 })}>
        <div class="grid-stack-item-content card weather-card">
          <div class="card-head"><CloudSun size={13} /> Weather</div>
          <WeatherWidget />
        </div>
      </div>

      <div class="grid-stack-item" {...gs({ id: "repos", x: 0, y: 4, w: 4, h: 3, minW: 2, minH: 2 })}>
        <div class="grid-stack-item-content card">
          <div class="card-head"><FolderGit2 size={13} /> Repos</div>
          <ul class="list">
            {#each visibleRepos as r (r.name)}
              <li>
                <span class="sdot" style="--c:{STATUS_COLOR[r.status]}"></span>
                <span class="li-name">{r.name}</span>
                <span class="li-detail">{r.detail}</span>
              </li>
            {/each}
          </ul>
        </div>
      </div>

      <div class="grid-stack-item" {...gs({ id: "news", x: 4, y: 4, w: 4, h: 3, minW: 2, minH: 2 })}>
        <div class="grid-stack-item-content card">
          <div class="card-head"><Megaphone size={13} /> Company news</div>
          <ul class="list">
            {#each NEWS as n (n.title)}
              <li class="news">
                <span class="news-tag">{n.tag}</span>
                <span class="li-name">{n.title}</span>
                <span class="li-detail">{n.time}</span>
              </li>
            {/each}
          </ul>
        </div>
      </div>

      <div class="grid-stack-item" {...gs({ id: "team", x: 8, y: 4, w: 4, h: 3, minW: 2, minH: 2 })}>
        <div class="grid-stack-item-content card">
          <div class="card-head"><Users size={13} /> Team</div>
          <div class="team">
            {#each TEAM as m (m.name)}
              <span class="av" class:online={m.online} title={m.name}><Avatar name={m.name} size={26} /></span>
            {/each}
          </div>
        </div>
      </div>

      <div class="grid-stack-item" {...gs({ id: "commits", x: 0, y: 7, w: 12, h: 5, minW: 4, minH: 3 })}>
        <div class="grid-stack-item-content card">
          <div class="card-head"><GitCommitHorizontal size={13} /> Recent commits</div>
          <div class="repoz">
            {#each visibleCommitGroups as g (g.repo)}
              <div class="repo-block">
                <div class="repo-line">
                  <span class="proj-dot" style="--c:{REPO_COLOR[g.repo] ?? 'var(--subtext)'}"></span>
                  <span class="repo-name">{g.repo}</span>
                  <span class="repo-path">{g.path}</span>
                  <span class="leader"></span>
                  {#if g.behind > 0}
                    <span class="behind">{g.behind} behind</span>
                  {:else}
                    <span class="clean">up to date</span>
                  {/if}
                </div>
                {#each g.commits as c (c.hash)}
                  <div class="crow">
                    <span class="hash">{c.hash}</span>
                    <span class="cmsg">{c.msg}</span>
                    <span class="leader"></span>
                    <span class="stats">
                      <span class="add">+{c.add}</span>
                      {#if c.del > 0}<span class="del">−{c.del}</span>{/if}
                    </span>
                    <span class="author">{c.by}</span>
                  </div>
                {/each}
              </div>
            {/each}
            <div class="repoz-foot">{totalBehind} behind</div>
          </div>
        </div>
      </div>
    </div>
    </div>
  </div>
</div>

<style>
  .dash {
    height: 100%;
    display: flex;
    background: var(--base);
  }
  .main {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 1rem;
  }
  .inner {
    width: 100%;
    max-width: 1320px;
    margin: 0 auto;
  }

  /* Project picker sidebar — choose which projects the dashboard visualizes. */
  .sidebar {
    width: 185px;
    flex-shrink: 0;
    background: rgba(20, 20, 32, var(--sidebar-glass-alpha));
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
    padding: 0.25rem 0.5rem 0.35rem;
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
    color: var(--text);
  }
  .item:not(.active) .item-label {
    color: var(--faint);
  }
  .cbox {
    width: 14px;
    height: 14px;
    border-radius: 0.3rem;
    border: 1.5px solid var(--border-strong, var(--subtext));
    flex-shrink: 0;
    position: relative;
    transition: background 0.12s, border-color 0.12s;
  }
  .cbox.on {
    background: var(--accent);
    border-color: var(--accent);
  }
  .cbox.on::after {
    content: "";
    position: absolute;
    left: 4px;
    top: 1px;
    width: 3px;
    height: 7px;
    border: solid var(--base);
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
  }

  .hero {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding: 0.7rem 1.1rem;
    margin-bottom: 0; /* the grid's own top margin is the only gap */
    border: 1px solid var(--border);
    border-radius: 0.7rem;
    background:
      radial-gradient(120% 160% at 100% 0%, rgba(203, 166, 247, 0.13), transparent 55%),
      var(--mantle);
  }
  .hello {
    font-size: 1.3rem;
    font-weight: 700;
    color: var(--text);
  }
  .date {
    color: var(--subtext);
    font-size: 0.8rem;
  }
  .clock {
    margin-left: auto;
    font-size: 2.1rem;
    font-weight: 700;
    line-height: 1;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.01em;
  }
  .reset {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 0.5rem;
    border: 1px solid var(--border);
    background: var(--glass-hover);
    color: var(--faint);
    cursor: pointer;
  }
  .reset:hover {
    background: var(--glass-active);
    color: var(--text);
  }

  /* Gridstack item = a card. The whole card is the drag handle (content is
     non-interactive), so kill text selection and show a grab cursor. */
  .grid-stack-item-content.card {
    inset: 0;
    background: var(--mantle);
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    padding: 0.7rem;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    /* normal pointer in the body — only becomes "grabbing" once you drag, and
       the edges/corners show their own move/resize cursors on hover. */
    cursor: default;
    user-select: none;
    -webkit-user-select: none;
    /* slight depth — a soft drop shadow + faint top highlight */
    box-shadow:
      0 1px 0 rgba(255, 255, 255, 0.03) inset,
      0 4px 14px rgba(0, 0, 0, 0.28);
    transition: border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease;
  }
  .grid-stack-item-content.card:hover {
    border-color: color-mix(in srgb, var(--accent) 28%, var(--border));
    box-shadow:
      0 1px 0 rgba(255, 255, 255, 0.05) inset,
      0 6px 20px rgba(0, 0, 0, 0.36);
  }
  /* Smooth move/resize animation for every item. */
  :global(.grid-stack-item) {
    transition: transform 0.18s cubic-bezier(0.22, 0.61, 0.36, 1),
      width 0.18s cubic-bezier(0.22, 0.61, 0.36, 1),
      height 0.18s cubic-bezier(0.22, 0.61, 0.36, 1);
  }
  :global(.grid-stack-item.ui-draggable-dragging),
  :global(.grid-stack-item.ui-resizable-resizing) {
    transition: none;
    z-index: 100;
  }
  :global(.grid-stack-item.ui-draggable-dragging .card) {
    cursor: grabbing;
    border-color: var(--accent);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.4);
  }
  .card-head {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.64rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--subtext);
    margin-bottom: 0.6rem;
    flex-shrink: 0;
  }

  /* Resize handles — thin, rounded, sleek; fade in ~0.3s after hover. */
  :global(.grid-stack-item > .ui-resizable-handle) {
    opacity: 0;
    background-image: none;
    transition: opacity 0.18s ease; /* quick fade-out */
  }
  :global(.grid-stack-item:hover > .ui-resizable-handle) {
    opacity: 1;
    transition: opacity 0.22s ease 0.3s; /* delayed fade-in */
  }
  /* corner grips: tiny rounded dots (all four corners) */
  :global(.grid-stack-item > .ui-resizable-se),
  :global(.grid-stack-item > .ui-resizable-sw),
  :global(.grid-stack-item > .ui-resizable-ne),
  :global(.grid-stack-item > .ui-resizable-nw) {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--accent) 85%, transparent);
    box-shadow: 0 0 0 2px var(--mantle);
  }
  :global(.grid-stack-item > .ui-resizable-se) {
    right: 5px;
    bottom: 5px;
  }
  :global(.grid-stack-item > .ui-resizable-sw) {
    left: 5px;
    bottom: 5px;
  }
  :global(.grid-stack-item > .ui-resizable-ne) {
    right: 5px;
    top: 5px;
  }
  :global(.grid-stack-item > .ui-resizable-nw) {
    left: 5px;
    top: 5px;
  }
  /* edge grips: thin rounded pills */
  :global(.grid-stack-item > .ui-resizable-e),
  :global(.grid-stack-item > .ui-resizable-w) {
    width: 2px;
    height: 22px;
    top: 50%;
    margin-top: -11px;
    border-radius: 2px;
    background: color-mix(in srgb, var(--accent) 55%, transparent);
  }
  :global(.grid-stack-item > .ui-resizable-e) {
    right: 3px;
  }
  :global(.grid-stack-item > .ui-resizable-w) {
    left: 3px;
  }
  :global(.grid-stack-item > .ui-resizable-s),
  :global(.grid-stack-item > .ui-resizable-n) {
    width: 22px;
    height: 2px;
    left: 50%;
    margin-left: -11px;
    border-radius: 2px;
    background: color-mix(in srgb, var(--accent) 55%, transparent);
  }
  :global(.grid-stack-item > .ui-resizable-s) {
    bottom: 3px;
  }
  :global(.grid-stack-item > .ui-resizable-n) {
    top: 3px;
  }
  /* drag placeholder: a soft rounded ghost where the widget will land */
  :global(.grid-stack > .grid-stack-placeholder > .placeholder-content) {
    border: 1.5px dashed color-mix(in srgb, var(--accent) 55%, transparent);
    border-radius: 0.7rem;
    background: color-mix(in srgb, var(--accent) 9%, transparent);
    margin: 0;
  }

  /* Tiles — higher specificity than .grid-stack-item-content.card so the
     row layout wins (otherwise the column default clips the number). */
  .grid-stack-item-content.card.tile {
    flex-direction: row;
    align-items: center;
    gap: 0.7rem;
    padding: 0.7rem 0.85rem;
  }
  .tile-ico {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 0.55rem;
    color: var(--c);
    background: color-mix(in srgb, var(--c) 16%, transparent);
    flex-shrink: 0;
  }
  .tile-text {
    min-width: 0;
  }
  .tile-num {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text);
    line-height: 1.05;
  }
  .tile-label {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--faint);
    white-space: nowrap;
  }
  .trend {
    color: var(--c);
  }
  .spark {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 34px;
    margin-left: auto;
    flex-shrink: 0;
  }
  .spark span {
    width: 4px;
    border-radius: 2px;
    background: color-mix(in srgb, var(--c) 55%, transparent);
  }

  /* Calendar */
  .cal-wd,
  .cal-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 3px;
  }
  .cal-wd {
    margin-bottom: 4px;
  }
  .cal-wd span {
    text-align: center;
    font-size: 0.58rem;
    color: var(--faint);
    text-transform: uppercase;
  }
  .cal-cell {
    min-height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.35rem;
    font-size: 0.74rem;
    color: var(--subtext);
    position: relative;
  }
  .cal-cell.today {
    background: var(--accent);
    color: var(--base);
    font-weight: 700;
  }
  .cal-cell.empty {
    visibility: hidden;
  }
  .cal-ev {
    position: absolute;
    bottom: 3px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--accent-public);
  }

  /* Heatmap */
  .heatmap {
    display: flex;
    gap: 3px;
    flex-wrap: wrap;
  }
  .week {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .cell {
    width: 13px;
    height: 13px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.05);
  }
  .lvl1 {
    background: color-mix(in srgb, var(--accent-public) 32%, transparent);
  }
  .lvl2 {
    background: color-mix(in srgb, var(--accent-public) 58%, transparent);
  }
  .lvl3 {
    background: color-mix(in srgb, var(--accent-public) 80%, transparent);
  }
  .lvl4 {
    background: var(--accent-public);
  }
  .git-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 0.7rem;
    font-size: 0.64rem;
    color: var(--faint);
  }
  .git-foot span {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }
  .legend .cell {
    width: 10px;
    height: 10px;
  }

  /* Lists */
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    overflow-y: auto;
  }
  .list li {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    font-size: 0.8rem;
  }
  .sdot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--c);
    flex-shrink: 0;
  }
  .li-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text);
  }
  .li-detail {
    font-size: 0.66rem;
    color: var(--faint);
    flex-shrink: 0;
  }
  .news-tag {
    font-size: 0.54rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 700;
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 16%, transparent);
    padding: 0.05rem 0.4rem;
    border-radius: 0.5rem;
    flex-shrink: 0;
    min-width: 3.4rem;
    text-align: center;
  }
  .team {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    overflow-y: auto;
  }
  .av {
    position: relative;
    display: inline-flex;
  }
  .av.online::after {
    content: "";
    position: absolute;
    right: -1px;
    bottom: -1px;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--accent-public);
    border: 2px solid var(--mantle);
  }
  .weather-card {
    padding: 0.7rem;
  }

  /* Commit feed — repoz CLI listing: one full-width row per commit, with a
     per-repo path header, dotted leaders, +/− churn, and an author column. */
  .repoz {
    flex: 1;
    overflow-y: auto;
    padding: 0.55rem 0.7rem 0.45rem;
    background: var(--base);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-family: ui-monospace, "SF Mono", "JetBrains Mono", monospace;
    font-size: 0.78rem;
    line-height: 1.65;
  }
  .repo-block {
    margin-bottom: 0.7rem;
  }
  /* repo header: dot · name · path ···· N behind */
  .repo-line {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 600;
  }
  .proj-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--c);
    opacity: 0.65;
    flex-shrink: 0;
  }
  .repo-name {
    color: var(--text);
  }
  .repo-path {
    color: var(--faint);
    font-weight: 400;
  }
  .behind {
    color: var(--accent-global);
    flex-shrink: 0;
  }
  .clean {
    color: var(--faint);
    flex-shrink: 0;
  }
  /* commit row: hash  message ···· +add −del  author */
  .crow {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding-left: 0.6rem;
  }
  .hash {
    color: var(--accent-personal);
    flex-shrink: 0;
  }
  .cmsg {
    color: var(--text);
    flex: 0 1 auto;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  /* dotted leader fills the gap between content and the right-hand columns */
  .leader {
    flex: 1 1 1.5rem;
    align-self: center;
    min-width: 1rem;
    height: 0.85em;
    border-bottom: 1px dotted rgba(255, 255, 255, 0.18);
  }
  .stats {
    flex-shrink: 0;
    width: 6.5em;
    text-align: right;
    font-variant-numeric: tabular-nums;
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  .add {
    color: var(--accent-public);
  }
  .del {
    color: var(--danger);
  }
  .author {
    flex-shrink: 0;
    width: 4.5em;
    color: var(--faint);
  }
  .repoz-foot {
    padding-top: 0.45rem;
    border-top: 1px solid var(--border);
    color: var(--accent-global);
  }
</style>
