<script lang="ts">
  import type { RouteId } from "../router";
  import { authState } from "../stores/auth";

  export let route: RouteId;
  export let onOpenPalette: (seed?: string) => void;

  const titleMap: Record<RouteId, string> = {
    drive: "Cloud Drive",
    images: "Cloud Drive",
    videos: "Cloud Drive",
    documents: "Cloud Drive",
    sharing: "Sharing Center",
    notes: "Notes",
    workspace: "Workspace",
    operations: "Operations",
    search: "Search",
  };
</script>

<header class="topbar">
  <div class="topbar-title">
    <span class="brand-mark">TSSP</span>
    <div>
      <strong>{titleMap[route]}</strong>
      <span>Local-first cloud system</span>
    </div>
  </div>

  <button class="topbar-search" type="button" on:click={() => onOpenPalette("")}>
    <span>Search files, notes, workspaces, commands</span>
    <kbd>Ctrl K</kbd>
  </button>

  <div class="topbar-actions">
    <button class="btn btn-secondary" type="button">Upload</button>
    <button class="btn btn-secondary" type="button">Refresh</button>
    <div class="session-pill">
      {#if $authState.loading}
        Loading session
      {:else if $authState.user}
        {$authState.user.name} · {$authState.user.role}
      {:else}
        Open local
      {/if}
    </div>
  </div>
</header>
