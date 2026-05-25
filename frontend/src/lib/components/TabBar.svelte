<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Tab {
    id: string;
    name: string;
    isDirty?: boolean;
    icon?: any;
  }

  interface $$Props {
    tabs?: Tab[];
    activeTabId?: string;
    onSelectTab?: (tabId: string) => void;
    onCloseTab?: (tabId: string) => void;
    class?: string;
  }

  let {
    tabs = [],
    activeTabId,
    onSelectTab,
    onCloseTab,
    class: className,
  } = $props<$$Props>();

  let scrollContainer: HTMLElement;

  function scrollToTab(tabId: string) {
    const tab = scrollContainer?.querySelector(`[data-tab-id="${tabId}"]`) as HTMLElement;
    if (tab && scrollContainer) {
      tab.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
    }
  }

  $effect(() => {
    if (activeTabId) {
      scrollToTab(activeTabId);
    }
  });
</script>

<div class="tab-bar {className || ''}">
  <div class="tabs-container" bind:this={scrollContainer}>
    {#each tabs as tab (tab.id)}
      <button
        class="tab"
        class:active={activeTabId === tab.id}
        data-tab-id={tab.id}
        on:click={() => onSelectTab?.(tab.id)}
      >
        {#if tab.icon}
          <svelte:component this={tab.icon} size={14} />
        {/if}
        <span class="tab-name">{tab.name}</span>
        {#if tab.isDirty}
          <span class="dirty-indicator" title="Unsaved changes">●</span>
        {/if}
        <button
          class="close-btn"
          on:click|stopPropagation={() => onCloseTab?.(tab.id)}
          title="Close"
        >
          <Icons.X size={12} />
        </button>
      </button>
    {/each}
  </div>

  {#if tabs.length > 0}
    <button class="tab-menu" title="Tab options">
      <Icons.ChevronDown size={14} />
    </button>
  {/if}
</div>

<style>
  .tab-bar {
    display: flex;
    align-items: center;
    height: 40px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    gap: var(--s-1);
    flex-shrink: 0;
  }

  .tabs-container {
    flex: 1;
    display: flex;
    gap: 0;
    overflow-x: auto;
    scroll-behavior: smooth;
    scrollbar-width: thin;
    scrollbar-color: var(--surface-2) transparent;
  }

  .tabs-container::-webkit-scrollbar {
    height: 4px;
  }

  .tabs-container::-webkit-scrollbar-track {
    background: transparent;
  }

  .tabs-container::-webkit-scrollbar-thumb {
    background: var(--surface-2);
    border-radius: 2px;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 0 var(--s-3);
    height: 100%;
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-12);
    cursor: pointer;
    white-space: nowrap;
    border-right: 1px solid var(--hairline);
    transition: all var(--duration-quick) var(--ease-smooth);
    font-family: var(--ff-sans);
  }

  .tab:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .tab.active {
    background: var(--surface-2);
    color: var(--blue);
    border-bottom: 2px solid var(--blue);
    padding-bottom: -2px;
  }

  .tab-name {
    flex: 1;
    min-width: 100px;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dirty-indicator {
    font-size: 8px;
    color: var(--orange);
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .close-btn {
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: inherit;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-1);
    opacity: 0;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .tab:hover .close-btn {
    opacity: 1;
  }

  .close-btn:hover {
    background: rgba(255, 107, 107, 0.2);
    color: var(--danger);
  }

  .tab-menu {
    width: 32px;
    height: 100%;
    padding: 0 var(--s-2);
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
    border-left: 1px solid var(--border);
  }

  .tab-menu:hover {
    background: var(--surface-2);
    color: var(--text);
  }
</style>
