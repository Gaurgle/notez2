<script lang="ts">
  import MarkdownIt from "markdown-it";
  import hljs from "highlight.js/lib/common";
  import "highlight.js/styles/atom-one-dark.css";

  let { content, dim = false }: { content: string; dim?: boolean } = $props();

  function escapeHtml(s: string): string {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

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

  let rendered = $derived(md.render(content || "*empty note*"));
</script>

<div class="markdown-body" class:dim>
  {@html rendered}
</div>

<style>
  .markdown-body {
    height: 100%;
    overflow-y: auto;
    padding: 1.5rem max(1.75rem, calc((100% - 760px) / 2)) 4rem;
    font-size: 0.95rem;
    line-height: 1.7;
    color: var(--text);
    background: rgba(14, 14, 22, 0.6);
    transition: opacity 0.12s;
  }
  .markdown-body.dim {
    opacity: 0.5;
  }
  .markdown-body :global(> :first-child) {
    margin-top: 0;
  }
  .markdown-body :global(h1),
  .markdown-body :global(h2),
  .markdown-body :global(h3) {
    margin: 1.2rem 0 0.6rem;
    line-height: 1.25;
    font-weight: 700;
  }
  .markdown-body :global(h1) {
    font-size: 1.45rem;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.3rem;
  }
  .markdown-body :global(h2) {
    font-size: 1.2rem;
  }
  .markdown-body :global(h3) {
    font-size: 1.05rem;
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
    background: rgba(0, 0, 0, 0.45) !important;
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
