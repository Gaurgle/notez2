<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import NoteList from "$lib/components/NoteList.svelte";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import NewNoteDialog from "$lib/components/NewNoteDialog.svelte";
  import LogPanel from "$lib/components/LogPanel.svelte";
  import { listNotes, readNote, createNote, saveNote, appendLog } from "$lib/ipc";
  import type { NoteListItem, Scope } from "$lib/types";

  let notes = $state<NoteListItem[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let activeScope = $state<Scope | "all">("all");
  let activeProject = $state<string | null>(null);

  let selectedPath = $state<string | null>(null);
  let content = $state("");

  let showNewNote = $state(false);
  let showLog = $state(false);
  let toast = $state<string | null>(null);

  let filtered = $derived(
    notes.filter(
      (n) =>
        (activeScope === "all" || n.scope === activeScope) &&
        (activeProject === null || n.project === activeProject)
    )
  );

  async function refresh(selectPath?: string) {
    try {
      notes = await listNotes();
      error = null;
      if (selectPath) {
        const hit = notes.find((n) => n.path === selectPath);
        if (hit) await select(hit);
      }
    } catch (e) {
      error = String(e);
    }
  }

  onMount(async () => {
    await refresh();
    loading = false;
  });

  async function select(note: NoteListItem) {
    selectedPath = note.path;
    try {
      content = await readNote(note.path);
    } catch (e) {
      content = `Failed to read note:\n${e}`;
    }
  }

  async function onSave(newContent: string) {
    if (!selectedPath) return;
    try {
      await saveNote(selectedPath, newContent);
    } catch (e) {
      flash(`Save failed: ${e}`);
    }
  }

  async function onCreate(scope: Scope, title: string, body: string | null) {
    showNewNote = false;
    try {
      const path = await createNote(scope, title, body);
      await refresh(path);
      flash("Note created");
    } catch (e) {
      flash(`Create failed: ${e}`);
    }
  }

  async function onLog(scope: Scope, message: string) {
    showLog = false;
    try {
      const path = await appendLog(scope, message);
      await refresh(path);
      flash("Logged");
    } catch (e) {
      flash(`Log failed: ${e}`);
    }
  }

  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  function flash(msg: string) {
    toast = msg;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), 2500);
  }
</script>

<div class="notes">
  <Sidebar
    {notes}
    {activeScope}
    {activeProject}
    onScope={(s) => (activeScope = s)}
    onProject={(p) => (activeProject = p)}
  />

  <div class="main">
    <div class="toolbar">
      <button class="primary" onclick={() => (showNewNote = true)}>+ New</button>
      <button class="ghost" onclick={() => (showLog = true)}>Log</button>
      <div class="spacer"></div>
      {#if toast}<span class="toast">{toast}</span>{/if}
    </div>

    <div class="panes">
      <div class="list-col">
        {#if loading}
          <div class="status">Loading notes…</div>
        {:else if error}
          <div class="status error">{error}</div>
        {:else}
          <NoteList notes={filtered} {selectedPath} onSelect={select} />
        {/if}
      </div>

      <NoteEditor path={selectedPath} {content} {onSave} />
    </div>
  </div>
</div>

{#if showNewNote}
  <NewNoteDialog {onCreate} onClose={() => (showNewNote = false)} />
{/if}
{#if showLog}
  <LogPanel {onLog} onClose={() => (showLog = false)} />
{/if}

<style>
  .notes {
    display: grid;
    grid-template-columns: 200px 1fr;
    height: 100%;
    overflow: hidden;
  }
  .main {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--surface);
    background: var(--mantle);
  }
  .toolbar .spacer {
    flex: 1;
  }
  .toast {
    font-size: 0.72rem;
    color: var(--subtext);
  }
  .panes {
    display: grid;
    grid-template-columns: 280px 1fr;
    flex: 1;
    min-height: 0;
  }
  .list-col {
    min-height: 0;
    overflow: hidden;
  }
  .status {
    padding: 1rem;
    color: var(--subtext);
    font-size: 0.85rem;
  }
  .status.error {
    color: #f38ba8;
    white-space: pre-wrap;
  }
</style>
