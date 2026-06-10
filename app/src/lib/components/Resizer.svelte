<script lang="ts">
  let {
    get,
    set,
    dir = 1,
    min = 160,
    max = 1100,
  }: {
    get: () => number;
    set: (n: number) => void;
    dir?: 1 | -1;
    min?: number;
    max?: number;
  } = $props();

  function start(e: PointerEvent) {
    const startX = e.clientX;
    const startW = get();
    const move = (ev: PointerEvent) =>
      set(Math.max(min, Math.min(max, startW + dir * (ev.clientX - startX))));
    const stop = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", stop);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", stop);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none"; // no text highlight while dragging
    e.preventDefault();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="resizer" onpointerdown={start}></div>

<style>
  .resizer {
    width: 5px;
    flex-shrink: 0;
    cursor: col-resize;
    background: var(--border);
    transition: background 0.12s;
  }
  .resizer:hover {
    background: var(--accent);
  }
</style>
