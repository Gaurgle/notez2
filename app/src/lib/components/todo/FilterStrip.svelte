<script lang="ts">
  import { TAG_DEFS } from "$lib/types";

  let {
    text,
    activeBits,
    onText,
    onToggleBit,
  }: {
    text: string;
    activeBits: number;
    onText: (t: string) => void;
    onToggleBit: (bit: number) => void;
  } = $props();
</script>

<div class="strip">
  <input
    class="search"
    placeholder="filter… (text)"
    value={text}
    oninput={(e) => onText((e.target as HTMLInputElement).value)}
  />
  <div class="dots">
    {#each TAG_DEFS as def (def.bit)}
      <button
        class="dot"
        class:on={(activeBits & def.bit) !== 0}
        style="--c:{def.color}"
        title={def.label}
        aria-label={def.label}
        onclick={() => onToggleBit(def.bit)}
      ></button>
    {/each}
  </div>
</div>

<style>
  .strip {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--surface);
    background: var(--mantle);
  }
  .search {
    flex: 1;
    background: var(--base);
    border: 1px solid var(--surface);
    border-radius: 0.35rem;
    color: var(--text);
    padding: 0.35rem 0.5rem;
    font: inherit;
    font-size: 0.8rem;
  }
  .search:focus {
    outline: none;
    border-color: var(--accent);
  }
  .dots {
    display: flex;
    gap: 0.35rem;
  }
  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1px solid var(--c);
    background: transparent;
    cursor: pointer;
    padding: 0;
    opacity: 0.45;
  }
  .dot.on {
    background: var(--c);
    opacity: 1;
  }
</style>
