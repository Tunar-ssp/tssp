<script lang="ts">
  import Banner from "./Banner.svelte";
  import CommandPalette from "./CommandPalette.svelte";
  import SideNav from "./SideNav.svelte";
  import TopBar from "./TopBar.svelte";
  import type { AppId } from "../router";
  import type { AppView } from "../stores/ui";
  import { openCommandPalette } from "../stores/ui";

  interface $$Props {
    currentView: AppId;
    title: string;
  }

  let { currentView, title }: $$Props = $props();
  let mobileOpen = $state(false);

  function mapAppIdToAppView(id: AppId): AppView {
    const map: Record<AppId, AppView> = {
      drive: 'drive',
      knowledge: 'notes',
      workspace: 'workspace',
      operations: 'admin',
    };
    return map[id] || 'drive';
  }

  function closeMobile() {
    mobileOpen = false;
  }
</script>

<div class="app-shell">
  <SideNav app={currentView} {mobileOpen} onCloseMobile={closeMobile} />
  {#if mobileOpen}
    <button
      type="button"
      class="sidebar-backdrop"
      aria-label="Close menu"
      onclick={closeMobile}
    ></button>
  {/if}
  <div class="app-main">
    <TopBar
      currentView={mapAppIdToAppView(currentView)}
      {title}
      onHome={() => openCommandPalette()}
      onCommandPalette={openCommandPalette}
      onUpload={() => {}}
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
