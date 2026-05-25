<script lang="ts">
  import { onMount } from 'svelte';
  import { probeAuth } from '$lib/stores/auth';
  import { currentView, banner } from '$lib/stores/ui';
  import TopBar from '$lib/components/TopBar.svelte';
  import Dock from '$lib/components/Dock.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import HomeView from './views/home/HomeView.svelte';
  import DriveView from './views/drive/DriveView.svelte';
  import NotesView from './views/notes/NotesView.svelte';
  import WorkspaceView from './views/workspace/WorkspaceView.svelte';
  import OperationsView from './views/operations/OperationsView.svelte';

  onMount(() => {
    probeAuth();
  });

  const viewMap = {
    home: HomeView,
    drive: DriveView,
    notes: NotesView,
    workspace: WorkspaceView,
    operations: OperationsView,
  };

  $: CurrentView = viewMap[$currentView as keyof typeof viewMap] || HomeView;
</script>

<div class="app">
  <TopBar context={$currentView} />

  <div class="shell">
    <main class="main">
      {#if $banner}
        <div class="banner {$banner.type}">
          {$banner.message}
        </div>
      {/if}
      <svelte:component this={CurrentView} />
    </main>
  </div>

  <Dock />
  <CommandPalette />
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  .app {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    color: var(--text);
    font-family: var(--ff-sans);
  }

  .shell {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .banner {
    padding: 12px 16px;
    border-bottom: 1px solid;
    font-size: var(--fs-13);
  }

  .banner.success {
    background: rgba(52, 211, 153, 0.1);
    border-color: rgba(52, 211, 153, 0.25);
    color: var(--success);
  }

  .banner.error {
    background: rgba(255, 107, 107, 0.1);
    border-color: rgba(255, 107, 107, 0.25);
    color: var(--danger);
  }

  .banner.info {
    background: rgba(110, 168, 255, 0.1);
    border-color: rgba(110, 168, 255, 0.25);
    color: var(--blue);
  }
</style>
