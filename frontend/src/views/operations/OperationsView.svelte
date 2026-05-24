<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import SystemStatus from '$lib/components/SystemStatus.svelte';
  import SafeConsole from '$lib/components/SafeConsole.svelte';

  let activeTab = 'status';

  const tabs = [
    { id: 'status', label: 'System Status', icon: Icons.Server },
    { id: 'console', label: 'Diagnostics', icon: Icons.Terminal },
  ];
</script>

<div class="ops-view">
  <div class="ops-header">
    <div>
      <h2>Operations Console</h2>
      <p class="subtitle">Admin, diagnostics, and system control</p>
    </div>
  </div>

  <div class="tabs-bar">
    {#each tabs as tab (tab.id)}
      <button
        class="tab-btn"
        class:active={activeTab === tab.id}
        on:click={() => (activeTab = tab.id)}
      >
        <svelte:component this={tab.icon} size={16} />
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="content">
    {#if activeTab === 'status'}
      <SystemStatus />
    {:else if activeTab === 'console'}
      <SafeConsole />
    {/if}
  </div>
</div>

<style>
  .ops-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg);
  }

  .ops-header {
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .ops-header h2 {
    margin: 0;
    font-size: var(--fs-24);
    color: var(--text);
  }

  .subtitle {
    margin: 4px 0 0;
    font-size: var(--fs-13);
    color: var(--text-2);
  }

  .tabs-bar {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    padding: 0 24px;
    overflow-x: auto;
  }

  .tab-btn {
    padding: 12px 16px;
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-13);
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    white-space: nowrap;
    border-bottom: 2px solid transparent;
    transition: all 0.15s;
  }

  .tab-btn:hover {
    color: var(--text);
  }

  .tab-btn.active {
    color: var(--blue);
    border-bottom-color: var(--blue);
  }

  .content {
    flex: 1;
    overflow: hidden;
  }
</style>
