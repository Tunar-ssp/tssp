<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Tab {
    id: string;
    label: string;
    isDirty?: boolean;
    language?: string;
  }

  interface $$Props {
    tabs?: Tab[];
    activeTabId?: string | null;
    onSelectTab?: (id: string) => void;
    onCloseTab?: (id: string) => void;
  }

  let {
    tabs = [],
    activeTabId = null,
    onSelectTab = () => {},
    onCloseTab = () => {},
  }: $$Props = $props();

  let tabsContainer: HTMLDivElement;

  function scrollToActiveTab() {
    if (tabsContainer && activeTabId) {
      const activeTabEl = tabsContainer.querySelector(`[data-tab-id="${activeTabId}"]`);
      if (activeTabEl) {
        activeTabEl.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
      }
    }
  }

  $effect(() => {
    scrollToActiveTab();
  });
</script>

<div class="tab-bar">
  <div class="tabs-scroll" bind:this={tabsContainer}>
    <div class="tabs-list">
      {#each tabs as tab (tab.id)}
        <button
          class="tab"
          class:active={activeTabId === tab.id}
          data-tab-id={tab.id}
          onclick={() => onSelectTab(tab.id)}
        >
          <span class="tab-label">{tab.label}</span>
          {#if tab.isDirty}
            <span class="tab-dirty" title="Unsaved changes">●</span>
          {/if}
          <div
            class="tab-close"
            role="button"
            tabindex="0"
            onclick={(e) => {
              e.stopPropagation();
              onCloseTab(tab.id);
            }}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.stopPropagation();
                onCloseTab(tab.id);
              }
            }}
            title="Close tab"
          >
            <Icons.X size={14} />
          </div>
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  .tab-bar {
    display: flex;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    height: 40px;
    flex-shrink: 0;
  }

  .tabs-scroll {
    flex: 1;
    overflow-x: auto;
    overflow-y: hidden;
    scroll-behavior: smooth;
  }

  .tabs-scroll::-webkit-scrollbar {
    height: 6px;
  }

  .tabs-scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  .tabs-scroll::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
  }

  .tabs-scroll::-webkit-scrollbar-thumb:hover {
    background: var(--muted);
  }

  .tabs-list {
    display: flex;
    gap: 0;
    padding: 0 4px;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    min-width: 120px;
    max-width: 200px;
    border: none;
    border-right: 1px solid var(--hairline);
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-12);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    white-space: nowrap;
  }

  .tab:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .tab.active {
    background: var(--bg);
    color: var(--text);
    border-bottom: 2px solid var(--blue);
  }

  .tab-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-dirty {
    font-size: 10px;
    color: var(--orange);
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .tab-close {
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    border-radius: var(--r-1);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
    flex-shrink: 0;
  }

  .tab-close:hover {
    background: var(--surface-2);
    color: var(--text);
  }
</style>
