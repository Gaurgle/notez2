<script lang="ts">
  import type { Scope } from "$lib/types";
  import { SCOPE_META } from "$lib/types";

  let {
    onCreate,
    onClose,
  }: {
    onCreate: (scope: Scope, title: string, body: string | null) => void;
    onClose: () => void;
  } = $props();

  let title = $state("");
  let body = $state("");
  let scope = $state<Scope>("personal");

  const scopes: Scope[] = ["personal", "public", "local", "global"];

  function submit(e: Event) {
    e.preventDefault();
    onCreate(scope, title, body.trim() ? body : null);
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
    <label>
      Scope
      <select bind:value={scope}>
        {#each scopes as s (s)}
          <option value={s}>{SCOPE_META[s].label}</option>
        {/each}
      </select>
    </label>
    <div class="actions">
      <button type="button" class="ghost" onclick={onClose}>Cancel</button>
      <button type="submit" class="primary">Create</button>
    </div>
  </form>
</div>
