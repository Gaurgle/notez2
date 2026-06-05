<script lang="ts">
  import type { Scope } from "$lib/types";
  import { SCOPE_META } from "$lib/types";

  let {
    onLog,
    onClose,
  }: {
    onLog: (scope: Scope, message: string) => void;
    onClose: () => void;
  } = $props();

  let message = $state("");
  let scope = $state<Scope>("global");

  const scopes: Scope[] = ["global", "personal"];

  function submit(e: Event) {
    e.preventDefault();
    if (message.trim()) onLog(scope, message);
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
    <h2>Daily log entry</h2>
    <label>
      Message
      <!-- svelte-ignore a11y_autofocus -->
      <input bind:value={message} placeholder="what happened…" autofocus />
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
      <button type="submit" class="primary">Append</button>
    </div>
  </form>
</div>
