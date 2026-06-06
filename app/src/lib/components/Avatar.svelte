<script lang="ts">
  import { hashStr } from "$lib/mock";

  let { name, size = 18 }: { name: string; size?: number } = $props();

  const COLORS = [
    "#cba6f7",
    "#89b4fa",
    "#a6e3a1",
    "#fab387",
    "#f38ba8",
    "#94e2d5",
    "#f9e2af",
    "#eba0ac",
  ];
  let color = $derived(COLORS[hashStr(name) % COLORS.length]);
</script>

<span
  class="avatar"
  title={name}
  style="--ac:{color}; width:{size}px; height:{size}px; font-size:{Math.round(size * 0.46)}px"
>
  {name[0]?.toUpperCase() ?? "?"}
</span>

<style>
  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    flex-shrink: 0;
    background: color-mix(in srgb, var(--ac) 28%, transparent);
    color: var(--ac);
    font-weight: 700;
    border: 1px solid var(--base);
  }
</style>
