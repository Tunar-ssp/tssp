<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Tab {
    id: string;
    label: string;
    isDirty?: boolean;
    language?: string;
  }

  interface $$Props {
    tabs: Tab[];
    activeTabId: string | null;
    onTabSelect?: (id: string) => void;
    onTabClose?: (id: string) => void;
  }

  let { tabs, activeTabId, onTabSelect, onTabClose }: $$Props = $props();
</script>

<div class="workspace-tabs">
  <div class="tabs-list">
    {#each tabs as tab (tab.id)}
      <div class="tab-item" class:active={activeTabId === tab.id}>
        <button
          type="button"
          class="tab-button"
          onclick={() => onTabSelect?.(tab.id)}
        >
          <span class="tab-label">
            {#if tab.isDirty}
              <span class="dirty-indicator" title="Unsaved changes"></span>
            {/if}
            {tab.label}
          </span>
        </button>
        <button
          type="button"
          class="tab-close"
          onclick={(e) => {
            e.stopPropagation();
            onTabClose?.(tab.id);
          }}
          title="Close tab"
        >
          <Icons.X size={14} />
        </button>
      </div>
    {/each}
  </div>
</div>

<style>
  .workspace-tabs {
    display: flex;
    align-items: center;
    height: 44px;
    padding: 0 12px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    overflow-x: auto;
    overflow-y: hidden;
  }

  .tabs-list {
    display: flex;
    gap: 4px;
    height: 100%;
  }

  .tab-item {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 32px;
    padding: 0 12px;
    border: 1px solid transparent;
    border-radius: 6px;
    background: transparent;
    color: var(--text-2);
    font-size: 13px;
    white-space: nowrap;
    transition: all 0.2s;
  }

  .tab-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .tab-item.active {
    background: var(--surface-2);
    border-color: var(--blue);
    color: var(--text);
  }

  .tab-button {
    flex: 1;
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: inherit;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .tab-button:focus {
    outline: none;
  }

  .tab-label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .dirty-indicator {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--warning);
  }

  .tab-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.2s;
  }

  .tab-close:hover {
    background: rgba(255, 59, 48, 0.14);
    color: var(--danger);
  }
</style>
