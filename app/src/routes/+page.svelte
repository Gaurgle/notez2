<script lang="ts">
  import NotesView from "$lib/components/NotesView.svelte";
  import TodozView from "$lib/components/TodozView.svelte";

  let mode = $state<"notes" | "todoz">("notes");

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
  <nav class="rail">
    <button
      class="rail-btn"
      class:active={mode === "notes"}
      onclick={() => (mode = "notes")}
      title="Notes"
    >
      <span class="glyph">✎</span>
      <span class="label">Notes</span>
    </button>
    <button
      class="rail-btn"
      class:active={mode === "todoz"}
      onclick={() => (mode = "todoz")}
      title="Todos"
    >
      <span class="glyph">☑</span>
      <span class="label">Todos</span>
    </button>
  </nav>

  <main class="content">
    {#if mode === "notes"}
      <NotesView />
    {:else}
      <TodozView />
    {/if}
  </main>
</div>

<style>
  .shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }
  .rail {
    width: 64px;
    background: var(--glass);
    -webkit-backdrop-filter: var(--blur);
    backdrop-filter: var(--blur);
    border-right: 1px solid var(--border);
    box-shadow: inset 0 1px 0 var(--highlight);
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.6rem 0.4rem;
    flex-shrink: 0;
  }
  .rail-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.15rem;
    padding: 0.5rem 0;
    background: none;
    border: none;
    border-radius: 0.5rem;
    color: var(--subtext);
    cursor: pointer;
    font: inherit;
  }
  .rail-btn:hover {
    background: var(--surface);
    color: var(--text);
  }
  .rail-btn.active {
    background: var(--glass-active);
    color: var(--accent);
    box-shadow: inset 0 1px 0 var(--highlight), 0 0 18px rgba(203, 166, 247, 0.25);
  }
  .glyph {
    font-size: 1.1rem;
  }
  .label {
    font-size: 0.6rem;
  }
  .content {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }
</style>
