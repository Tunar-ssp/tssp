<script lang="ts">
  import type { AppId } from "../router";
  import { authState } from "../stores/auth";

  export let app: AppId;
  export let onOpenPalette: (seed?: string) => void;
  export let onToggleMenu: () => void;

  const titles: Record<AppId, string> = {
    drive: "Cloud Drive",
    knowledge: "Knowledge",
    workspace: "Workspace",
    operations: "Operations",
  };
</script>

<header class="topbar">
  <button type="button" class="topbar-menu btn btn-ghost" on:click={onToggleMenu} aria-label="Menu">
    ☰
  </button>
  <div class="topbar-title">{titles[app]}</div>

  <button class="topbar-search" type="button" on:click={() => onOpenPalette("")}>
    <span>Search files, notes, commands…</span>
    <kbd>Ctrl K</kbd>
  </button>

  <div class="topbar-actions">
    <div class="session-pill">
      {#if $authState.loading}
        …
      {:else if $authState.user}
        {$authState.user.name} · {$authState.user.role}
      {:else}
        Local access
      {/if}
    </div>
  </div>
</header>
