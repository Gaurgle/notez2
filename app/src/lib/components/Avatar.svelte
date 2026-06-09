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

  // Custom hover tooltip — snappier than the native `title` delay, and portaled
  // to <body> so it isn't clipped by a card's overflow / Gridstack transforms.
  let el: HTMLElement;
  let show = $state(false);
  let tx = $state(0);
  let ty = $state(0);
  let timer: ReturnType<typeof setTimeout> | undefined;

  function enter() {
    timer = setTimeout(() => {
      if (!el) return;
      const r = el.getBoundingClientRect();
      tx = r.left + r.width / 2;
      ty = r.top;
      show = true;
    }, 130);
  }
  function leave() {
    clearTimeout(timer);
    show = false;
  }

  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        node.remove();
      },
    };
  }
</script>

<span
  bind:this={el}
  class="avatar"
  role="img"
  aria-label={name}
  onmouseenter={enter}
  onmouseleave={leave}
  style="--ac:{color}; width:{size}px; height:{size}px; font-size:{Math.round(size * 0.46)}px"
>
  {name[0]?.toUpperCase() ?? "?"}
</span>

{#if show}
  <span class="avatar-tip" use:portal style="left:{tx}px; top:{ty}px">{name}</span>
{/if}

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
  /* portaled to <body>, so it must be a global style */
  :global(.avatar-tip) {
    position: fixed;
    transform: translate(-50%, calc(-100% - 7px));
    background: var(--surface-active, #2a2a3a);
    color: var(--text, #fff);
    font-size: 0.66rem;
    font-weight: 600;
    line-height: 1;
    padding: 0.22rem 0.45rem;
    border-radius: 0.45rem;
    white-space: nowrap;
    pointer-events: none;
    z-index: 9999;
    border: 1px solid var(--border, rgba(255, 255, 255, 0.1));
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.4);
  }
</style>
