<script lang="ts">
  import { onMount } from "svelte";
  import { migratePreview, migrateApply } from "$lib/ipc";
  import type { PlanItem } from "$lib/types";

  let { onClose, onDone }: { onClose: () => void; onDone: () => void } = $props();

  let plan = $state<PlanItem[]>([]);
  let log = $state<string[] | null>(null);
  let loading = $state(true);
  let applying = $state(false);

  onMount(async () => {
    try {
      plan = await migratePreview();
    } catch (e) {
      log = [String(e)];
    } finally {
      loading = false;
    }
  });

  async function apply() {
    applying = true;
    try {
      log = await migrateApply();
      onDone();
    } catch (e) {
      log = [String(e)];
    } finally {
      applying = false;
    }
  }
</script>

<div
  class="overlay"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
>
  <div class="dialog wide">
    <h2>Migrate legacy notes</h2>

    {#if loading}
      <p class="muted">Scanning ~/notez…</p>
    {:else if log}
      <ul class="log">
        {#each log as line, i (i)}
          <li>{line}</li>
        {/each}
      </ul>
      <div class="actions">
        <button class="primary" onclick={onClose}>Done</button>
      </div>
    {:else if plan.length === 0}
      <p class="muted">Nothing to migrate — no legacy project directories found in ~/notez.</p>
      <div class="actions">
        <button class="ghost" onclick={onClose}>Close</button>
      </div>
    {:else}
      <p class="muted">
        Moves each numbered project dir into <code>personal/</code> and attaches the project.
        Your ~/notez is a git repo, so this is reversible.
      </p>
      <div class="plan">
        {#each plan as item (item.name)}
          <div class="prow" class:skip={item.note.includes("skip")}>
            <span class="pname">{item.name}</span>
            <span class="parrow">{item.from} → {item.to}</span>
            <span class="pnote">{item.note}</span>
          </div>
        {/each}
      </div>
      <div class="actions">
        <button class="ghost" onclick={onClose}>Cancel</button>
        <button class="primary" onclick={apply} disabled={applying}>
          {applying ? "Migrating…" : `Migrate ${plan.length}`}
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .wide {
    width: 620px;
  }
  .muted {
    font-size: 0.8rem;
    color: var(--subtext);
    margin: 0;
  }
  code {
    background: rgba(0, 0, 0, 0.3);
    padding: 0.05rem 0.3rem;
    border-radius: 0.25rem;
  }
  .plan {
    max-height: 320px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .prow {
    display: grid;
    grid-template-columns: 110px 1fr auto;
    gap: 0.5rem;
    align-items: center;
    padding: 0.4rem 0.5rem;
    border-radius: 0.4rem;
    background: rgba(0, 0, 0, 0.2);
    font-size: 0.74rem;
  }
  .prow.skip {
    opacity: 0.55;
  }
  .pname {
    font-weight: 600;
    color: var(--text);
  }
  .parrow {
    color: var(--faint);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .pnote {
    color: var(--accent-public);
    font-size: 0.68rem;
  }
  .log {
    margin: 0;
    padding-left: 1.1rem;
    font-size: 0.78rem;
    color: var(--subtext);
    max-height: 320px;
    overflow-y: auto;
  }
</style>
