<script lang="ts">
  // Mock "home" dashboard. Only the clock + the calendar month are real.
  import Avatar from "$lib/components/Avatar.svelte";
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
  let hm = $derived(now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }));
  let ss = $derived(now.toLocaleTimeString([], { second: "2-digit" }).replace(/\D/g, ""));
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

  // git-log style commit feed (repoz-in-the-dashboard).
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

  // Group commits by project, projects ordered by their most recent commit.
  type Commit = (typeof COMMITS)[number];
  const COMMIT_GROUPS: { repo: string; commits: Commit[] }[] = (() => {
    const order: string[] = [];
    const map: Record<string, Commit[]> = {};
    for (const c of COMMITS) {
      if (!map[c.repo]) {
        map[c.repo] = [];
        order.push(c.repo);
      }
      map[c.repo].push(c);
    }
    return order.map((repo) => ({ repo, commits: map[repo] }));
  })();
</script>

<div class="dash">
  <div class="inner">
    <header class="hero">
      <div class="greet">
        <div class="hello">{greeting}, you</div>
        <div class="date">{dateStr}</div>
      </div>
      <div class="clock"><span class="hm">{hm}</span><span class="ss">{ss}</span></div>
      <div class="weather">
        <CloudSun size={28} />
        <div><div class="w-temp">7°</div><div class="w-place">Stockholm</div></div>
      </div>
    </header>

    <div class="tiles">
      {#each STATS as s (s.label)}
        <section class="card tile">
          <div class="tile-ico" style="--c:{s.color}"><s.icon size={17} /></div>
          <div class="tile-text">
            <div class="tile-num">{s.value}</div>
            <div class="tile-label">{s.label} · <span class="trend">{s.trend}</span></div>
          </div>
          <div class="spark" style="--c:{s.color}">
            {#each spark(s.label) as h (h)}<span style="height:{h}%"></span>{/each}
          </div>
        </section>
      {/each}
    </div>

    <div class="feature">
      <section class="card">
        <div class="card-head"><CalendarDays size={13} /> {monthName}</div>
        <div class="cal-wd">{#each WD as w, i (i)}<span>{w}</span>{/each}</div>
        <div class="cal-grid">
          {#each calCells as c, i (i)}
            <div class="cal-cell" class:empty={c.day === null} class:today={c.day === todayNum}>
              {#if c.day}{c.day}{#if c.event}<span class="cal-ev"></span>{/if}{/if}
            </div>
          {/each}
        </div>
      </section>

      <section class="card">
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
      </section>
    </div>

    <div class="detail">
      <section class="card">
        <div class="card-head"><FolderGit2 size={13} /> Repos</div>
        <ul class="list">
          {#each REPOS as r (r.name)}
            <li>
              <span class="sdot" style="--c:{STATUS_COLOR[r.status]}"></span>
              <span class="li-name">{r.name}</span>
              <span class="li-detail">{r.detail}</span>
            </li>
          {/each}
        </ul>
      </section>

      <section class="card">
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
      </section>

      <section class="card team-card">
        <div class="card-head"><Users size={13} /> Team</div>
        <div class="team">
          {#each TEAM as m (m.name)}
            <span class="av" class:online={m.online} title={m.name}><Avatar name={m.name} size={26} /></span>
          {/each}
        </div>
      </section>
    </div>

    <section class="card feed">
      <div class="card-head"><GitCommitHorizontal size={13} /> Recent commits</div>
      <div class="commits">
        {#each COMMIT_GROUPS as g (g.repo)}
          <div class="proj-head">
            <span class="proj-dot" style="--c:{REPO_COLOR[g.repo] ?? 'var(--subtext)'}"></span>
            <span class="proj-name">{g.repo}</span>
            <span class="proj-count">{g.commits.length}</span>
          </div>
          {#each g.commits as c (c.hash)}
            <div class="commit">
              <span class="hash">{c.hash}</span>
              <span class="cmsg">{c.msg}</span>
              <Avatar name={c.by} size={14} />
              <span class="ctime">{c.time}</span>
            </div>
          {/each}
        {/each}
      </div>
    </section>
  </div>
</div>

<style>
  .dash {
    height: 100%;
    overflow-y: auto;
    background: var(--base);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .inner {
    width: 100%;
    max-width: 1320px;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }

  .hero {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding: 0.7rem 1.1rem;
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
    display: flex;
    align-items: baseline;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }
  .hm {
    font-size: 2.1rem;
    font-weight: 700;
    line-height: 1;
  }
  .ss {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--subtext);
    margin-left: 0.25rem;
  }
  .weather {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--accent-global);
    padding-left: 1rem;
    border-left: 1px solid var(--border);
  }
  .w-temp {
    font-size: 1.3rem;
    font-weight: 700;
    color: var(--text);
    line-height: 1;
  }
  .w-place {
    font-size: 0.68rem;
    color: var(--faint);
  }

  .card {
    background: var(--mantle);
    border: 1px solid var(--border);
    border-radius: 0.6rem;
    padding: 0.8rem;
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
    margin-bottom: 0.65rem;
  }

  /* Tiles */
  .tiles {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.7rem;
  }
  .tile {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    padding: 0.75rem 0.85rem;
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

  /* Feature + detail rows */
  .feature {
    display: grid;
    grid-template-columns: 1.15fr 1fr;
    gap: 0.7rem;
    align-items: stretch;
  }
  .detail {
    display: grid;
    grid-template-columns: 1fr 1.3fr 1fr;
    gap: 0.7rem;
    align-items: stretch;
  }
  .feed {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  @media (max-width: 900px) {
    .feature,
    .detail,
    .tiles {
      grid-template-columns: 1fr 1fr;
    }
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
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.35rem;
    font-size: 0.74rem;
    color: var(--subtext);
    position: relative;
  }
  .cal-cell:not(.empty):hover {
    background: var(--surface);
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
  .cal-cell.today .cal-ev {
    background: var(--base);
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

  /* Recent feed (fills remaining height) */
  .commits {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }
  .proj-head {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-top: 0.55rem;
    padding: 0.1rem 0.25rem;
    font-size: 0.74rem;
    font-weight: 700;
  }
  .proj-head:first-child {
    margin-top: 0;
  }
  .proj-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--c);
    opacity: 0.65;
    flex-shrink: 0;
  }
  .proj-name {
    color: var(--text);
  }
  .proj-count {
    font-size: 0.58rem;
    font-weight: 600;
    color: var(--faint);
    background: var(--glass-hover);
    padding: 0.02rem 0.35rem;
    border-radius: 0.5rem;
  }
  .commit {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.26rem 0.25rem 0.26rem 1.4rem;
    font-size: 0.8rem;
  }
  .hash {
    font-family: ui-monospace, "SF Mono", "JetBrains Mono", monospace;
    font-size: 0.72rem;
    color: var(--faint);
    flex-shrink: 0;
    width: 4.3rem;
  }
  .cmsg {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--subtext);
  }
  .ctime {
    font-size: 0.66rem;
    color: var(--faint);
    width: 2.4rem;
    text-align: right;
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }
</style>
