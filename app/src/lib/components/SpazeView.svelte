<script lang="ts">
  // "spaze" view — a desktop take on the spaze TUI chat (terminal-first team
  // chat, GitHub identity, inline #note / #todo capture). Rooms and members are
  // real (one room per airwavez repo, member counts from contributors) and the
  // composer posts as your real GitHub identity. Live message sync has no
  // backend yet, so messages you send stay local this session.
  import { onMount } from "svelte";
  import Avatar from "$lib/components/Avatar.svelte";
  import { githubContributors, githubUser } from "$lib/ipc";
  import { repoStore } from "$lib/repos.svelte";
  import type { GhRepo } from "$lib/types";
  import { Hash, Send, Users } from "lucide-svelte";

  /** Short repo name from a full owner/repo key. */
  const shortName = (full: string) => full.split("/").pop() ?? full;

  let { active = true }: { active?: boolean } = $props();

  interface Msg {
    author: string;
    time: string;
    text: string;
    avatar?: string | null;
  }

  // Rooms are the active repos (full owner/repo keys), shared via repoStore.
  let rooms = $derived<GhRepo[]>(repoStore.activeRepos);
  let memberCount = $state<Record<string, number>>({});
  let me = $state<{ login: string; avatar: string | null }>({ login: "you", avatar: null });
  let activeRoom = $state(""); // full owner/repo name
  let threads = $state<Record<string, Msg[]>>({});
  let loading = $state(true);
  let seededFor = new Set<string>();

  onMount(async () => {
    try {
      const u = await githubUser();
      me = { login: u.login, avatar: u.avatar_url };
    } catch {
      /* offline */
    }
    await repoStore.ensure(me.login);
    loading = false;
  });

  // Seed rooms (intro line + member counts) as the active selection changes.
  $effect(() => {
    const rs = rooms;
    if (!activeRoom && rs.length) activeRoom = rs[0].full_name;
    for (const r of rs) {
      if (seededFor.has(r.full_name)) continue;
      seededFor.add(r.full_name);
      const desc = r.description?.trim();
      threads[r.full_name] = desc
        ? [{ author: me.login, time: "", text: desc, avatar: me.avatar }]
        : (threads[r.full_name] ?? []);
      githubContributors(r.full_name)
        .then((c) => (memberCount[r.full_name] = c.length))
        .catch(() => (memberCount[r.full_name] = 0));
    }
  });

  let draft = $state("");

  function send() {
    const t = draft.trim();
    if (!t || !activeRoom) return;
    threads[activeRoom] = [
      ...(threads[activeRoom] ?? []),
      { author: me.login, time: "now", text: t, avatar: me.avatar },
    ];
    draft = "";
  }

  // Split a message into plain text + #note/#todo capture tokens for highlight.
  function segments(text: string): { t: string; tag: boolean }[] {
    return text.split(/(\s+)/).map((w) => ({
      t: w,
      tag: /^#(note|todo)\b/i.test(w),
    }));
  }
</script>

<div class="spaze">
  <aside class="rooms">
    <div class="server">
      <span class="server-name">spaze</span>
      <span class="server-host">{me.login}</span>
    </div>
    <div class="room-label">Rooms</div>
    {#each rooms as r (r.full_name)}
      <button class="room" class:active={activeRoom === r.full_name} onclick={() => (activeRoom = r.full_name)}>
        <Hash size={14} />
        <span class="room-name">{r.name}</span>
        {#if memberCount[r.full_name]}<span class="badge">{memberCount[r.full_name]}</span>{/if}
      </button>
    {/each}
    {#if rooms.length === 0}
      <div class="room-empty">{loading ? "loading…" : "no active repos — pick some on Home"}</div>
    {/if}
  </aside>

  <div class="main">
    <header class="room-head">
      <Hash size={16} />
      <span class="head-name">{activeRoom ? shortName(activeRoom) : "—"}</span>
      <div class="spacer"></div>
      <span class="members"><Users size={13} /> {memberCount[activeRoom] ?? 0}</span>
    </header>

    <div class="timeline">
      {#each threads[activeRoom] ?? [] as m, i (i)}
        <div class="msg">
          <Avatar name={m.author} src={m.avatar} size={28} />
          <div class="msg-body">
            <div class="msg-head">
              <span class="author">{m.author}</span>
              <span class="time">{m.time}</span>
            </div>
            <div class="text">
              {#each segments(m.text) as s, si (si)}<span class:capture={s.tag}>{s.t}</span>{/each}
            </div>
          </div>
        </div>
      {/each}
    </div>

    <form
      class="composer"
      onsubmit={(e) => {
        e.preventDefault();
        send();
      }}
    >
      <input
        class="input"
        placeholder={`Message #${activeRoom ? shortName(activeRoom) : ""}…  (try #note or #todo)`}
        bind:value={draft}
      />
      <button class="ghost iconbtn icononly" type="submit" title="Send" aria-label="Send">
        <Send size={15} />
      </button>
    </form>

    <div class="statusbar">
      <span class="sb-path"># {activeRoom ? shortName(activeRoom) : ""}</span>
      <div class="sb-spacer"></div>
      <span class="sb-count">
        {(threads[activeRoom] ?? []).length} messages · {memberCount[activeRoom] ?? 0} members · local
      </span>
    </div>
  </div>
</div>

<style>
  .spaze {
    display: flex;
    height: 100%;
    background: var(--base);
  }
  .rooms {
    width: 200px;
    flex-shrink: 0;
    border-right: 1px solid var(--border);
    background: rgba(20, 20, 32, var(--sidebar-glass-alpha));
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    overflow-y: auto;
  }
  .server {
    display: flex;
    flex-direction: column;
    padding: 0.4rem 0.5rem 0.6rem;
  }
  .server-name {
    font-weight: 700;
    color: var(--accent);
  }
  .server-host {
    font-size: 0.62rem;
    color: var(--faint);
  }
  .room-label {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--faint);
    padding: 0.25rem 0.5rem;
  }
  .room-empty {
    padding: 0.4rem 0.5rem;
    font-size: 0.72rem;
    color: var(--faint);
  }
  .room {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.35rem 0.5rem;
    border-radius: 0.4rem;
    border: none;
    background: none;
    color: var(--subtext);
    cursor: pointer;
    font: inherit;
    font-size: 0.82rem;
  }
  .room:hover {
    background: var(--surface);
    color: var(--text);
  }
  .room.active {
    background: var(--surface-active);
    color: var(--text);
  }
  .room-name {
    flex: 1;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .badge {
    font-size: 0.6rem;
    font-weight: 700;
    color: var(--base);
    background: var(--accent);
    border-radius: 0.6rem;
    padding: 0.02rem 0.35rem;
  }
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .room-head {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.55rem 0.85rem;
    border-bottom: 1px solid var(--border);
    color: var(--text);
    font-weight: 600;
    flex-shrink: 0;
  }
  .members {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.72rem;
    color: var(--faint);
    font-weight: 400;
  }
  .timeline {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0.85rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }
  .msg {
    display: flex;
    gap: 0.6rem;
  }
  .msg-body {
    min-width: 0;
  }
  .msg-head {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }
  .author {
    font-weight: 700;
    font-size: 0.82rem;
    color: var(--text);
  }
  .time {
    font-size: 0.66rem;
    color: var(--faint);
  }
  .text {
    font-size: 0.86rem;
    line-height: 1.5;
    color: var(--subtext);
  }
  .capture {
    color: var(--accent-public);
    font-weight: 600;
  }
  .composer {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 0.85rem;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }
  .input {
    flex: 1;
    min-width: 0;
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    color: var(--text);
    padding: 0.55rem 0.7rem;
    font: inherit;
    font-size: 0.85rem;
  }
  .input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .statusbar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.3rem 0.85rem;
    border-top: 1px solid var(--border);
    background: var(--mantle);
    flex-shrink: 0;
    font-size: 0.72rem;
  }
  .sb-path {
    color: var(--subtext);
  }
  .sb-spacer {
    flex: 1;
  }
  .sb-count {
    color: var(--faint);
  }
</style>
