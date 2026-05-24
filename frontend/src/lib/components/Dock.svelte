<script lang="ts">
  import { currentView } from '$lib/stores/ui';
  import { Cloud, FileText, SquareTerminal, Shield, Home } from 'lucide-svelte';

  const apps = [
    { id: 'home', label: 'Launcher', icon: Home },
    { id: 'drive', label: 'Cloud Drive', icon: Cloud },
    { id: 'notes', label: 'Notes', icon: FileText },
    { id: 'workspace', label: 'Workspace', icon: SquareTerminal },
    { id: 'admin', label: 'Admin', icon: Shield },
  ];

  function selectApp(id: string) {
    currentView.set(id);
  }
</script>

<div class="dock">
  <div class="dock-inner">
    {#each apps as app (app.id)}
      <button
        class="dock-item {$currentView === app.id ? 'active' : ''}"
        on:click={() => selectApp(app.id)}
        title={app.label}
      >
        <div class="dock-icon">
          <svelte:component this={app.icon} size={28} />
        </div>
        {#if $currentView === app.id}
          <div class="dock-label">{app.label}</div>
        {/if}
        <div class="dock-indicator {$currentView === app.id ? 'active' : ''}"></div>
      </button>
    {/each}
  </div>
</div>

<style>
  .dock {
    position: fixed;
    bottom: 18px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 30;
  }

  .dock-inner {
    display: flex;
    align-items: flex-end;
    gap: 12px;
    padding: 10px;
    border-radius: 22px;
    background: var(--dock-glass);
    border: 1px solid var(--dock-edge);
    box-shadow: var(--shadow-dock);
    backdrop-filter: blur(28px) saturate(1.4);
  }

  .dock-item {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    cursor: pointer;
    transition: all 0.2s;
    padding: 0;
  }

  .dock-item.active .dock-icon {
    transform: translateY(-6px) scale(1.06);
    filter: none;
  }

  .dock-item:not(.active) .dock-icon {
    filter: saturate(0.92);
  }

  .dock-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text);
    transition: all 0.2s;
  }

  .dock-label {
    position: absolute;
    top: -28px;
    padding: 3px 8px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.08);
    font-size: 10px;
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
  }

  .dock-indicator {
    width: 4px;
    height: 4px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.22);
    transition: all 0.2s;
  }

  .dock-indicator.active {
    width: 16px;
    background: var(--text);
  }
</style>
