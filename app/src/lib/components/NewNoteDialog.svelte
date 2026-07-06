<script lang="ts">
  import type { Scope } from "$lib/types";
  import { SCOPE_META } from "$lib/types";

  let {
    onCreate,
    onClose,
    targetFolder = null,
    targetDisplay = "",
  }: {
    onCreate: (scope: Scope, title: string, body: string | null, dir: string | null) => void;
    onClose: () => void;
    /** Folder selected in the sidebar tree (root-relative). Locks the note
     *  to the global scope and creates it inside that folder. */
    targetFolder?: string | null;
    /** Tilde-contracted notez root, for the target hint. */
    targetDisplay?: string;
  } = $props();

  let title = $state("");
  let body = $state("");
  let scope = $state<Scope>("personal");

  const scopes: Scope[] = ["personal", "public", "local", "global"];

  function submit(e: Event) {
    e.preventDefault();
    // A folder target implies the global scope (the select is hidden then).
    onCreate(targetFolder ? "global" : scope, title, body.trim() ? body : null, targetFolder);
  }
</script>

<div
  class="overlay"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
>
  <form class="dialog" onsubmit={submit}>
    <h2>New note</h2>
    <label>
      Title
      <!-- svelte-ignore a11y_autofocus -->
      <input bind:value={title} placeholder="untitled" autofocus />
    </label>
    <label>
      Body (optional)
      <textarea bind:value={body} rows="4"></textarea>
    </label>
    {#if targetFolder}
      <div class="target">
        Creates in <code>{targetDisplay}/{targetFolder}/</code>
      </div>
    {:else}
      <label>
        Scope
        <select bind:value={scope}>
          {#each scopes as s (s)}
            <option value={s}>{SCOPE_META[s].label}</option>
          {/each}
        </select>
      </label>
      <div class="scope-hint">{SCOPE_META[scope].hint}</div>
    {/if}
    <div class="actions">
      <button type="button" class="ghost" onclick={onClose}>Cancel</button>
      <button type="submit" class="primary">Create</button>
    </div>
  </form>
</div>

<style>
  .scope-hint {
    font-size: 0.7rem;
    color: var(--faint);
    margin-top: -0.35rem;
  }
  .target {
    font-size: 0.72rem;
    color: var(--subtext);
  }
  .target code {
    color: var(--accent-global);
    background: var(--surface);
    padding: 0.05rem 0.3rem;
    border-radius: 0.3rem;
  }
</style>
