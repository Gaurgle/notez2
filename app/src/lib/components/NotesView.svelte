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
  import MarkdownPreview from "$lib/components/MarkdownPreview.svelte";
  import Resizer from "$lib/components/Resizer.svelte";
  import { Toaster, Toggle } from "melt/builders";
  import {
    Eye,
    PanelRight,
    Plus,
    ScrollText,
    RefreshCw,
    ArrowDownWideNarrow,
    ArrowUpNarrowWide,
    ArrowDownAZ,
  } from "lucide-svelte";
  import { SCOPE_META } from "$lib/types";
  import { mockProjectAuthors, relativeTime } from "$lib/mock";
  import { projectStats, countBy } from "$lib/stats.svelte";
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
  let searchEl = $state<HTMLInputElement>();
  const focusSearch = () => {
    searchEl?.focus();
    searchEl?.select();
  };

  function setScope(s: Scope | "all") {
    activeScope = s;
    activeProject = null;
    selIndex = 0;
  }
  function setProject(p: string | null) {
    activeProject = p;
    activeScope = "all";
    selIndex = 0;
  }

  let selectedPath = $state<string | null>(null);
  let selIndex = $state(0);
  let content = $state(""); // loaded note text (drives the editor; only changes on note switch)
  let liveContent = $state(""); // latest saved text (drives the preview pane)
  // Preview + inspector both open by default.
  let showPreview = $state(true);
  let showInspector = $state(true);
  let sidebarWidth = $state(190);
  let listWidth = $state(280);
  let previewWidth = $state(440);
  let inspectorWidth = $state(210);

  let vimMode = $state(
    typeof localStorage !== "undefined" && localStorage.getItem("notez.vim") === "1"
  );
  let vimStatus = $state(""); // "normal" | "insert" | "visual" | …
  $effect(() => {
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("notez.vim", vimMode ? "1" : "0");
    }
  });

  let hoveredNote = $state<NoteListItem | null>(null);
  let previewContent = $state("");
  let hoverTimer: ReturnType<typeof setTimeout> | undefined;

  // Hover peeks only in the preview pane (dimmed); the editor always stays on
  // the selected note so it's never lost.
  let previewing = $derived(hoveredNote !== null && hoveredNote.path !== selectedPath);
  let inspected = $derived(hoveredNote ?? notes.find((n) => n.path === selectedPath) ?? null);
  let previewPaneContent = $derived(previewing ? previewContent : liveContent);
  let wordCount = $derived(content.trim() ? content.trim().split(/\s+/).length : 0);

  // Hovering a scope/project in the sidebar shows that group's contributors +
  // activity in the inspector; otherwise the inspector reflects the note.
  let hoveredNav = $state<{ kind: "scope" | "project"; name: string } | null>(null);
  let insp = $derived.by(() => {
    if (hoveredNav) {
      const nav = hoveredNav;
      const isScope = nav.kind === "scope";
      const label = isScope
        ? nav.name === "all"
          ? "All notes"
          : SCOPE_META[nav.name as Scope].label
        : nav.name;
      const set = isScope
        ? notes.filter((n) => nav.name === "all" || n.scope === nav.name)
        : notes.filter((n) => n.project === nav.name);
      const latest = set.reduce((m, n) => Math.max(m, n.modified), 0);
      return {
        title: label,
        scope: isScope && nav.name !== "all" ? nav.name : null,
        flags: 0,
        showTags: false,
        people: mockProjectAuthors(nav.name),
        time: latest ? relativeTime(latest) : null,
        rows: [
          { label: isScope ? "Scope" : "Project", value: label },
          { label: "Notes", value: String(set.length) },
          ...(isScope
            ? []
            : [{ label: "Todos", value: String(projectStats.todos[nav.name] ?? 0) }]),
        ],
      };
    }
    const n = inspected;
    if (!n) {
      return {
        title: null,
        scope: null,
        flags: 0,
        showTags: true,
        people: [] as string[],
        time: null as string | null,
        rows: [] as { label: string; value: string }[],
      };
    }
    return {
      title: n.name,
      scope: n.scope as string,
      flags: n.flags,
      showTags: true,
      people: mockProjectAuthors(n.project ?? n.name),
      time: relativeTime(n.modified),
      rows: [
        { label: "Scope", value: SCOPE_META[n.scope].label },
        { label: "Project", value: n.project ?? "—" },
        { label: "Path", value: n.path },
      ],
    };
  });

  function onHover(note: NoteListItem | null) {
    // Volatile peek: leaving a row (or any keypress) reverts to the selected
    // note, so the selected note stays accessible.
    clearTimeout(hoverTimer);
    if (!note) {
      hoveredNote = null;
      return;
    }
    hoveredNote = note;
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
  const toaster = new Toaster<{ message: string }>({ closeDelay: 2500 });

  // Toggle switches bound to the existing pane/mode state, so the p / i /
  // Ctrl+; keyboard shortcuts and the switches stay in sync.
  const previewToggle = new Toggle({ value: () => showPreview, onValueChange: (v) => (showPreview = v) });
  const inspectorToggle = new Toggle({ value: () => showInspector, onValueChange: (v) => (showInspector = v) });
  const vimToggle = new Toggle({ value: () => vimMode, onValueChange: (v) => (vimMode = v) });

  let sortMode = $state<"latest" | "oldest" | "name">("latest");

  let filtered = $derived(
    notes
      .filter(
        (n) =>
          (activeScope === "all" || n.scope === activeScope) &&
          (activeProject === null || n.project === activeProject) &&
          (searchText.trim() === "" ||
            n.name.toLowerCase().includes(searchText.toLowerCase()))
      )
      .sort((a, b) =>
        sortMode === "name"
          ? a.name.localeCompare(b.name)
          : sortMode === "oldest"
            ? a.modified - b.modified
            : b.modified - a.modified
      )
  );

  async function refresh(selectPath?: string) {
    try {
      notes = await listNotes();
      projectStats.notes = countBy(notes, (n) => n.project);
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
    liveContent = content;
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
      // Cmd/Ctrl+F jumps to search from anywhere, including while editing
      // (where "/" must type a literal slash).
      if ((e.metaKey || e.ctrlKey) && (e.key === "f" || e.key === "F")) {
        focusSearch();
        e.preventDefault();
        return;
      }
      // Ctrl+; toggles vim mode from anywhere, including inside the editor.
      if (e.ctrlKey && e.key === ";") {
        vimMode = !vimMode;
        e.preventDefault();
        return;
      }
      if (isTyping(e.target)) {
        if (e.key === "Escape") (e.target as HTMLElement).blur();
        return;
      }
      hoveredNote = null; // any key reverts a hover-peek to the selected note
      if (e.key === "/") {
        focusSearch();
        e.preventDefault();
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
      } else if (e.key === "l" || e.key === "ArrowRight" || e.key === "Enter") {
        // Step into the editor for the current note.
        if (previewing && hoveredNote) select(hoveredNote);
        setTimeout(
          () => (document.querySelector(".cm-content") as HTMLElement | null)?.focus(),
          0
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
      } else if (e.key === "p") {
        showPreview = !showPreview;
      } else if (e.key === "i") {
        showInspector = !showInspector;
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  async function onSave(newContent: string) {
    if (!selectedPath) return;
    liveContent = newContent; // keep the preview pane in sync with saved edits
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

  function flash(msg: string) {
    toaster.addToast({ data: { message: msg } });
  }
</script>

<div class="notes">
  <Sidebar
    {notes}
    {activeScope}
    {activeProject}
    registeredProjects={projects}
    onScope={setScope}
    onProject={setProject}
    onAttach={() => (showAttach = true)}
    {onDetach}
    onMigrate={() => (showMigrate = true)}
    onHover={(item) => (hoveredNav = item)}
    width={sidebarWidth}
  />
  <Resizer get={() => sidebarWidth} set={(n) => (sidebarWidth = n)} dir={1} min={170} max={420} />

  <div class="main">
    <div class="viewbar">
      <input
        class="searchbar"
        placeholder="Search notes…  ( / )"
        bind:this={searchEl}
        value={searchText}
        oninput={(e) => (searchText = (e.target as HTMLInputElement).value)}
      />
      <span class="counts">{filtered.length} / {notes.length}</span>
      <button
        class="ghost sortbtn iconbtn"
        title="Sort order"
        onclick={() =>
          (sortMode = sortMode === "latest" ? "oldest" : sortMode === "oldest" ? "name" : "latest")}
      >
        {#if sortMode === "latest"}<ArrowDownWideNarrow size={14} />
        {:else if sortMode === "oldest"}<ArrowUpNarrowWide size={14} />
        {:else}<ArrowDownAZ size={14} />{/if}
        <span>{sortMode === "latest" ? "Latest" : sortMode === "oldest" ? "Oldest" : "Name"}</span>
      </button>
      <div class="spacer"></div>
      <button class="ghost iconbtn icononly" onclick={() => (showNewNote = true)} title="New note" aria-label="New note">
        <Plus size={16} />
      </button>
      <button class="ghost iconbtn icononly" onclick={() => (showLog = true)} title="Log" aria-label="Log">
        <ScrollText size={15} />
      </button>
      <button class="ghost iconbtn icononly" onclick={doSync} disabled={syncing} title="Sync" aria-label="Sync">
        <span class="ico" class:spin={syncing}><RefreshCw size={15} /></span>
      </button>
    </div>

    <div class="panes">
      <div class="list-col" style="width:{listWidth}px">
        {#if error}
          <div class="status error">{error}</div>
        {:else}
          <NoteList
            notes={filtered}
            {selectedPath}
            onSelect={select}
            {onHover}
            inProjectMode={activeProject !== null}
          />
        {/if}
      </div>
      <Resizer get={() => listWidth} set={(n) => (listWidth = n)} dir={1} min={180} max={520} />

      <div class="editor-col">
        <NoteEditor
          path={selectedPath}
          {content}
          {onSave}
          {vimMode}
          onVimMode={(m) => (vimStatus = m)}
        />
      </div>

      {#if showPreview}
        <Resizer get={() => previewWidth} set={(n) => (previewWidth = n)} dir={-1} min={280} max={1100} />
        <div class="preview-col" style="width:{previewWidth}px">
          <MarkdownPreview content={previewPaneContent} dim={previewing} />
        </div>
      {/if}

      {#if showInspector}
        <Resizer get={() => inspectorWidth} set={(n) => (inspectorWidth = n)} dir={-1} min={180} max={520} />
        <Inspector
          width={inspectorWidth}
          title={insp.title}
          scope={insp.scope}
          flags={insp.flags}
          showTags={insp.showTags}
          people={insp.people}
          time={insp.time}
          rows={insp.rows}
        />
      {/if}
    </div>

    <div class="statusbar">
      <span class="sb-path">{selectedPath ?? "no note selected"}</span>
      <div class="sb-spacer"></div>
      {#if vimMode && vimStatus}
        <span class="vim-mode {vimStatus.split('-')[0]}">{vimStatus.replace('-', ' ').toUpperCase()}</span>
      {/if}
      {#if selectedPath}
        <span class="sb-count">{content.length} chars · {wordCount} words</span>
      {/if}
      <div class="pane-toggles">
        <button class="pane-toggle" class:on={previewToggle.value} {...previewToggle.trigger} title="Preview (p)" aria-label="Toggle preview">
          <Eye size={14} />
        </button>
        <button class="pane-toggle" class:on={inspectorToggle.value} {...inspectorToggle.trigger} title="Inspector (i)" aria-label="Toggle inspector">
          <PanelRight size={14} />
        </button>
      </div>
      <button class="vim-pill" class:on={vimToggle.value} {...vimToggle.trigger} title="Toggle vim mode (Ctrl+;)">
        VIM
      </button>
    </div>
  </div>
</div>

<div class="toaster" {...toaster.root}>
  {#each toaster.toasts as t (t.id)}
    <div class="toast-card" {...t.content}>
      <span {...t.title}>{t.data.message}</span>
      <button class="toast-x" {...t.close} aria-label="dismiss">×</button>
    </div>
  {/each}
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
    display: flex;
    height: 100%;
    overflow: hidden;
  }
  .toaster {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    z-index: 50;
  }
  .toast-card {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background: var(--glass-strong);
    -webkit-backdrop-filter: var(--blur);
    backdrop-filter: var(--blur);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow);
    padding: 0.55rem 0.8rem;
    font-size: 0.8rem;
    color: var(--text);
    animation: pop 0.18s cubic-bezier(0.2, 0.9, 0.3, 1.2);
  }
  .toast-x {
    background: none;
    border: none;
    color: var(--faint);
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    padding: 0;
  }
  .toast-x:hover {
    color: var(--text);
  }
  .main {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    /* Opaque content region — only the rail + sidebar are translucent glass. */
    background: var(--base);
  }
  .panes {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  .list-col {
    flex-shrink: 0;
    min-height: 0;
    overflow: hidden;
  }
  .editor-col {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .preview-col {
    flex-shrink: 0;
    min-width: 0;
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

  .statusbar {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.25rem 0.75rem;
    border-top: 1px solid var(--border);
    background: var(--mantle);
    font-size: 0.68rem;
    color: var(--faint);
    flex-shrink: 0;
  }
  .sb-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 50%;
  }
  .sb-spacer {
    flex: 1;
  }
  .sb-count {
    color: var(--subtext);
    white-space: nowrap;
  }
  .vim-pill {
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    padding: 0.12rem 0.55rem;
    border-radius: 0.6rem;
    border: 1px solid var(--border);
    background: var(--glass-hover);
    color: var(--faint);
    cursor: pointer;
  }
  .vim-pill.on {
    color: var(--accent-public);
    background: color-mix(in srgb, var(--accent-public) 14%, transparent);
    border-color: color-mix(in srgb, var(--accent-public) 32%, transparent);
  }
  .sortbtn {
    padding: 0.42rem 0.7rem;
    font-size: 0.74rem;
    font-weight: 500;
    color: var(--subtext);
    min-width: 4.4rem;
  }
  .vim-mode {
    font-weight: 800;
    font-size: 0.62rem;
    letter-spacing: 0.06em;
    padding: 0.1rem 0.5rem;
    border-radius: 0.4rem;
    color: #11131a;
    white-space: nowrap;
  }
  .vim-mode.normal {
    background: var(--accent-local);
  }
  .vim-mode.insert {
    background: var(--accent-public);
  }
  .vim-mode.visual {
    background: var(--accent-global);
  }
  .vim-mode.replace {
    background: var(--danger);
  }
</style>
