<script lang="ts">
  import { get } from 'svelte/store';
  import { api } from '$lib/api';
  import { uploadFiles } from '$lib/services/driveService';
  import { registerGlobalKeyboardHandlers } from '$lib/services/keyboardService';
  import { isAdmin, isLoading as authLoading, probeAuth, user } from '$lib/stores/auth';
  import {
    banner,
    commandPaletteOpen,
    currentView,
    dockMode,
    navigateTo,
    preferences,
    settingsTrayOpen,
    shortcutsOverlayOpen,
    type AppView,
    toggleSettingsTray,
    toggleShortcutsOverlay,
    toggleCommandPalette,
  } from '$lib/stores/ui';
  import * as Icons from 'lucide-svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import Dock from '$lib/components/Dock.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import SettingsTray from '$lib/components/SettingsTray.svelte';
  import ShortcutsOverlay from '$lib/components/ShortcutsOverlay.svelte';
  import NotificationCenter from '$lib/components/NotificationCenter.svelte';
  import '$lib/responsive.css';
  import SignInView from './views/auth/SignInView.svelte';
  import HomeView from './views/home/HomeLauncher.svelte';
  import DriveView from './views/drive/DriveView.svelte';
  import NotesView from './views/notes/NotesSurface.svelte';
  import WorkspaceView from './views/workspace/WorkspaceSurface.svelte';
  import OperationsView from './views/operations/OperationsView.svelte';
  import UploadQueue from '$lib/components/UploadQueue.svelte';

  let uploadInput: HTMLInputElement | null = $state(null);

  $effect(() => {
    void probeAuth();
  });

  $effect(() => {
    const cleanupKeyboard = registerGlobalKeyboardHandlers();

    const handleUploadRequest = () => {
      uploadInput?.click();
    };

    document.addEventListener('tssp:request-upload', handleUploadRequest as EventListener);

    return () => {
      cleanupKeyboard();
      document.removeEventListener('tssp:request-upload', handleUploadRequest as EventListener);
    };
  });

  const viewMap = {
    home: HomeView,
    drive: DriveView,
    notes: NotesView,
    workspace: WorkspaceView,
    admin: OperationsView,
  };

  let CurrentView = $derived(viewMap[$currentView as keyof typeof viewMap] || HomeView);

  const appMeta: Record<AppView, { title: string; icon: any; accent: string; crumbs: string[] }> = {
    home: { title: 'Launcher', icon: Icons.Home, accent: '#7c8190', crumbs: ['Launcher'] },
    drive: { title: 'Cloud Drive', icon: Icons.Cloud, accent: '#6ea8ff', crumbs: ['Drive'] },
    notes: { title: 'Notes', icon: Icons.BookText, accent: '#5be39a', crumbs: ['Notes'] },
    workspace: { title: 'Workspace', icon: Icons.Code2, accent: '#ff8a3d', crumbs: ['Workspace'] },
    admin: { title: 'Admin', icon: Icons.Shield, accent: '#a394ff', crumbs: ['Admin'] },
  };

  $effect(() => {
    if (!$isAdmin && $currentView === 'admin') {
      navigateTo('home');
    }
  });

  let availableAppIds = $derived(
    ($preferences.dockOrder || []).filter((view) => $isAdmin || view !== 'admin')
  );

  let dockItems = $derived([
    {
      id: 'home',
      label: 'Home',
      icon: Icons.Power,
      accent: '#5be39a',
      action: () => navigateTo('home'),
    },
    ...availableAppIds.map((view) => ({
      id: view,
      label: appMeta[view].title,
      icon: appMeta[view].icon,
      accent: appMeta[view].accent,
      action: () => navigateTo(view),
    })),
  ]);

  const commands = [
    {
      id: 'command-settings',
      label: 'Open settings',
      description: 'Adjust dock, theme, density, and defaults',
      icon: Icons.Settings2,
      action: () => toggleSettingsTray(),
      shortcut: '⌘,',
    },
  ];

  async function handleLogout() {
    try {
      await api.logout();
    } catch {
      // Ignore logout transport issues; local auth state still needs to clear.
    }
    user.set(null);
    navigateTo('home');
  }

  async function handleUploadSelected(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    if (!input.files?.length) return;
    await uploadFiles(input.files);
    input.value = '';
    navigateTo('drive');
  }
</script>

<input bind:this={uploadInput} type="file" multiple hidden onchange={handleUploadSelected} />

{#if $authLoading}
  <div class="auth-loading">
    <div class="loading-card">
      <div class="loading-brand">tssp</div>
      <h1>Preparing local cloud</h1>
      <p>Checking session state and restoring the shell.</p>
    </div>
  </div>
{:else if !$user}
  <SignInView />
{:else}
    <div class="app" class:home-active={$currentView === 'home'}>
    {#if !['notes', 'workspace'].includes($currentView)}
      <TopBar
        currentView={$currentView}
        title={appMeta[$currentView]?.title || 'TSSP'}
        crumbs={appMeta[$currentView]?.crumbs || ['TSSP']}
        userName={$user.name}
        role={$user.role}
        dockMode={$dockMode}
        onHome={() => navigateTo('home')}
        onCommandPalette={toggleCommandPalette}
        onUpload={() => uploadInput?.click()}
        onSettings={toggleSettingsTray}
        onLogout={handleLogout}
      />
    {/if}

    <div class="shell" class:no-topbar={['notes', 'workspace'].includes($currentView)}>
      <main class="main">
        {#if $banner}
          <div class="banner {$banner.type}">
            {$banner.message}
          </div>
        {/if}
        <CurrentView />
      </main>
    </div>

    <Dock items={dockItems} activeId={$currentView} mode={['notes', 'workspace'].includes($currentView) ? 'autohide' : $dockMode} />
    <UploadQueue />
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
{/if}

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
    background:
      radial-gradient(circle at 14% 0%, rgba(91, 227, 154, 0.08), transparent 28%),
      radial-gradient(circle at 86% 0%, rgba(255, 95, 162, 0.06), transparent 24%),
      linear-gradient(180deg, #090b10 0%, #06070a 100%);
    color: var(--text);
    font-family: var(--ff-sans);
  }

  .app.home-active {
    background:
      radial-gradient(circle at 14% 0%, rgba(91, 227, 154, 0.12), transparent 36%),
      radial-gradient(circle at 86% 0%, rgba(255, 95, 162, 0.08), transparent 28%),
      linear-gradient(180deg, #090b10 0%, #06070a 100%);
  }

  .shell {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .shell.no-topbar {
    height: 100vh;
  }

  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  .banner {
    margin: 0 24px;
    padding: 12px 16px;
    border: 1px solid;
    border-radius: 18px;
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

  .auth-loading {
    width: 100vw;
    height: 100vh;
    display: grid;
    place-items: center;
    background:
      radial-gradient(circle at 14% 0%, rgba(91, 227, 154, 0.12), transparent 32%),
      radial-gradient(circle at 86% 0%, rgba(255, 95, 162, 0.08), transparent 28%),
      linear-gradient(180deg, #090b10 0%, #06070a 100%);
  }

  .loading-card {
    width: min(560px, calc(100vw - 32px));
    padding: 32px;
    border-radius: 28px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.03);
    box-shadow: var(--shadow-modal);
  }

  .loading-brand {
    font-family: var(--ff-hand);
    font-size: 54px;
    line-height: 1;
    margin-bottom: 24px;
  }

  .loading-card h1 {
    margin: 0 0 10px;
    font-size: clamp(28px, 4vw, 54px);
  }

  .loading-card p {
    margin: 0;
    color: var(--text-2);
    font-size: 16px;
  }
</style>
