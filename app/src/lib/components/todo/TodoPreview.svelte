<script lang="ts">
  import type { TodoTask } from "$lib/types";
  import { TAG_DEFS } from "$lib/types";

  let {
    task,
    subtasks = [],
    width = 320,
  }: {
    task: TodoTask | null;
    subtasks?: TodoTask[];
    width?: number;
  } = $props();

  // Indent each descendant relative to the previewed task's depth.
  const baseDepth = $derived(task?.depth ?? 0);

  function mark(state: string): string {
    return state === "checked" ? "✓" : state === "half" ? "–" : "";
  }
  function stateLabel(state: string): string {
    return state === "checked" ? "done" : state === "half" ? "partial" : "pending";
  }
</script>

<aside class="preview" style="width:{width}px">
  {#if !task}
    <div class="empty">Select a todo to preview it.</div>
  {:else}
    <div class="ttl" class:done={task.state === "checked"}>{task.text}</div>

    <div class="meta">
      <span class="badge {task.state}">{stateLabel(task.state)}</span>
      <span class="section">{task.section}</span>
    </div>

    {#if task.flags}
      <div class="tags">
        {#each TAG_DEFS as d (d.bit)}
          {#if (task.flags & d.bit) !== 0}
            <span class="tag" style="--c:{d.color}"><span class="td"></span>{d.label}</span>
          {/if}
        {/each}
      </div>
    {/if}

    <div class="sec-label">Subtasks</div>
    {#if subtasks.length === 0}
      <div class="empty sm">No subtasks.</div>
    {:else}
      <ul class="checklist">
        {#each subtasks as s (s.id)}
          <li
            class="ck"
            class:done={s.state === "checked"}
            style="padding-left:{(s.depth - baseDepth - 1) * 16}px"
          >
            <span class="box {s.state}">{mark(s.state)}</span>
            <span class="txt">{s.text}</span>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</aside>

<style>
  .preview {
    flex-shrink: 0;
    border-left: 1px solid var(--border);
    background: rgba(18, 18, 28, 0.92);
    padding: 1rem;
    overflow-y: auto;
    font-size: 0.82rem;
  }
  .empty {
    color: var(--faint);
    margin-top: 1rem;
  }
  .empty.sm {
    margin-top: 0.4rem;
    font-size: 0.78rem;
  }
  .ttl {
    font-weight: 600;
    font-size: 0.95rem;
    line-height: 1.35;
    color: var(--text);
    word-break: break-word;
  }
  .ttl.done {
    color: var(--faint);
    text-decoration: line-through;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.6rem;
  }
  .badge {
    font-size: 0.66rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0.12rem 0.5rem;
    border-radius: 0.6rem;
    background: var(--glass-hover);
    color: var(--subtext);
  }
  .badge.checked {
    color: var(--accent-public);
    background: color-mix(in srgb, var(--accent-public) 16%, transparent);
  }
  .badge.half {
    color: var(--accent-global);
    background: color-mix(in srgb, var(--accent-global) 16%, transparent);
  }
  .section {
    color: var(--subtext);
    font-size: 0.74rem;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-top: 0.7rem;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.7rem;
    color: var(--c);
    background: color-mix(in srgb, var(--c) 16%, transparent);
    padding: 0.14rem 0.5rem;
    border-radius: 0.6rem;
  }
  .td {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--c);
    box-shadow: 0 0 5px color-mix(in srgb, var(--c) 60%, transparent);
  }
  .sec-label {
    margin-top: 1rem;
    font-size: 0.66rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--faint);
  }
  .checklist {
    list-style: none;
    margin: 0.5rem 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .ck {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    color: var(--text);
  }
  .ck.done .txt {
    color: var(--faint);
    text-decoration: line-through;
  }
  .box {
    width: 15px;
    height: 15px;
    flex-shrink: 0;
    border: 1.5px solid var(--surface-active);
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.2);
    color: #11131a;
    display: grid;
    place-items: center;
    font-size: 0.62rem;
    font-weight: 800;
  }
  .box.checked {
    background: var(--accent-public);
    border-color: var(--accent-public);
  }
  .box.half {
    background: var(--accent-global);
    border-color: var(--accent-global);
  }
  .txt {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
