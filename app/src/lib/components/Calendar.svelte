<script lang="ts">
  import { onMount } from "svelte";

  // Reusable month calendar. Each view supplies its own context:
  //   marked   — ISO "YYYY-MM-DD" dates to flag with an event dot
  //   selected — parent-owned set of picked days (multi-select); highlighted
  //   onPick   — called when a day is clicked (e.g. toggle a day filter)
  //   onClear  — called by the clear button (shown when selected is non-empty)
  //   label    — small badge text (e.g. "mock"); hidden when empty
  // With no `marked`, it falls back to a decorative mock so todoz stays alive.
  let {
    marked,
    selected,
    onPick,
    onClear,
    label = "",
  }: {
    marked?: Set<string>;
    selected?: Set<string>;
    onPick?: (iso: string, date: Date) => void;
    onClear?: () => void;
    label?: string;
  } = $props();

  let ref = $state<Date | null>(null); // first day of the viewed month
  let today = $state<Date | null>(null);
  let localSel = $state<string | null>(null); // single-select fallback

  onMount(() => {
    const now = new Date();
    today = now;
    ref = new Date(now.getFullYear(), now.getMonth(), 1);
  });

  const WEEKDAYS = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
  const MOCK = new Set([4, 11, 12, 19, 26]); // decorative fallback (current month)

  function iso(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  }

  const monthLabel = $derived(
    ref ? ref.toLocaleString(undefined, { month: "long", year: "numeric" }) : ""
  );

  const cells = $derived.by(() => {
    if (!ref) return [] as (Date | null)[];
    const y = ref.getFullYear();
    const m = ref.getMonth();
    const lead = (new Date(y, m, 1).getDay() + 6) % 7; // Mon = 0
    const days = new Date(y, m + 1, 0).getDate();
    const out: (Date | null)[] = [];
    for (let i = 0; i < lead; i++) out.push(null);
    for (let d = 1; d <= days; d++) out.push(new Date(y, m, d));
    while (out.length % 7 !== 0) out.push(null);
    return out;
  });

  function isToday(d: Date): boolean {
    return !!today && iso(d) === iso(today);
  }
  function hasEvent(d: Date): boolean {
    if (marked) return marked.has(iso(d));
    return !!today && d.getMonth() === today.getMonth() && MOCK.has(d.getDate());
  }
  function isSelected(d: Date): boolean {
    const k = iso(d);
    return selected ? selected.has(k) : localSel === k;
  }
  function pick(d: Date) {
    const k = iso(d);
    if (!selected) localSel = localSel === k ? null : k;
    onPick?.(k, d);
  }
  function step(delta: number) {
    if (!ref) return;
    ref = new Date(ref.getFullYear(), ref.getMonth() + delta, 1);
  }
</script>

<div class="cal">
  <div class="cal-head">
    <span class="title">{monthLabel}</span>
    {#if label}<span class="placeholder">{label}</span>{/if}
    {#if onClear && selected && selected.size > 0}
      <button class="clear" onclick={onClear} title="Clear day filter">clear ({selected.size})</button>
    {/if}
    <div class="nav">
      <button aria-label="previous month" onclick={() => step(-1)}>‹</button>
      <button aria-label="next month" onclick={() => step(1)}>›</button>
    </div>
  </div>

  <div class="grid weekdays">
    {#each WEEKDAYS as w (w)}
      <span class="wd">{w}</span>
    {/each}
  </div>

  <div class="grid days">
    {#each cells as d, i (i)}
      {#if d}
        <button
          class="day"
          class:today={isToday(d)}
          class:selected={isSelected(d)}
          onclick={() => pick(d)}
        >
          <span class="n">{d.getDate()}</span>
          {#if hasEvent(d)}<span class="ev"></span>{/if}
        </button>
      {:else}
        <span class="day empty"></span>
      {/if}
    {/each}
  </div>
</div>

<style>
  .cal {
    flex-shrink: 0;
    border-top: 1px solid var(--border);
    background: rgba(16, 16, 26, 0.96);
    padding: 0.7rem 0.8rem 0.85rem;
  }
  .cal-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.55rem;
  }
  .title {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text);
  }
  .placeholder {
    font-size: 0.56rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--faint);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 0.05rem 0.35rem;
  }
  .clear {
    font-size: 0.6rem;
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 14%, transparent);
    border: none;
    border-radius: 0.5rem;
    padding: 0.1rem 0.4rem;
    cursor: pointer;
    font-family: inherit;
  }
  .clear:hover {
    background: color-mix(in srgb, var(--accent) 24%, transparent);
  }
  .nav {
    margin-left: auto;
    display: flex;
    gap: 0.2rem;
  }
  .nav button {
    width: 22px;
    height: 22px;
    display: grid;
    place-items: center;
    background: var(--glass-hover);
    border: 1px solid var(--border);
    border-radius: 0.4rem;
    color: var(--subtext);
    cursor: pointer;
    font-size: 0.9rem;
    line-height: 1;
  }
  .nav button:hover {
    background: var(--glass-active);
    color: var(--text);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
  }
  .weekdays {
    margin-bottom: 0.25rem;
  }
  .wd {
    text-align: center;
    font-size: 0.6rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--faint);
    padding: 0.15rem 0;
  }
  .day {
    position: relative;
    aspect-ratio: 1 / 1;
    display: grid;
    place-items: center;
    background: none;
    border: 1px solid transparent;
    border-radius: 0.4rem;
    color: var(--subtext);
    cursor: pointer;
    font: inherit;
    font-size: 0.74rem;
  }
  .day.empty {
    cursor: default;
  }
  .day:not(.empty):hover {
    background: var(--surface);
    color: var(--text);
  }
  .day.today {
    color: var(--accent);
    border-color: color-mix(in srgb, var(--accent) 40%, transparent);
    font-weight: 700;
  }
  .day.selected {
    background: color-mix(in srgb, var(--accent) 22%, transparent);
    color: var(--text);
  }
  .ev {
    position: absolute;
    bottom: 3px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--accent-global);
    box-shadow: 0 0 5px color-mix(in srgb, var(--accent-global) 60%, transparent);
  }
</style>
