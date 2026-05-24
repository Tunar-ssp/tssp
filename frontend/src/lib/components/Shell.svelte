<script lang="ts">
  import Banner from "./Banner.svelte";
  import CommandPalette from "./CommandPalette.svelte";
  import SideNav from "./SideNav.svelte";
  import TopBar from "./TopBar.svelte";
  import type { AppId } from "../router";
  import { openCommandPalette } from "../stores/ui";

  export let app: AppId;

  let mobileOpen = false;

  function closeMobile() {
    mobileOpen = false;
  }
</script>

<div class="app-shell">
  <SideNav {app} {mobileOpen} onCloseMobile={closeMobile} />
  {#if mobileOpen}
    <button
      type="button"
      class="sidebar-backdrop"
      aria-label="Close menu"
      on:click={closeMobile}
    ></button>
  {/if}
  <div class="app-main">
    <TopBar
      {app}
      onOpenPalette={openCommandPalette}
      onToggleMenu={() => (mobileOpen = !mobileOpen)}
    />
    <Banner />
    <main class="page-shell">
      <slot />
    </main>
  </div>
</div>
<CommandPalette />

<style>
  .sidebar-backdrop {
    position: fixed;
    inset: 0;
    z-index: 30;
    border: none;
    background: rgba(0, 0, 0, 0.55);
  }
</style>
