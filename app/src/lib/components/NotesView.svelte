<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import NoteList from "$lib/components/NoteList.svelte";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import NewNoteDialog from "$lib/components/NewNoteDialog.svelte";
  import LogPanel from "$lib/components/LogPanel.svelte";
  import AttachProjectDialog from "$lib/components/AttachProjectDialog.svelte";
  import MigrationDialog from "$lib/components/MigrationDialog.svelte";
  import Inspector from "$lib/components/Inspector.svelte";
  import { SCOPE_META } from "$lib/types";
  import {
    listNotes,
    readNote,
    createNote,
    saveNote,
    appendLog,
    setNoteTags,
    listProjects,
    attachProject,
    detachProject,
    syncNotez,
  } from "$lib/ipc";
  import type { NoteListItem, ProjectInfo, Scope } from "$lib/types";

  let { active = true }: { active?: boolean } = $props();

  let notes = $state<NoteListItem[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let activeScope = $state<Scope | "all">("all");
  let activeProject = $state<string | null>(null);
  let searchText = $state("");

  let selectedPath = $state<string | null>(null);
  let selIndex = $state(0);
  let content = $state("");
  let editorMode = $state<"read" | "edit">("read");

  let hoveredNote = $state<NoteListItem | null>(null);
  let previewContent = $state("");
  let hoverTimer: ReturnType<typeof setTimeout> | undefined;

  let previewing = $derived(hoveredNote !== null && hoveredNote.path !== selectedPath);
  let inspected = $derived(hoveredNote ?? notes.find((n) => n.path === selectedPath) ?? null);
  let editorPath = $derived(previewing && hoveredNote ? hoveredNote.path : selectedPath);
  let editorContent = $derived(previewing ? previewContent : content);

  function onHover(note: NoteListItem | null) {
    // Sticky: keep the last preview when the cursor leaves a row, so you can
    // actually move over to read it. It's cleared on select / Esc.
    if (!note) return;
    hoveredNote = note;
    clearTimeout(hoverTimer);
    if (note.path !== selectedPath) {
      hoverTimer = setTimeout(async () => {
        try {
          previewContent = await readNote(note.path);
        } catch {
          previewContent = "";
        }
      }, 80);
    }
  }

  let showNewNote = $state(false);
  let showLog = $state(false);
  let showAttach = $state(false);
  let showMigrate = $state(false);
  let projects = $state<ProjectInfo[]>([]);
  let syncing = $state(false);
  let toast = $state<string | null>(null);

  let filtered = $derived(
    notes.filter(
      (n) =>
        (activeScope === "all" || n.scope === activeScope) &&
        (activeProject === null || n.project === activeProject) &&
        (searchText.trim() === "" ||
          n.name.toLowerCase().includes(searchText.toLowerCase()))
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
    } finally {
      loading = false; // always reveal the list once notes have loaded
    }
  }

  onMount(() => {
    // Hard safety net: never let the list sit on "Loading…" — reveal it within
    // 1.5s no matter what happens with the IPC calls.
    const fallback = setTimeout(() => (loading = false), 1500);
    refresh().finally(() => {
      clearTimeout(fallback);
      loading = false;
    });
    loadProjects(); // parallel, non-blocking
  });

  async function loadProjects() {
    try {
      projects = await listProjects();
    } catch {
      /* registry may be empty */
    }
  }

  async function onAttach(name: string, path: string) {
    showAttach = false;
    try {
      await attachProject(name, path);
      await loadProjects();
      flash("Project attached");
    } catch (e) {
      flash(`Attach failed: ${e}`);
    }
  }

  async function onDetach(name: string) {
    try {
      await detachProject(name);
      await loadProjects();
      flash("Detached");
    } catch (e) {
      flash(`Detach failed: ${e}`);
    }
  }

  async function doSync() {
    syncing = true;
    try {
      const out = await syncNotez();
      flash(out ? (out.split("\n").pop() ?? "Synced") : "Synced");
      await refresh();
    } catch (e) {
      flash(`Sync failed: ${String(e).split("\n")[0]}`);
    } finally {
      syncing = false;
    }
  }

  async function select(note: NoteListItem) {
    selectedPath = note.path;
    hoveredNote = null; // commit: drop the preview
    selIndex = filtered.findIndex((n) => n.path === note.path);
    try {
      content = await readNote(note.path);
    } catch (e) {
      content = `Failed to read note:\n${e}`;
    }
  }

  function isTyping(target: EventTarget | null): boolean {
    const el = target as HTMLElement | null;
    return (
      el instanceof HTMLInputElement ||
      el instanceof HTMLTextAreaElement ||
      (el?.isContentEditable ?? false)
    );
  }

  function selectAt() {
    const n = filtered[selIndex];
    if (n) select(n);
  }

  $effect(() => {
    if (selIndex >= filtered.length) selIndex = Math.max(0, filtered.length - 1);
  });

  $effect(() => {
    function onKey(e: KeyboardEvent) {
      if (!active) return; // only the visible view handles keys
      if (isTyping(e.target)) {
        if (e.key === "Escape") (e.target as HTMLElement).blur();
        return;
      }
      if (e.key === "Escape") {
        if (showNewNote || showLog || showAttach || showMigrate) {
          showNewNote = false;
          showLog = false;
          showAttach = false;
          showMigrate = false;
        } else {
          hoveredNote = null;
        }
        return;
      }
      if (e.key === "j" || e.key === "ArrowDown") {
        selIndex = Math.min(selIndex + 1, filtered.length - 1);
        selectAt();
        e.preventDefault();
      } else if (e.key === "k" || e.key === "ArrowUp") {
        selIndex = Math.max(selIndex - 1, 0);
        selectAt();
        e.preventDefault();
      } else if (e.key === "g") {
        selIndex = 0;
        selectAt();
      } else if (e.key === "G") {
        selIndex = filtered.length - 1;
        selectAt();
      } else if (e.key === "l" || e.key === "ArrowRight") {
        // Step into the editor (edit mode) for the current note.
        if (previewing && hoveredNote) select(hoveredNote);
        editorMode = "edit";
        setTimeout(
          () => (document.querySelector(".cm-content") as HTMLElement | null)?.focus(),
          30
        );
        e.preventDefault();
      } else if (e.key >= "1" && e.key <= "5") {
        const note = filtered[selIndex];
        if (note) {
          const bit = 1 << (Number(e.key) - 1);
          const newFlags = note.flags ^ bit;
          notes = notes.map((n) => (n.path === note.path ? { ...n, flags: newFlags } : n));
          setNoteTags(note.path, newFlags).catch((err) => flash(`Tag failed: ${err}`));
        }
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

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
    registeredProjects={projects}
    onScope={(s) => (activeScope = s)}
    onProject={(p) => (activeProject = p)}
    onAttach={() => (showAttach = true)}
    {onDetach}
    onMigrate={() => (showMigrate = true)}
  />

  <div class="main">
    <div class="viewbar">
      <input
        class="searchbar"
        placeholder="Search notes…"
        value={searchText}
        oninput={(e) => (searchText = (e.target as HTMLInputElement).value)}
      />
      <div class="spacer"></div>
      {#if toast}<span class="toast">{toast}</span>{/if}
      <button class="primary" onclick={() => (showNewNote = true)}>+ New</button>
      <button class="ghost" onclick={() => (showLog = true)}>Log</button>
      <button class="ghost" onclick={doSync} disabled={syncing}>
        {syncing ? "Syncing…" : "Sync"}
      </button>
    </div>

    <div class="panes">
      <div class="list-col">
        {#if error}
          <div class="status error">{error}</div>
        {:else}
          <NoteList notes={filtered} {selectedPath} onSelect={select} {onHover} />
        {/if}
      </div>

      <NoteEditor
        path={editorPath}
        content={editorContent}
        dim={previewing}
        editable={!previewing}
        bind:mode={editorMode}
        {onSave}
      />

      <Inspector
        title={inspected?.name ?? null}
        scope={inspected?.scope ?? null}
        flags={inspected?.flags ?? 0}
        rows={inspected
          ? [
              { label: "Scope", value: SCOPE_META[inspected.scope].label },
              { label: "Project", value: inspected.project ?? "—" },
              { label: "Path", value: inspected.path },
            ]
          : []}
      />
    </div>
  </div>
</div>

{#if showNewNote}
  <NewNoteDialog {onCreate} onClose={() => (showNewNote = false)} />
{/if}
{#if showLog}
  <LogPanel {onLog} onClose={() => (showLog = false)} />
{/if}
{#if showAttach}
  <AttachProjectDialog {onAttach} onClose={() => (showAttach = false)} />
{/if}
{#if showMigrate}
  <MigrationDialog
    onClose={() => (showMigrate = false)}
    onDone={async () => {
      await refresh();
      await loadProjects();
    }}
  />
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
  .panes {
    display: grid;
    grid-template-columns: 280px 1fr 260px;
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
