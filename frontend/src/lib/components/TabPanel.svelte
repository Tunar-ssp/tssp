<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as Icons from 'lucide-svelte';

  interface Tab {
    id: string;
    label: string;
    icon?: any;
    content?: Snippet;
  }

  interface Props {
    tabs: Tab[];
    activeTab?: string;
    onTabChange?: (tabId: string) => void;
    children?: Snippet;
  }

  let { tabs, activeTab = tabs[0]?.id, onTabChange, children }: Props = $props();

  function handleTabClick(tabId: string) {
    activeTab = tabId;
    onTabChange?.(tabId);
  }
</script>

<div class="tab-panel">
  <div class="tab-buttons">
    {#each tabs as tab (tab.id)}
      <button
        class="tab-btn"
        class:active={activeTab === tab.id}
        onclick={() => handleTabClick(tab.id)}
      >
        {#if tab.icon}
          <tab.icon size={16} />
        {/if}
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="tab-content">
    {#each tabs as tab (tab.id)}
      {#if activeTab === tab.id}
        <div class="tab-pane">
          {#if tab.content}
            {@render tab.content()}
          {:else if children}
            {@render children()}
          {/if}
        </div>
      {/if}
    {/each}
  </div>
</div>

<style>
  .tab-panel {
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
  }

  .tab-buttons {
    display: flex;
    gap: var(--s-2);
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-3) var(--s-4);
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-14);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    border-bottom: 2px solid transparent;
    white-space: nowrap;
  }

  .tab-btn:hover:not(.active) {
    color: var(--text);
    background: rgba(255, 255, 255, 0.05);
  }

  .tab-btn.active {
    color: var(--blue);
    border-bottom-color: var(--blue);
  }

  .tab-content {
    flex: 1;
  }

  .tab-pane {
    animation: fadeIn 150ms var(--ease-smooth);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
</style>
