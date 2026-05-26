<script lang="ts">
  import { filterCommands, getCommandsWithCategories } from '$lib/blocks/slashCommands';
  import type { ExtendedSlashCommand } from '$lib/blocks/slashCommands';
  import type { SlashCommand } from '$lib/blocks/types';
  import { onMount } from 'svelte';

  interface Props {
    query?: string;
    position?: { top: number; left: number };
    onSelect?: (command: SlashCommand) => void;
    onClose?: () => void;
  }

  let { query = '', position = { top: 0, left: 0 }, onSelect, onClose }: Props = $props();

  let selectedIndex = $state(0);
  let filteredCommands = $state<ExtendedSlashCommand[]>([]);
  let displayItems = $state<Array<
    { type: 'header'; label: string } | { type: 'command'; command: ExtendedSlashCommand }
  >>([]);
  let selectableItems = $state<ExtendedSlashCommand[]>([]);
  let menuElement: HTMLDivElement;

  $effect(() => {
    filteredCommands = filterCommands(query);

    // If search query, show flat list. Otherwise show categorized
    if (query.trim()) {
      displayItems = filteredCommands.map(cmd => ({ type: 'command' as const, command: cmd }));
    } else {
      displayItems = getCommandsWithCategories();
    }

    // Track selectable items (skip headers)
    selectableItems = displayItems
      .filter(item => item.type === 'command')
      .map(item => (item as any).command);

    selectedIndex = 0;
  });

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, selectableItems.length - 1);
      scrollToSelected();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      scrollToSelected();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (selectableItems[selectedIndex]) {
        onSelect?.(selectableItems[selectedIndex]);
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose?.();
    }
  }

  function scrollToSelected() {
    const items = menuElement?.querySelectorAll('[role="option"]');
    if (items && items[selectedIndex]) {
      items[selectedIndex].scrollIntoView({ block: 'nearest' });
    }
  }

  function handleSelect(command: SlashCommand) {
    onSelect?.(command);
  }

  onMount(() => {
    menuElement?.focus();
  });
</script>

<div
  bind:this={menuElement}
  class="slash-menu"
  style:top="{position.top}px"
  style:left="{position.left}px"
  role="listbox"
  tabindex="0"
  onkeydown={handleKeyDown}
>
  {#if selectableItems.length === 0}
    <div class="no-results">
      <div class="no-results-icon">✨</div>
      <div>No commands found</div>
      <div class="no-results-hint">Try searching for headings, lists, or code</div>
    </div>
  {:else}
    <div class="commands-list">
      {#each displayItems as item, itemIdx}
        {#if item.type === 'header'}
          <div class="category-header">{item.label}</div>
        {:else}
          {@const command = item.command}
          {@const selectIdx = selectableItems.indexOf(command)}
          <button
            class="command-item"
            class:selected={selectedIndex === selectIdx}
            role="option"
            aria-selected={selectedIndex === selectIdx}
            onclick={() => handleSelect(command)}
            onmouseenter={() => (selectedIndex = selectIdx)}
          >
            <span class="command-icon">{command.icon}</span>
            <div class="command-info">
              <div class="command-header">
                <div class="command-label">{command.label}</div>
                {#if command.metadata?.shortcuts && command.metadata.shortcuts.length > 0}
                  <div class="command-shortcuts">
                    {#each command.metadata.shortcuts as shortcut}
                      <kbd>{shortcut}</kbd>
                    {/each}
                  </div>
                {/if}
              </div>
              <div class="command-description">{command.description}</div>
            </div>
          </button>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .slash-menu {
    position: fixed;
    background-color: var(--bg);
    border: 1px solid var(--border);
    border-radius: 10px;
    box-shadow: 0 10px 38px 0 rgba(0, 0, 0, 0.12), 0 10px 20px 0 rgba(0, 0, 0, 0.1);
    z-index: 1000;
    max-height: 480px;
    overflow-y: auto;
    min-width: 320px;
    outline: none;
    backdrop-filter: blur(12px);
  }

  .no-results {
    padding: 32px 24px;
    text-align: center;
    color: var(--muted);
    font-size: 14px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .no-results-icon {
    font-size: 28px;
    opacity: 0.6;
  }

  .no-results-hint {
    font-size: 12px;
    opacity: 0.7;
    margin-top: 4px;
  }

  .commands-list {
    display: flex;
    flex-direction: column;
    padding: 6px;
  }

  .category-header {
    padding: 10px 16px 6px 16px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.5px;
    text-transform: uppercase;
    color: var(--muted);
    margin-top: 4px;
    pointer-events: none;
  }

  .command-item {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 10px 12px;
    border: none;
    background: none;
    color: var(--text);
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
    border-radius: 8px;
    font-family: inherit;
  }

  .command-item:hover {
    background-color: rgba(59, 130, 246, 0.08);
    transform: translateX(2px);
  }

  .command-item.selected {
    background-color: rgba(59, 130, 246, 0.15);
    color: var(--text);
  }

  .command-icon {
    flex-shrink: 0;
    font-size: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
  }

  .command-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .command-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .command-label {
    font-size: 14px;
    font-weight: 500;
    flex: 1;
  }

  .command-shortcuts {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .command-shortcuts kbd {
    display: inline-block;
    padding: 2px 6px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 3px;
    font-size: 11px;
    font-family: 'Monaco', 'Courier New', monospace;
    color: var(--muted);
    white-space: nowrap;
  }

  .command-description {
    font-size: 12px;
    color: var(--muted);
  }

  /* Scrollbar styling */
  .slash-menu::-webkit-scrollbar {
    width: 6px;
  }

  .slash-menu::-webkit-scrollbar-track {
    background: transparent;
  }

  .slash-menu::-webkit-scrollbar-thumb {
    background-color: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
  }

  .slash-menu::-webkit-scrollbar-thumb:hover {
    background-color: rgba(0, 0, 0, 0.3);
  }
</style>
