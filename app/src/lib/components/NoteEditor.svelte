<script lang="ts">
  import { EditorState } from "@codemirror/state";
  import { EditorView, lineNumbers, keymap } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { languages } from "@codemirror/language-data";
  import { oneDark } from "@codemirror/theme-one-dark";
  import { vim, getCM } from "@replit/codemirror-vim";

  let {
    path,
    content,
    onSave,
    dim = false,
    editable = true,
    vimMode = false,
    onVimMode,
  }: {
    path: string | null;
    content: string;
    onSave: (content: string) => void;
    dim?: boolean;
    editable?: boolean;
    vimMode?: boolean;
    onVimMode?: (mode: string) => void;
  } = $props();

  let host = $state<HTMLDivElement>();
  let view: EditorView | undefined;
  let applyingExternal = false;
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let saved = $state(true);

  function scheduleSave(doc: string) {
    saved = false;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      onSave(doc);
      saved = true;
    }, 500);
  }

  function buildState(doc: string): EditorState {
    // vim() must come first so its keymap takes precedence.
    const base = vimMode ? [vim()] : [];
    base.push(lineNumbers(), EditorView.lineWrapping, markdown({ codeLanguages: languages }), oneDark);
    if (!editable) {
      base.push(EditorState.readOnly.of(true), EditorView.editable.of(false));
      return EditorState.create({ doc, extensions: base });
    }
    base.push(
      history(),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      EditorView.updateListener.of((u) => {
        if (u.docChanged && !applyingExternal) scheduleSave(u.state.doc.toString());
      })
    );
    return EditorState.create({ doc, extensions: base });
  }

  $effect(() => {
    // Read both reactive inputs up front so they're always tracked, even on
    // the first run when `host` isn't bound yet.
    const doc = content;
    const ed = editable;
    const vm = vimMode;
    void ed;
    void vm;
    if (!host) return;
    // Recreate the view on each content/editable change — robust against
    // container-size/measure timing issues from `setState`.
    view?.destroy();
    applyingExternal = true;
    view = new EditorView({ state: buildState(doc), parent: host });
    applyingExternal = false;
    saved = true;

    if (vm && editable) {
      const cm = getCM(view);
      onVimMode?.("normal");
      cm?.on("vim-mode-change", (e: { mode: string; subMode?: string }) => {
        onVimMode?.(e.subMode ? `${e.mode}-${e.subMode}` : e.mode);
      });
    } else {
      onVimMode?.("");
    }
  });

  $effect(() => () => {
    clearTimeout(saveTimer);
    view?.destroy();
  });
</script>

<div class="editor-pane" class:dim>
  {#if path}
    <div class="editor-bar">
      <span class="editor-path">{path}</span>
      {#if dim}
        <span class="preview-tag">preview</span>
      {:else}
        <span class="save-state" class:dirty={!saved}>{saved ? "saved" : "saving…"}</span>
      {/if}
    </div>
    <div class="editor" bind:this={host}></div>
  {:else}
    <div class="editor-empty">Select a note, or create one with “+ New”.</div>
  {/if}
</div>

<style>
  .editor-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
    background: rgba(18, 18, 28, 0.94);
  }
  .editor-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }
  .editor-path {
    flex: 1;
    font-size: 0.72rem;
    color: var(--subtext);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .save-state {
    font-size: 0.68rem;
    color: var(--accent-public);
  }
  .save-state.dirty {
    color: var(--accent-global);
  }
  .preview-tag {
    font-size: 0.66rem;
    color: var(--accent);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .editor {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }
  .editor :global(.cm-editor) {
    height: 100%;
    font-size: 0.85rem;
  }
  .editor :global(.cm-editor),
  .editor :global(.cm-gutters) {
    background: transparent !important;
  }
  .editor :global(.cm-activeLine),
  .editor :global(.cm-activeLineGutter) {
    background: rgba(255, 255, 255, 0.045) !important;
  }
  .editor :global(.cm-gutters) {
    border-right: none;
    color: var(--faint);
  }
  .editor-pane.dim .editor {
    opacity: 0.5;
  }
  .editor-empty {
    display: grid;
    place-items: center;
    height: 100%;
    color: var(--subtext);
    font-size: 0.85rem;
  }
</style>
