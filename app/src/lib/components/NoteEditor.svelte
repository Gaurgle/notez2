<script lang="ts">
  import { EditorState } from "@codemirror/state";
  import { EditorView, lineNumbers, keymap } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { languages } from "@codemirror/language-data";
  import { oneDark } from "@codemirror/theme-one-dark";
  import MarkdownIt from "markdown-it";
  import hljs from "highlight.js";
  import "highlight.js/styles/atom-one-dark.css";

  let {
    path,
    content,
    onSave,
    dim = false,
    editable = true,
    mode = $bindable<"read" | "edit">("read"),
  }: {
    path: string | null;
    content: string;
    onSave: (content: string) => void;
    dim?: boolean;
    editable?: boolean;
    mode?: "read" | "edit";
  } = $props();

  function escapeHtml(s: string): string {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  // Read-mode renderer: markdown-it + highlight.js for fenced code blocks.
  const md = new MarkdownIt({
    html: false,
    linkify: true,
    breaks: false,
    highlight: (str: string, lang: string): string => {
      if (lang && hljs.getLanguage(lang)) {
        try {
          return `<pre class="hljs"><code>${hljs.highlight(str, { language: lang, ignoreIllegals: true }).value}</code></pre>`;
        } catch {
          /* fall through */
        }
      }
      return `<pre class="hljs"><code>${escapeHtml(str)}</code></pre>`;
    },
  });

  let host = $state<HTMLDivElement>();
  let view: EditorView | undefined;
  let applyingExternal = false;
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let saved = $state(true);

  // Preview (non-editable) is always rendered; otherwise honor the toggle.
  let effectiveMode = $derived(!editable ? "read" : mode);
  let rendered = $derived(md.render(content || "*empty note*"));

  // Each freshly opened note starts in read mode.
  $effect(() => {
    path;
    mode = "read";
  });

  function scheduleSave(doc: string) {
    saved = false;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      onSave(doc);
      saved = true;
    }, 500);
  }

  function buildState(doc: string): EditorState {
    return EditorState.create({
      doc,
      extensions: [
        lineNumbers(),
        history(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        EditorView.lineWrapping,
        markdown({ codeLanguages: languages }),
        oneDark,
        EditorView.updateListener.of((u) => {
          if (u.docChanged && !applyingExternal) scheduleSave(u.state.doc.toString());
        }),
      ],
    });
  }

  // Build / tear down the CodeMirror view as edit mode and content change.
  $effect(() => {
    effectiveMode;
    content;
    if (effectiveMode !== "edit" || !host) {
      view?.destroy();
      view = undefined;
      return;
    }
    const state = buildState(content);
    applyingExternal = true;
    if (!view) view = new EditorView({ state, parent: host });
    else view.setState(state);
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
        <button class="mode-toggle" onclick={() => (mode = mode === "read" ? "edit" : "read")}>
          {mode === "read" ? "✎ Edit" : "👁 Read"}
        </button>
        {#if mode === "edit"}
          <span class="save-state" class:dirty={!saved}>{saved ? "saved" : "saving…"}</span>
        {/if}
      {/if}
    </div>

    {#if effectiveMode === "edit"}
      <div class="editor" bind:this={host}></div>
    {:else}
      <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
      <div
        class="markdown-body"
        ondblclick={() => {
          if (editable) mode = "edit";
        }}
      >
        {@html rendered}
      </div>
    {/if}
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
  .mode-toggle {
    background: var(--glass-hover);
    border: 1px solid var(--border);
    border-radius: 0.4rem;
    color: var(--text);
    font: inherit;
    font-size: 0.7rem;
    padding: 0.2rem 0.55rem;
    cursor: pointer;
  }
  .mode-toggle:hover {
    background: var(--glass-active);
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
  .editor-pane.dim .editor,
  .editor-pane.dim .markdown-body {
    opacity: 0.5;
  }
  .editor-empty {
    display: grid;
    place-items: center;
    height: 100%;
    color: var(--subtext);
    font-size: 0.85rem;
  }

  /* Rendered markdown */
  .markdown-body {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
    padding: 1.25rem 1.75rem;
    font-size: 0.92rem;
    line-height: 1.65;
    color: var(--text);
  }
  .markdown-body :global(h1),
  .markdown-body :global(h2),
  .markdown-body :global(h3) {
    margin: 1.2rem 0 0.6rem;
    line-height: 1.25;
    font-weight: 700;
  }
  .markdown-body :global(h1) {
    font-size: 1.5rem;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.3rem;
  }
  .markdown-body :global(h2) {
    font-size: 1.25rem;
  }
  .markdown-body :global(h3) {
    font-size: 1.08rem;
  }
  .markdown-body :global(p) {
    margin: 0.6rem 0;
  }
  .markdown-body :global(a) {
    color: var(--accent-local);
    text-decoration: none;
  }
  .markdown-body :global(a:hover) {
    text-decoration: underline;
  }
  .markdown-body :global(ul),
  .markdown-body :global(ol) {
    margin: 0.5rem 0;
    padding-left: 1.5rem;
  }
  .markdown-body :global(li) {
    margin: 0.2rem 0;
  }
  .markdown-body :global(code) {
    background: rgba(255, 255, 255, 0.08);
    padding: 0.1rem 0.35rem;
    border-radius: 0.3rem;
    font-family: ui-monospace, "SF Mono", monospace;
    font-size: 0.85em;
  }
  .markdown-body :global(pre) {
    background: rgba(0, 0, 0, 0.4) !important;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 0.85rem 1rem;
    overflow-x: auto;
    margin: 0.75rem 0;
  }
  .markdown-body :global(pre code) {
    background: none;
    padding: 0;
    font-size: 0.82rem;
    line-height: 1.5;
  }
  .markdown-body :global(blockquote) {
    border-left: 3px solid var(--accent);
    margin: 0.75rem 0;
    padding: 0.1rem 0 0.1rem 1rem;
    color: var(--subtext);
  }
  .markdown-body :global(hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 1.25rem 0;
  }
  .markdown-body :global(table) {
    border-collapse: collapse;
    margin: 0.75rem 0;
  }
  .markdown-body :global(th),
  .markdown-body :global(td) {
    border: 1px solid var(--border);
    padding: 0.35rem 0.6rem;
  }
  .markdown-body :global(img) {
    max-width: 100%;
    border-radius: 0.4rem;
  }
</style>
