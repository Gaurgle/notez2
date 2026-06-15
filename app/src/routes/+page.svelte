<script lang="ts">
  import DashboardView from "$lib/components/DashboardView.svelte";
  import NotesView from "$lib/components/NotesView.svelte";
  import TodozView from "$lib/components/TodozView.svelte";
  import TicketzView from "$lib/components/TicketzView.svelte";
  import SpazeView from "$lib/components/SpazeView.svelte";
  import {
    LayoutDashboard,
    FileText,
    ListChecks,
    KanbanSquare,
    MessagesSquare,
  } from "lucide-svelte";

  type Mode = "home" | "notes" | "todoz" | "ticketz" | "spaze";
  const MODES: Mode[] = ["home", "notes", "todoz", "ticketz", "spaze"];
  let mode = $state<Mode>("home");

  function isTyping(target: EventTarget | null): boolean {
    const el = target as HTMLElement | null;
    return (
      el instanceof HTMLInputElement ||
      el instanceof HTMLTextAreaElement ||
      (el?.isContentEditable ?? false)
    );
  }

  $effect(() => {
    function onKey(e: KeyboardEvent) {
      if (isTyping(e.target)) return;
      if (e.key === "Tab" && e.ctrlKey) {
        e.preventDefault();
        const dir = e.shiftKey ? -1 : 1;
        mode = MODES[(MODES.indexOf(mode) + dir + MODES.length) % MODES.length];
        return;
      }
      // Ctrl+1..6 jumps straight to a view (6 reserved for a future view).
      if (e.ctrlKey && !e.shiftKey && !e.altKey && /^[1-6]$/.test(e.key)) {
        const idx = Number(e.key) - 1;
        if (idx < MODES.length) {
          e.preventDefault();
          mode = MODES[idx];
        }
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });
</script>

<div class="shell">
  <div class="topbar">
    <div class="tabs">
      <button class="tab" class:active={mode === "home"} onclick={() => (mode = "home")} title="Home (Ctrl+1)">
        <LayoutDashboard size={15} /><span class="label">Home</span>
      </button>
      <button class="tab" class:active={mode === "notes"} onclick={() => (mode = "notes")} title="Notes (Ctrl+2)">
        <FileText size={15} /><span class="label">Notes</span>
      </button>
      <button class="tab" class:active={mode === "todoz"} onclick={() => (mode = "todoz")} title="Todos (Ctrl+3)">
        <ListChecks size={15} /><span class="label">Todos</span>
      </button>
      <button class="tab" class:active={mode === "ticketz"} onclick={() => (mode = "ticketz")} title="Tickets (Ctrl+4)">
        <KanbanSquare size={15} /><span class="label">Tickets</span>
      </button>
      <button class="tab" class:active={mode === "spaze"} onclick={() => (mode = "spaze")} title="Spaze (Ctrl+5)">
        <MessagesSquare size={15} /><span class="label">Spaze</span>
      </button>
    </div>
  </div>

  <main class="content">
    <!-- Views stay mounted so switching tabs preserves state. -->
    <div class="view" class:hidden={mode !== "home"}>
      <DashboardView active={mode === "home"} />
    </div>
    <div class="view" class:hidden={mode !== "notes"}>
      <NotesView active={mode === "notes"} />
    </div>
    <div class="view" class:hidden={mode !== "todoz"}>
      <TodozView active={mode === "todoz"} />
    </div>
    <div class="view" class:hidden={mode !== "ticketz"}>
      <TicketzView active={mode === "ticketz"} />
    </div>
    <div class="view" class:hidden={mode !== "spaze"}>
      <SpazeView active={mode === "spaze"} />
    </div>
  </main>
</div>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }
  .topbar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.3rem 0.55rem;
    background: rgba(20, 20, 32, var(--sidebar-glass-alpha));
    -webkit-backdrop-filter: var(--blur);
    backdrop-filter: var(--blur);
    border-bottom: 1px solid var(--border);
    box-shadow: inset 0 1px 0 var(--highlight);
    flex-shrink: 0;
  }
  .tabs {
    display: flex;
    gap: 0.25rem;
  }
  .tab {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.28rem 0.7rem;
    border-radius: 0.5rem;
    background: none;
    border: 1px solid transparent;
    color: var(--subtext);
    cursor: pointer;
    font: inherit;
    font-size: 0.8rem;
    font-weight: 500;
  }
  .tab:hover {
    background: var(--surface);
    color: var(--text);
  }
  .tab.active {
    background: var(--glass-active);
    color: var(--accent);
    box-shadow: inset 0 1px 0 var(--highlight), 0 0 14px rgba(203, 166, 247, 0.18);
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .view {
    height: 100%;
  }
  .view.hidden {
    display: none;
  }
</style>
