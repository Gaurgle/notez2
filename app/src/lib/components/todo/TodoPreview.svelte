<script lang="ts">
  import type { TodoTask } from "$lib/types";
  import { TAG_DEFS } from "$lib/types";

  let {
    task,
    subtasks = [],
  }: {
    task: TodoTask | null;
    subtasks?: TodoTask[];
  } = $props();

  const baseDepth = $derived(task?.depth ?? 0);

  function stateLabel(state: string): string {
    return state === "checked" ? "done" : state === "half" ? "in progress" : "pending";
  }

  // Subtask roll-up — an informative summary, not a re-render of the board.
  const doneCount = $derived(subtasks.filter((s) => s.state === "checked").length);
  const pct = $derived(
    subtasks.length === 0 ? 0 : Math.round((doneCount / subtasks.length) * 100)
  );
</script>

<aside class="preview">
  {#if !task}
    <div class="empty">Select a todo to see a summary.</div>
  {:else}
    <div class="ttl">{task.text}</div>

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

    {#if subtasks.length > 0}
      <div class="sec-label">Subtasks · {doneCount} of {subtasks.length} done</div>
      <div class="bar"><span style="width:{pct}%"></span></div>
      <ul class="sublist">
        {#each subtasks as s (s.id)}
          <li
            class="sub"
            class:muted={s.state === "checked"}
            style="padding-left:{(s.depth - baseDepth - 1) * 14}px"
          >
            {s.text}
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</aside>

<style>
  .preview {
    flex: 1;
    min-height: 0;
    background: rgba(18, 18, 28, 0.92);
    padding: 1rem;
    overflow-y: auto;
    font-size: 0.82rem;
  }
  .empty {
    color: var(--faint);
    margin-top: 1rem;
  }
  .ttl {
    font-weight: 600;
    font-size: 0.95rem;
    line-height: 1.35;
    color: var(--text);
    word-break: break-word;
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
  .bar {
    margin-top: 0.4rem;
    height: 5px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--accent-public);
    border-radius: 3px;
    transition: width 0.2s;
  }
  .sublist {
    list-style: none;
    margin: 0.6rem 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .sub {
    color: var(--subtext);
    line-height: 1.3;
    word-break: break-word;
  }
  .sub.muted {
    color: var(--faint);
  }
</style>
