<script lang="ts">
  // Mock "spaze" view — a desktop take on the spaze TUI chat (terminal-first
  // team chat, GitHub identity, inline #note / #todo capture). Data is mock.
  import Avatar from "$lib/components/Avatar.svelte";
  import { Hash, Send, Users } from "lucide-svelte";

  let { active = true }: { active?: boolean } = $props();

  interface Msg {
    author: string;
    time: string;
    text: string;
  }

  const ROOMS = [
    { key: "general", name: "general", unread: 0 },
    { key: "notez2", name: "notez2", unread: 3 },
    { key: "spaze", name: "spaze", unread: 0 },
    { key: "repoz", name: "repoz", unread: 1 },
    { key: "random", name: "random", unread: 0 },
  ];
  const MEMBERS: Record<string, number> = { general: 6, notez2: 4, spaze: 3, repoz: 2, random: 6 };

  let activeRoom = $state("notez2");

  let threads = $state<Record<string, Msg[]>>({
    general: [
      { author: "nora", time: "08:30", text: "morning all ☕" },
      { author: "kai", time: "08:34", text: "self-hosting spaze in 10 min still the goal? #note deploy guide" },
    ],
    notez2: [
      { author: "alex", time: "09:41", text: "pushed the dedupe fix — collect_all was double-walking personal/" },
      { author: "you", time: "09:43", text: "huge. that crash was brutal. #todo verify on the laptop too" },
      { author: "mira", time: "09:50", text: "inspector now shows contributors + cross-project counts, looks clean" },
      { author: "alex", time: "10:02", text: "next: @date encoding for calendar so it round-trips through the CLI" },
    ],
    spaze: [
      { author: "you", time: "11:12", text: "Phase 1.D shipped — sidebar nav with hjkl + Enter" },
      { author: "sam", time: "11:15", text: "GitHub device-flow auth is the unlock for identity everywhere" },
    ],
    repoz: [
      { author: "sam", time: "14:20", text: "repoz now scans all repos for dirty trees + unpushed commits" },
    ],
    random: [{ author: "nora", time: "16:00", text: "ratatui vs svelte, fight 🥊" }],
  });

  let draft = $state("");

  function send() {
    const t = draft.trim();
    if (!t) return;
    threads[activeRoom] = [
      ...(threads[activeRoom] ?? []),
      { author: "you", time: "now", text: t },
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
      <span class="server-host">self-hosted</span>
    </div>
    <div class="room-label">Rooms</div>
    {#each ROOMS as r (r.key)}
      <button class="room" class:active={activeRoom === r.key} onclick={() => (activeRoom = r.key)}>
        <Hash size={14} />
        <span class="room-name">{r.name}</span>
        {#if r.unread > 0}<span class="badge">{r.unread}</span>{/if}
      </button>
    {/each}
  </aside>

  <div class="main">
    <header class="room-head">
      <Hash size={16} />
      <span class="head-name">{activeRoom}</span>
      <div class="spacer"></div>
      <span class="members"><Users size={13} /> {MEMBERS[activeRoom] ?? 0}</span>
    </header>

    <div class="timeline">
      {#each threads[activeRoom] ?? [] as m, i (i)}
        <div class="msg">
          <Avatar name={m.author} size={28} />
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
        placeholder={`Message #${activeRoom}…  (try #note or #todo)`}
        bind:value={draft}
      />
      <button class="ghost iconbtn icononly" type="submit" title="Send" aria-label="Send">
        <Send size={15} />
      </button>
    </form>

    <div class="statusbar">
      <span class="sb-path"># {activeRoom}</span>
      <div class="sb-spacer"></div>
      <span class="sb-count">
        {(threads[activeRoom] ?? []).length} messages · {MEMBERS[activeRoom] ?? 0} members · mock
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
