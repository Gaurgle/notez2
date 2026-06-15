<script lang="ts">
  // Mock "home" dashboard on a draggable/resizable Gridstack board.
  // The clock, the calendar month, and the weather (Open-Meteo) are real.
  import { onMount, onDestroy } from "svelte";
  import { GridStack } from "gridstack";
  import "gridstack/dist/gridstack.min.css";
  import Avatar from "$lib/components/Avatar.svelte";
  import MachineAvatar from "$lib/components/MachineAvatar.svelte";
  import WeatherWidget from "$lib/components/WeatherWidget.svelte";
  import { hashStr, relativeTime } from "$lib/mock";
  import {
    githubUser,
    githubCommits,
    githubIssues,
    githubContributors,
    listNotes,
    loadTodoBoard,
  } from "$lib/ipc";
  import { repoStore } from "$lib/repos.svelte";
  import type { GhCommit, GhIssue, GhContributor, GhUser } from "$lib/types";
  import {
    CloudSun,
    GitCommitHorizontal,
    FolderGit2,
    Users,
    CircleDot,
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

  // --- real data: the active repos (across personal + orgs) via the gh CLI --
  // The repo list + selection live in the shared repoStore so every view agrees
  // on which repos are active. The heavy per-repo fetches below re-run whenever
  // that selection changes.
  let user = $state<GhUser | null>(null);
  let commits = $state<GhCommit[]>([]);
  let issues = $state<GhIssue[]>([]);
  let contributors = $state<GhContributor[]>([]);
  let noteCount = $state(0);
  let todoOpen = $state(0);
  let todoDone = $state(0);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let fetchToken = 0;

  onMount(async () => {
    try {
      user = await githubUser();
    } catch {
      /* offline: identity stays "you" */
    }
    await repoStore.ensure(user?.login);
    // Local notez stats, independent of GitHub (best-effort).
    try {
      noteCount = (await listNotes()).length;
    } catch {
      /* ignore */
    }
    try {
      const board = await loadTodoBoard();
      const tasks = board.items.filter((t) => !t.is_header);
      todoOpen = tasks.filter((t) => t.state !== "checked").length;
      todoDone = tasks.filter((t) => t.state === "checked").length;
    } catch {
      /* ignore */
    }
  });

  // Re-fetch commits/issues/contributors whenever the active selection changes.
  $effect(() => {
    const names = repoStore.activeNames;
    if (repoStore.loading) return;
    const token = ++fetchToken;
    loadActive(names, token);
  });

  async function loadActive(names: string[], token: number) {
    loading = true;
    loadError = repoStore.error;
    if (names.length === 0) {
      commits = [];
      issues = [];
      contributors = [];
      loading = false;
      return;
    }
    try {
      const [c, iss, ...contribLists] = await Promise.all([
        githubCommits(names, 100),
        githubIssues(names),
        ...names.map((n) => githubContributors(n)),
      ]);
      if (token !== fetchToken) return; // a newer selection superseded this one
      commits = c;
      issues = iss;
      const cmap = new Map<string, GhContributor>();
      for (const list of contribLists as GhContributor[][]) {
        for (const person of list) {
          const existing = cmap.get(person.login);
          if (existing) existing.contributions += person.contributions;
          else cmap.set(person.login, { ...person });
        }
      }
      contributors = [...cmap.values()].sort((a, b) => b.contributions - a.contributions);
    } catch (e) {
      if (token === fetchToken) loadError = String(e);
    } finally {
      if (token === fetchToken) loading = false;
    }
  }

  const firstName = $derived(user ? (user.name || user.login).split(" ")[0] : "you");
  /** Short repo name from a full owner/repo key. */
  const shortName = (full: string) => full.split("/").pop() ?? full;

  // Sidebar repo filter (the user has many repos across owners).
  let repoFilter = $state("");
  const matchesFilter = (r: { name: string; full_name: string }) =>
    !repoFilter ||
    r.name.toLowerCase().includes(repoFilter.toLowerCase()) ||
    r.full_name.toLowerCase().includes(repoFilter.toLowerCase());
  /** Relative time from an ISO-8601 timestamp. */
  const isoRel = (iso: string) => (iso ? relativeTime(Date.parse(iso) / 1000) : "");

  function spark(seed: string): number[] {
    return Array.from({ length: 9 }, (_, i) => 22 + (hashStr(seed + ":" + i) % 78));
  }
  let stats = $derived([
    { label: "Notes", value: noteCount, trend: "all scopes", icon: FileText, color: "var(--accent-local)" },
    { label: "Todos", value: todoOpen, trend: `${todoDone} done`, icon: ListChecks, color: "var(--accent-public)" },
    {
      label: "Issues",
      value: issues.filter((i) => i.state === "open").length,
      trend: `${issues.length} total`,
      icon: KanbanSquare,
      color: "var(--accent-personal)",
    },
    { label: "Commits", value: commits.length, trend: `${repoStore.activeNames.length} active repos`, icon: GitCommitHorizontal, color: "var(--accent-global)" },
  ]);

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

  const WEEKS = 16;
  function isoDay(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  }
  // Real git-activity heatmap: WEEKS columns (oldest left), each Mon..Sun,
  // bucketed from real commit dates into 5 intensity levels.
  let grid = $state<number[][]>([]);
  let streak = $state(0);
  $effect(() => {
    const counts = new Map<string, number>();
    for (const c of commits) {
      const day = c.date.slice(0, 10);
      if (day) counts.set(day, (counts.get(day) ?? 0) + 1);
    }
    const today = new Date();
    const dow = (today.getDay() + 6) % 7; // Mon = 0
    const monday = new Date(today);
    monday.setDate(today.getDate() - dow);
    const out: number[][] = [];
    for (let w = WEEKS - 1; w >= 0; w--) {
      const col: number[] = [];
      for (let d = 0; d < 7; d++) {
        const cell = new Date(monday);
        cell.setDate(monday.getDate() - w * 7 + d);
        const n = cell > today ? -1 : counts.get(isoDay(cell)) ?? 0;
        col.push(n < 0 ? 0 : n === 0 ? 0 : n >= 6 ? 4 : n >= 4 ? 3 : n >= 2 ? 2 : 1);
      }
      out.push(col);
    }
    grid = out;
    // Current streak: consecutive days with >=1 commit ending today (or
    // yesterday, so an idle morning doesn't zero a live streak).
    let n = 0;
    const probe = new Date();
    if (!counts.has(isoDay(probe))) probe.setDate(probe.getDate() - 1);
    while (counts.has(isoDay(probe))) {
      n++;
      probe.setDate(probe.getDate() - 1);
    }
    streak = n;
  });

  // Accent color per repo, stable across renders.
  const REPO_ACCENTS = [
    "var(--accent-personal)",
    "var(--accent-local)",
    "var(--accent-public)",
    "var(--accent-global)",
  ];
  const repoColor = (name: string) => REPO_ACCENTS[hashStr(name) % REPO_ACCENTS.length];

  // Active repos drive every widget. The fetch above already scoped commits and
  // issues to the selection, so these just shape that data for display.
  let visibleRepos = $derived(repoStore.activeRepos);
  let openIssues = $derived(issues.filter((i) => i.state === "open"));
  // Real commits grouped by repo (newest first), capped per repo for the feed.
  let visibleCommitGroups = $derived.by(() => {
    const order: string[] = [];
    const map = new Map<string, GhCommit[]>();
    for (const c of commits) {
      if (!map.has(c.repo)) {
        map.set(c.repo, []);
        order.push(c.repo);
      }
      const arr = map.get(c.repo)!;
      if (arr.length < 6) arr.push(c);
    }
    return order.map((repo) => ({
      repo,
      name: shortName(repo),
      open: issues.filter((i) => i.repo === repo && i.state === "open").length,
      commits: map.get(repo) ?? [],
    }));
  });
  let totalCommits = $derived(visibleCommitGroups.reduce((n, g) => n + g.commits.length, 0));

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
    <nav class="group repo-picker">
      <div class="group-label">Active repos · {repoStore.activeNames.length}</div>
      <input class="repo-filter" placeholder="filter…" bind:value={repoFilter} />
      {#each repoStore.groups as g (g.owner)}
        {@const frepos = g.repos.filter(matchesFilter)}
        {#if frepos.length}
          {@const sel = g.repos.filter((r) => repoStore.selected.has(r.full_name)).length}
          <div class="owner-row">
            <span class="owner-name">{g.isMe ? "personal" : g.owner}</span>
            <button
              class="owner-toggle"
              title="Toggle all in {g.owner}"
              onclick={() => repoStore.setGroup(g.repos, sel < g.repos.length)}
            >
              {sel}/{g.repos.length}
            </button>
          </div>
          {#each frepos as r (r.full_name)}
            <button class="item" class:active={repoStore.selected.has(r.full_name)} onclick={() => repoStore.toggle(r.full_name)}>
              <span class="cbox" class:on={repoStore.selected.has(r.full_name)}></span>
              <span class="item-label">{r.name}</span>
              {#if r.is_private}<span class="lock" title="private">·</span>{/if}
            </button>
          {/each}
        {/if}
      {/each}
      {#if repoStore.repos.length === 0}
        <div class="side-empty">{repoStore.loading ? "loading…" : "no repos"}</div>
      {/if}
      {#if repoStore.archivedCount > 0 || repoStore.showArchived}
        <button class="archive-toggle" onclick={() => (repoStore.showArchived = !repoStore.showArchived)}>
          {repoStore.showArchived ? "hide dormant repos" : `show ${repoStore.archivedCount} dormant (6mo+)`}
        </button>
      {/if}
    </nav>
  </aside>

  <div class="main">
    <div class="inner">
      <header class="hero">
      <div class="greet">
        <div class="hello">{greeting}, {firstName}</div>
        <div class="date">{dateStr}{#if loadError} · <span class="err">offline</span>{/if}</div>
      </div>
      <div class="clock">{clock}</div>
      <button class="reset" onclick={resetLayout} title="Reset widget layout" aria-label="Reset layout">
        <RotateCcw size={14} />
      </button>
    </header>

    <div class="grid-stack" bind:this={gridEl}>
      {#each stats as s, i (s.label)}
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
            <span><Flame size={12} /> {streak}-day streak</span>
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
                <span class="sdot" style="--c:{repoColor(r.name)}"></span>
                <span class="li-name">{r.name}</span>
                <span class="li-detail">{r.language ?? "docs"} · {isoRel(r.pushed_at)}</span>
              </li>
            {/each}
            {#if visibleRepos.length === 0}
              <li class="empty-row">{loading ? "loading…" : "no repos selected"}</li>
            {/if}
          </ul>
        </div>
      </div>

      <div class="grid-stack-item" {...gs({ id: "news", x: 4, y: 4, w: 4, h: 3, minW: 2, minH: 2 })}>
        <div class="grid-stack-item-content card">
          <div class="card-head"><CircleDot size={13} /> Open issues</div>
          <ul class="list">
            {#each openIssues.slice(0, 7) as it (it.repo + it.number)}
              <li class="news">
                <span class="news-tag">{shortName(it.repo)}</span>
                <span class="li-name">{it.title}</span>
                <span class="li-detail">#{it.number}</span>
              </li>
            {/each}
            {#if openIssues.length === 0}
              <li class="empty-row">{loading ? "loading…" : "no open issues"}</li>
            {/if}
          </ul>
        </div>
      </div>

      <div class="grid-stack-item" {...gs({ id: "team", x: 8, y: 4, w: 4, h: 3, minW: 2, minH: 2 })}>
        <div class="grid-stack-item-content card">
          <div class="card-head"><Users size={13} /> Contributors</div>
          <div class="team">
            {#each contributors as m (m.login)}
              <span class="av" title={`${m.login} · ${m.contributions} commits`}>
                <Avatar name={m.login} src={m.avatar_url} size={26} />
              </span>
            {/each}
            {#if contributors.length === 0}
              <span class="empty-row">{loading ? "loading…" : "no contributors"}</span>
            {/if}
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
                  <span class="proj-dot" style="--c:{repoColor(g.repo)}"></span>
                  <span class="repo-name">{g.name}</span>
                  <span class="repo-path">{g.repo}</span>
                  <span class="leader"></span>
                  {#if g.open > 0}
                    <span class="behind">{g.open} open</span>
                  {:else}
                    <span class="clean">no issues</span>
                  {/if}
                </div>
                {#each g.commits as c (c.sha)}
                  <div class="crow">
                    <span class="hash">{c.sha.slice(0, 7)}</span>
                    <span class="cmsg">{c.message}</span>
                    <span class="leader"></span>
                    <span class="ctime">{isoRel(c.date)}</span>
                    <span class="cauthor">
                      <Avatar name={c.author_login ?? c.author} src={c.avatar_url} size={16} />
                    </span>
                  </div>
                {/each}
              </div>
            {/each}
            {#if visibleCommitGroups.length === 0}
              <div class="empty-row">{loading ? "loading commits…" : loadError ? "could not reach GitHub" : "no commits"}</div>
            {/if}
            <div class="repoz-foot">{totalCommits} commits shown</div>
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
  .ctime {
    flex-shrink: 0;
    text-align: right;
    color: var(--faint);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .cauthor {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
  }
  .repoz-foot {
    padding-top: 0.45rem;
    border-top: 1px solid var(--border);
    color: var(--accent-global);
  }
  .empty-row {
    color: var(--faint);
    font-size: 0.78rem;
    padding: 0.4rem 0;
  }
  .side-empty {
    color: var(--faint);
    font-size: 0.72rem;
    padding: 0.25rem 0.5rem;
  }
  .repo-filter {
    width: 100%;
    box-sizing: border-box;
    margin: 0 0 0.35rem;
    padding: 0.3rem 0.45rem;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: 0.4rem;
    color: var(--text);
    font: inherit;
    font-size: 0.74rem;
  }
  .repo-filter:focus {
    outline: none;
    border-color: var(--accent);
  }
  .owner-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.45rem 0.5rem 0.2rem;
  }
  .owner-name {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--subtext);
    font-weight: 700;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .owner-toggle {
    font-size: 0.6rem;
    font-variant-numeric: tabular-nums;
    color: var(--faint);
    background: none;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 0.02rem 0.35rem;
    cursor: pointer;
  }
  .owner-toggle:hover {
    color: var(--accent);
    border-color: var(--accent);
  }
  .lock {
    color: var(--faint);
    margin-left: auto;
  }
  .archive-toggle {
    margin-top: 0.4rem;
    width: 100%;
    padding: 0.3rem 0.45rem;
    background: none;
    border: 1px dashed var(--border);
    border-radius: 0.4rem;
    color: var(--faint);
    cursor: pointer;
    font: inherit;
    font-size: 0.68rem;
  }
  .archive-toggle:hover {
    color: var(--accent);
    border-color: var(--accent);
  }
  .err {
    color: var(--accent-global);
  }
</style>
