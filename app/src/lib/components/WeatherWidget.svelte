<script lang="ts">
  import { onMount } from "svelte";

  // Live weather via Open-Meteo (no API key). Location resolved from the
  // browser geolocation (with consent) → IP lookup → Stockholm default.
  // Falls back to mock data if anything fails, so the widget always renders.

  let place = $state("…");
  let temp = $state<number | null>(null);
  let humidity = $state<number | null>(null);
  let wind = $state<number | null>(null);
  let code = $state(2);
  let isDay = $state(true);
  let loading = $state(true);

  type Scene = "clear" | "cloud" | "rain" | "snow" | "fog" | "storm";
  let scene = $derived(toScene(code));
  let label = $derived(toLabel(code));

  function toScene(c: number): Scene {
    if (c === 0 || c === 1) return "clear";
    if (c === 45 || c === 48) return "fog";
    if (c >= 95) return "storm";
    if ((c >= 71 && c <= 77) || c === 85 || c === 86) return "snow";
    if ((c >= 51 && c <= 67) || (c >= 80 && c <= 82)) return "rain";
    return "cloud";
  }
  function toLabel(c: number): string {
    const map: Record<number, string> = {
      0: "Clear",
      1: "Mainly clear",
      2: "Partly cloudy",
      3: "Overcast",
      45: "Fog",
      48: "Rime fog",
      51: "Light drizzle",
      53: "Drizzle",
      55: "Dense drizzle",
      61: "Light rain",
      63: "Rain",
      65: "Heavy rain",
      71: "Light snow",
      73: "Snow",
      75: "Heavy snow",
      80: "Rain showers",
      81: "Showers",
      82: "Violent showers",
      85: "Snow showers",
      95: "Thunderstorm",
      96: "Thunderstorm",
      99: "Thunderstorm",
    };
    return map[c] ?? "Cloudy";
  }

  async function resolveLocation(): Promise<{ lat: number; lon: number; name: string }> {
    // 1) geolocation (asks consent), short timeout so it can't hang the widget
    try {
      const pos = await new Promise<GeolocationPosition>((res, rej) => {
        if (!navigator.geolocation) return rej();
        navigator.geolocation.getCurrentPosition(res, rej, { timeout: 4000 });
      });
      return { lat: pos.coords.latitude, lon: pos.coords.longitude, name: "Your location" };
    } catch {
      /* fall through */
    }
    // 2) IP-based lookup (no prompt)
    try {
      const r = await fetch("https://ipapi.co/json/");
      const j = await r.json();
      if (typeof j.latitude === "number") {
        return { lat: j.latitude, lon: j.longitude, name: j.city ?? "Unknown" };
      }
    } catch {
      /* fall through */
    }
    // 3) default
    return { lat: 59.3293, lon: 18.0686, name: "Stockholm" };
  }

  async function load() {
    try {
      const loc = await resolveLocation();
      place = loc.name;
      const url =
        `https://api.open-meteo.com/v1/forecast?latitude=${loc.lat}&longitude=${loc.lon}` +
        `&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m,is_day`;
      const r = await fetch(url);
      const j = await r.json();
      const c = j.current;
      temp = Math.round(c.temperature_2m);
      humidity = c.relative_humidity_2m;
      wind = Math.round(c.wind_speed_10m);
      code = c.weather_code;
      isDay = c.is_day === 1;
    } catch {
      // mock fallback
      place = "Stockholm";
      temp = 7;
      humidity = 72;
      wind = 12;
      code = 2;
    } finally {
      loading = false;
    }
  }

  onMount(load);
</script>

<div class="weather scene-{scene}" class:night={!isDay} class:loading>
  <div class="scene" aria-hidden="true">
    {#if scene === "clear"}
      <span class="sun"></span>
    {:else if scene === "storm"}
      <span class="cloud big"></span><span class="bolt"></span>
      <span class="drop d1"></span><span class="drop d2"></span>
    {:else if scene === "rain"}
      <span class="cloud big"></span>
      <span class="drop d1"></span><span class="drop d2"></span><span class="drop d3"></span><span class="drop d4"></span>
    {:else if scene === "snow"}
      <span class="cloud big"></span>
      <span class="flake f1"></span><span class="flake f2"></span><span class="flake f3"></span>
    {:else if scene === "fog"}
      <span class="fogbar b1"></span><span class="fogbar b2"></span><span class="fogbar b3"></span>
    {:else}
      <span class="sun small"></span><span class="cloud"></span>
    {/if}
  </div>

  <div class="info">
    <div class="temp">{temp ?? "–"}<span class="deg">°</span></div>
    <div class="label">{label}</div>
    <div class="place">{place}</div>
    <div class="stats">
      <span>💧 {humidity ?? "–"}%</span>
      <span>🌬 {wind ?? "–"} km/h</span>
    </div>
  </div>
</div>

<style>
  .weather {
    display: flex;
    align-items: center;
    gap: 1rem;
    height: 100%;
    min-height: 130px;
    padding: 0.5rem 0.5rem 0.5rem 0.25rem;
  }
  .scene {
    position: relative;
    width: 92px;
    height: 92px;
    flex-shrink: 0;
  }
  .info {
    min-width: 0;
  }
  .temp {
    font-size: 2.4rem;
    font-weight: 700;
    color: var(--text);
    line-height: 1;
  }
  .deg {
    color: var(--subtext);
  }
  .label {
    font-size: 0.85rem;
    color: var(--subtext);
    margin-top: 0.15rem;
  }
  .place {
    font-size: 0.72rem;
    color: var(--faint);
  }
  .stats {
    display: flex;
    gap: 0.85rem;
    margin-top: 0.5rem;
    font-size: 0.74rem;
    color: var(--subtext);
  }

  /* --- sun --- */
  .sun {
    position: absolute;
    inset: 26px;
    border-radius: 50%;
    background: radial-gradient(circle, #ffe08a, #fab040);
    box-shadow: 0 0 22px rgba(250, 176, 64, 0.55);
    animation: pulse 3s ease-in-out infinite;
  }
  .sun::before {
    content: "";
    position: absolute;
    inset: -12px;
    border-radius: 50%;
    background:
      conic-gradient(
        from 0deg,
        rgba(250, 176, 64, 0.5) 0 6%,
        transparent 6% 12.5%,
        rgba(250, 176, 64, 0.5) 12.5% 18.5%,
        transparent 18.5% 25%
      );
    mask: radial-gradient(circle, transparent 58%, #000 60%);
    -webkit-mask: radial-gradient(circle, transparent 58%, #000 60%);
    animation: spin 18s linear infinite;
  }
  .sun.small {
    inset: auto;
    top: 8px;
    left: 10px;
    width: 42px;
    height: 42px;
  }
  @keyframes pulse {
    50% {
      transform: scale(1.06);
    }
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* --- clouds --- */
  .cloud {
    position: absolute;
    bottom: 30px;
    right: 8px;
    width: 52px;
    height: 20px;
    background: #aeb6cc;
    border-radius: 14px;
    animation: drift 6s ease-in-out infinite;
  }
  .cloud::before,
  .cloud::after {
    content: "";
    position: absolute;
    background: #aeb6cc;
    border-radius: 50%;
  }
  .cloud::before {
    width: 24px;
    height: 24px;
    top: -12px;
    left: 8px;
  }
  .cloud::after {
    width: 18px;
    height: 18px;
    top: -9px;
    right: 10px;
  }
  .cloud.big {
    width: 64px;
    height: 24px;
    bottom: 42px;
    left: 14px;
    right: auto;
    background: #9aa3bd;
  }
  .cloud.big::before {
    background: #9aa3bd;
    width: 30px;
    height: 30px;
    top: -15px;
  }
  .cloud.big::after {
    background: #9aa3bd;
  }
  @keyframes drift {
    50% {
      transform: translateX(5px);
    }
  }

  /* --- rain / snow --- */
  .drop {
    position: absolute;
    width: 2px;
    height: 10px;
    border-radius: 2px;
    background: var(--accent-local);
    top: 56px;
    animation: fall 0.9s linear infinite;
  }
  .d1 {
    left: 26px;
    animation-delay: 0s;
  }
  .d2 {
    left: 40px;
    animation-delay: 0.3s;
  }
  .d3 {
    left: 54px;
    animation-delay: 0.55s;
  }
  .d4 {
    left: 68px;
    animation-delay: 0.15s;
  }
  @keyframes fall {
    0% {
      transform: translateY(0);
      opacity: 0;
    }
    25% {
      opacity: 1;
    }
    100% {
      transform: translateY(26px);
      opacity: 0;
    }
  }
  .flake {
    position: absolute;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #dfe6f5;
    top: 56px;
    animation: snowfall 2.4s linear infinite;
  }
  .f1 {
    left: 28px;
  }
  .f2 {
    left: 46px;
    animation-delay: 0.8s;
  }
  .f3 {
    left: 64px;
    animation-delay: 1.5s;
  }
  @keyframes snowfall {
    0% {
      transform: translateY(0);
      opacity: 0;
    }
    20% {
      opacity: 1;
    }
    100% {
      transform: translateY(30px) translateX(6px);
      opacity: 0;
    }
  }

  /* --- storm --- */
  .bolt {
    position: absolute;
    top: 50px;
    left: 42px;
    width: 0;
    height: 0;
    border-left: 6px solid transparent;
    border-right: 6px solid transparent;
    border-top: 16px solid #ffd24a;
    filter: drop-shadow(0 0 6px rgba(255, 210, 74, 0.8));
    animation: flash 2.6s steps(1) infinite;
  }
  @keyframes flash {
    0%,
    8%,
    100% {
      opacity: 0;
    }
    4% {
      opacity: 1;
    }
  }

  /* --- fog --- */
  .fogbar {
    position: absolute;
    left: 12px;
    width: 64px;
    height: 7px;
    border-radius: 7px;
    background: #9aa3bd;
    opacity: 0.55;
    animation: fogdrift 4s ease-in-out infinite;
  }
  .b1 {
    top: 32px;
  }
  .b2 {
    top: 46px;
    animation-delay: 0.7s;
  }
  .b3 {
    top: 60px;
    animation-delay: 1.4s;
  }
  @keyframes fogdrift {
    50% {
      transform: translateX(7px);
      opacity: 0.8;
    }
  }

  .loading {
    opacity: 0.6;
  }
</style>
