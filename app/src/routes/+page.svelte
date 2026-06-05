<script lang="ts">
  import NotesView from "$lib/components/NotesView.svelte";
  import TodozView from "$lib/components/TodozView.svelte";
  import { Tabs } from "melt/builders";

  let mode = $state<"notes" | "todoz">("notes");

  const tabs = new Tabs<"notes" | "todoz">({
    value: () => mode,
    onValueChange: (v) => (mode = v),
  });

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
      if (e.key === "Tab" && e.ctrlKey && !isTyping(e.target)) {
        e.preventDefault();
        mode = mode === "notes" ? "todoz" : "notes";
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });
</script>

<div class="shell">
  <div class="topbar">
    <div class="tabs" {...tabs.triggerList}>
      <button
        class="tab"
        class:active={mode === "notes"}
        {...tabs.getTrigger("notes")}
        title="Notes"
      >
        <span class="glyph">✎</span><span class="label">Notes</span>
      </button>
      <button
        class="tab"
        class:active={mode === "todoz"}
        {...tabs.getTrigger("todoz")}
        title="Todos"
      >
        <span class="glyph">☑</span><span class="label">Todos</span>
      </button>
    </div>
  </div>

  <main class="content">
    <!-- Both views stay mounted so switching tabs preserves selection/scroll. -->
    <div class="view" class:hidden={mode !== "notes"} {...tabs.getContent("notes")}>
      <NotesView active={mode === "notes"} />
    </div>
    <div class="view" class:hidden={mode !== "todoz"} {...tabs.getContent("todoz")}>
      <TodozView active={mode === "todoz"} />
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
  .glyph {
    font-size: 0.95rem;
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
