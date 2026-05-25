<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { currentView } from '$lib/stores/ui';

  interface $$Props {
    onCommandPalette?: () => void;
    onSettings?: () => void;
    onProfile?: () => void;
    class?: string;
  }

  let {
    onCommandPalette,
    onSettings,
    onProfile,
    class: className,
  } = $props<$$Props>();

  const nav = [
    { label: 'Drive', view: 'drive', icon: Icons.HardDrive },
    { label: 'Notes', view: 'notes', icon: Icons.FileText },
    { label: 'Workspace', view: 'workspace', icon: Icons.Code2 },
    { label: 'Operations', view: 'operations', icon: Icons.Settings },
  ];
</script>

<header class="topbar {className || ''}">
  <div class="topbar-content">
    <button type="button" class="topbar-logo" onclick={() => currentView.set('home')} aria-label="Open TSSP home">
      <Icons.Zap size={20} />
      <span>TSSP</span>
    </button>

    <nav class="topbar-nav">
      {#each nav as item (item.view)}
        {@const Icon = item.icon}
        <button
          type="button"
          class="nav-item"
          class:active={$currentView === item.view}
          onclick={() => currentView.set(item.view)}
        >
          <Icon size={16} />
          <span>{item.label}</span>
        </button>
      {/each}
    </nav>

    <div class="topbar-actions">
      <button
        type="button"
        class="topbar-btn"
        onclick={onCommandPalette}
        title="Command Palette (Ctrl+K)"
      >
        <Icons.Search size={16} />
        <span class="hidden-xs">Search</span>
      </button>

      <button
        type="button"
        class="topbar-btn"
        onclick={onSettings}
        title="Settings"
      >
        <Icons.Settings size={16} />
        <span class="hidden-xs">Settings</span>
      </button>

      <button
        type="button"
        class="topbar-btn profile-btn"
        onclick={onProfile}
        title="Profile"
      >
        <Icons.User size={16} />
        <span class="hidden-xs">Profile</span>
      </button>
    </div>
  </div>
</header>

<style>
  .topbar {
    position: sticky;
    top: 0;
    z-index: 100;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    backdrop-filter: blur(10px);
    background-color: rgba(20, 22, 29, 0.8);
  }

  .topbar-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 56px;
    padding: 0 var(--s-6);
    gap: var(--s-6);
    max-width: 100%;
  }

  .topbar-logo {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2);
    border: none;
    background: transparent;
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text);
    flex-shrink: 0;
    cursor: pointer;
    border-radius: var(--r-2);
    transition: background var(--duration-quick) var(--ease-smooth);
  }

  .topbar-logo:hover {
    background: var(--surface-2);
  }

  .topbar-nav {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    flex: 1;
    margin: 0 var(--s-6);
    padding: 0;
    list-style: none;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-4);
    font-size: var(--fs-13);
    color: var(--text-2);
    border: none;
    background: transparent;
    border-radius: var(--r-2);
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    white-space: nowrap;
    font-family: var(--ff-sans);
    font-weight: 500;
  }

  .nav-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .nav-item.active {
    background: var(--blue-subtle);
    color: var(--blue);
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    flex-shrink: 0;
  }

  .topbar-btn {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-13);
    cursor: pointer;
    border-radius: var(--r-2);
    transition: all var(--duration-quick) var(--ease-smooth);
    white-space: nowrap;
  }

  .topbar-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .topbar-btn:focus-visible {
    outline: 2px solid var(--blue);
    outline-offset: 2px;
  }

  .profile-btn {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-1) var(--s-3);
    background: var(--surface-2);
  }

  .profile-btn:hover {
    background: var(--surface-3);
  }

  .hidden-xs {
    display: none;
  }

  @media (min-width: 640px) {
    .hidden-xs {
      display: inline;
    }
  }
</style>
