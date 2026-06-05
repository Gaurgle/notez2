<script lang="ts">
  import { EditorState } from "@codemirror/state";
  import { EditorView, lineNumbers, keymap } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { oneDark } from "@codemirror/theme-one-dark";

  let {
    path,
    content,
    onSave,
    dim = false,
    editable = true,
  }: {
    path: string | null;
    content: string;
    onSave: (content: string) => void;
    dim?: boolean;
    editable?: boolean;
  } = $props();

  let host = $state<HTMLDivElement>();
  let view: EditorView | undefined;
  // True while we apply external content (note switch / reload) so the
  // update listener doesn't echo that back as a user edit.
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
    const base = [lineNumbers(), EditorView.lineWrapping, markdown(), oneDark];
    if (!editable) {
      base.push(EditorState.readOnly.of(true), EditorView.editable.of(false));
      return EditorState.create({ doc, extensions: base });
    }
    base.push(
      history(),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      EditorView.updateListener.of((u) => {
        if (u.docChanged && !applyingExternal) {
          scheduleSave(u.state.doc.toString());
        }
      })
    );
    return EditorState.create({ doc, extensions: base });
  }

  // Re-runs whenever `content` or `editable` changes.
  $effect(() => {
    editable; // track
    if (!host) return;
    const state = buildState(content);
    applyingExternal = true;
    if (!view) {
      view = new EditorView({ state, parent: host });
    } else {
      view.setState(state);
    }
    applyingExternal = false;
    saved = true;
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
    border-bottom: 1px solid var(--surface);
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
  .editor-pane.dim .editor {
    opacity: 0.5;
    transition: opacity 0.1s;
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
  /* Let the glass pane show through the editor instead of oneDark's slab. */
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
  .editor-empty {
    display: grid;
    place-items: center;
    height: 100%;
    color: var(--subtext);
    font-size: 0.85rem;
  }
</style>
