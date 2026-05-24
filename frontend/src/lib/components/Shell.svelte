<script lang="ts">
  import Banner from "./Banner.svelte";
  import CommandPalette from "./CommandPalette.svelte";
  import SideNav from "./SideNav.svelte";
  import TopBar from "./TopBar.svelte";
  import type { RouteId } from "../router";
  import { openCommandPalette } from "../stores/ui";

  export let route: RouteId;
</script>

<div class="app-shell">
  <SideNav {route} />
  <div class="app-main">
    <TopBar {route} onOpenPalette={openCommandPalette} />
    <Banner />
    <main class="page-shell">
      <slot />
    </main>
  </div>
  <CommandPalette />
</div>
