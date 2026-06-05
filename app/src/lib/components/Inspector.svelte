<script lang="ts">
  import { TAG_DEFS } from "$lib/types";

  let {
    title,
    scope = null,
    flags = 0,
    rows = [],
  }: {
    title: string | null;
    scope?: string | null;
    flags?: number;
    rows?: { label: string; value: string }[];
  } = $props();
</script>

<aside class="inspector">
  {#if !title}
    <div class="empty">Hover or select an item to inspect it.</div>
  {:else}
    <div class="title">{title}</div>
    {#if scope}
      <span class="badge {scope}">{scope}</span>
    {/if}

    <div class="sec-label">Importance</div>
    <div class="tags">
      {#each TAG_DEFS as d (d.bit)}
        <span class="tag" class:on={(flags & d.bit) !== 0} style="--c:{d.color}">
          <span class="td"></span>{d.label}
        </span>
      {/each}
    </div>

    <dl>
      {#each rows as r (r.label)}
        <dt>{r.label}</dt>
        <dd>{r.value}</dd>
      {/each}
    </dl>
  {/if}
</aside>

<style>
  .inspector {
    width: 260px;
    flex-shrink: 0;
    border-left: 1px solid var(--border);
    background: var(--mantle);
    padding: 1rem;
    overflow-y: auto;
    font-size: 0.82rem;
  }
  .empty {
    color: var(--faint);
    font-size: 0.8rem;
    margin-top: 1rem;
  }
  .title {
    font-weight: 700;
    font-size: 0.95rem;
    line-height: 1.3;
    /* Always reserve exactly two rows so the panel never resizes with the
       title length; longer titles clamp with an ellipsis. */
    height: 2.6em;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    color: var(--text);
    word-break: break-word;
    margin-bottom: 0.5rem;
  }
  .badge {
    display: inline-block;
    font-size: 0.66rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0.12rem 0.5rem;
    border-radius: 0.6rem;
    background: var(--glass-hover);
    color: var(--subtext);
  }
  .badge.personal {
    color: var(--accent-personal);
  }
  .badge.public {
    color: var(--accent-public);
  }
  .badge.local {
    color: var(--accent-local);
  }
  .badge.global {
    color: var(--accent-global);
  }
  .sec-label {
    margin-top: 0.85rem;
    font-size: 0.66rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--faint);
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin: 0.4rem 0 0;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.7rem;
    color: var(--faint);
    background: rgba(255, 255, 255, 0.04);
    padding: 0.14rem 0.5rem;
    border-radius: 0.6rem;
    opacity: 0.6;
    transition: opacity 0.12s;
  }
  .tag.on {
    color: var(--c);
    background: color-mix(in srgb, var(--c) 16%, transparent);
    opacity: 1;
  }
  .td {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--faint);
  }
  .tag.on .td {
    background: var(--c);
    box-shadow: 0 0 5px color-mix(in srgb, var(--c) 60%, transparent);
  }
  dl {
    margin: 1rem 0 0;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 0.75rem;
  }
  dt {
    color: var(--faint);
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  dd {
    margin: 0;
    color: var(--subtext);
    word-break: break-all;
  }
</style>
