<script lang="ts">
  import { onMount } from "svelte";
  import { Avatar } from "melt/builders";
  import { machineName } from "$lib/ipc";

  // Per-machine identity badge: initials of this machine's hostname. Shared by
  // the notes and todoz sidebar headers so both stay visually balanced.
  let host = $state("");
  const avatar = new Avatar({ src: () => "" }); // no image — initials fallback
  const initials = $derived(host.replace(/\.local$/, "").slice(0, 2).toUpperCase() || "··");
  onMount(async () => {
    host = await machineName();
  });
</script>

<span class="avatar" {...avatar.fallback} title={host || "this machine"}>{initials}</span>

<style>
  .avatar {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 0.66rem;
    font-weight: 700;
    color: #1a1626;
    background: linear-gradient(180deg, var(--accent), #b48ceb);
    flex-shrink: 0;
  }
</style>
