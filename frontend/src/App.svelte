<script lang="ts">
  import { onMount } from 'svelte';
  import { probeAuth } from '$lib/stores/auth';
  import {
    currentView,
    banner,
    commandPaletteOpen,
    settingsTrayOpen,
    shortcutsOverlayOpen,
    toggleCommandPalette,
    toggleSettingsTray,
    toggleShortcutsOverlay,
  } from '$lib/stores/ui';
  import * as Icons from 'lucide-svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import Dock from '$lib/components/Dock.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import SettingsTray from '$lib/components/SettingsTray.svelte';
  import ShortcutsOverlay from '$lib/components/ShortcutsOverlay.svelte';
  import NotificationCenter from '$lib/components/NotificationCenter.svelte';
  import '$lib/responsive.css';
  import HomeView from './views/home/HomeLauncher.svelte';
  import DriveView from './views/drive/DriveView.svelte';
  import NotesView from './views/notes/NotesView.svelte';
  import WorkspaceView from './views/workspace/WorkspaceView.svelte';
  import OperationsView from './views/operations/OperationsView.svelte';

  onMount(() => {
    probeAuth();

    const handleKeydown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'k') {
        e.preventDefault();
        toggleCommandPalette();
      }
      if (e.ctrlKey && e.key === '?') {
        e.preventDefault();
        toggleShortcutsOverlay();
      }
    };

    document.addEventListener('keydown', handleKeydown);
    return () => document.removeEventListener('keydown', handleKeydown);
  });

  const viewMap = {
    home: HomeView,
    drive: DriveView,
    notes: NotesView,
    workspace: WorkspaceView,
    operations: OperationsView,
  };

  $: CurrentView = viewMap[$currentView as keyof typeof viewMap] || HomeView;

  const dockItems = [
    { id: 'drive', label: 'Cloud', icon: Icons.Cloud, accent: '#6ea8ff', action: () => currentView.set('drive') },
    { id: 'notes', label: 'Notes', icon: Icons.BookOpen, accent: '#fbbf24', action: () => currentView.set('notes') },
    { id: 'workspace', label: 'Workspace', icon: Icons.Code2, accent: '#5be39a', action: () => currentView.set('workspace') },
    { id: 'operations', label: 'Admin', icon: Icons.Shield, accent: '#a394ff', action: () => currentView.set('operations') },
  ];

  const commands = [
    {
      id: 'drive',
      label: 'Go to Drive',
      description: 'Access your cloud drive',
      icon: Icons.HardDrive,
      action: () => currentView.set('drive'),
      shortcut: 'Ctrl+D',
    },
    {
      id: 'notes',
      label: 'Go to Notes',
      description: 'Access your notes',
      icon: Icons.FileText,
      action: () => currentView.set('notes'),
      shortcut: 'Ctrl+N',
    },
    {
      id: 'workspace',
      label: 'Go to Workspace',
      description: 'Access your workspace',
      icon: Icons.Code2,
      action: () => currentView.set('workspace'),
      shortcut: 'Ctrl+E',
    },
    {
      id: 'operations',
      label: 'Go to Operations',
      description: 'System operations and admin',
      icon: Icons.Settings,
      action: () => currentView.set('operations'),
      shortcut: 'Ctrl+O',
    },
    {
      id: 'settings',
      label: 'Settings',
      description: 'Open settings',
      icon: Icons.Settings,
      action: () => toggleSettingsTray(),
      shortcut: 'Ctrl+,',
    },
  ];
</script>

<div class="app">
  <TopBar
    currentView={$currentView}
    onCommandPalette={toggleCommandPalette}
    onSettings={toggleSettingsTray}
    onProfile={() => null}
  />

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

  <Dock items={dockItems} activeId={$currentView} />
  <CommandPalette
    {commands}
    isOpen={$commandPaletteOpen}
    onClose={() => commandPaletteOpen.set(false)}
  />
  <SettingsTray
    isOpen={$settingsTrayOpen}
    onClose={() => settingsTrayOpen.set(false)}
  />
  <ShortcutsOverlay
    isOpen={$shortcutsOverlayOpen}
    onClose={() => shortcutsOverlayOpen.set(false)}
  />
  <NotificationCenter />
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
