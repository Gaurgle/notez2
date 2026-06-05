<script lang="ts">
  let {
    onAttach,
    onClose,
  }: {
    onAttach: (name: string, path: string) => void;
    onClose: () => void;
  } = $props();

  let name = $state("");
  let path = $state("");

  function submit(e: Event) {
    e.preventDefault();
    if (name.trim() && path.trim()) onAttach(name.trim(), path.trim());
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
    <h2>Attach project</h2>
    <label>
      Name
      <!-- svelte-ignore a11y_autofocus -->
      <input bind:value={name} autofocus placeholder="my-project" />
    </label>
    <label>
      Path
      <input bind:value={path} placeholder="~/repos/my-project" />
    </label>
    <div class="actions">
      <button type="button" class="ghost" onclick={onClose}>Cancel</button>
      <button type="submit" class="primary">Attach</button>
    </div>
  </form>
</div>
