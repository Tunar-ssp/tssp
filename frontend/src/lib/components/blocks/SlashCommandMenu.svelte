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
    border-radius: 8px;
    box-shadow: 0 10px 38px 0 rgba(0, 0, 0, 0.12), 0 10px 20px 0 rgba(0, 0, 0, 0.1);
    z-index: 1000;
    max-height: 400px;
    overflow-y: auto;
    min-width: 280px;
    outline: none;
  }

  .no-results {
    padding: 16px;
    text-align: center;
    color: var(--muted);
    font-size: 14px;
  }

  .commands-list {
    display: flex;
    flex-direction: column;
    padding: 4px;
  }

  .command-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border: none;
    background: none;
    color: var(--text);
    cursor: pointer;
    text-align: left;
    transition: background-color 0.15s;
    border-radius: 6px;
  }

  .command-item:hover {
    background-color: var(--bg-secondary);
  }

  .command-item.selected {
    background-color: rgba(59, 130, 246, 0.1);
    color: var(--text);
  }

  .command-icon {
    flex-shrink: 0;
    font-size: 18px;
  }

  .command-info {
    flex: 1;
    min-width: 0;
  }

  .command-label {
    font-size: 14px;
    font-weight: 500;
  }

  .command-description {
    font-size: 12px;
    color: var(--muted);
    margin-top: 2px;
  }
</style>
