<script lang="ts">
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';

  interface Command {
    id: string;
    label: string;
    description?: string;
    icon?: any;
    action: () => void;
    shortcut?: string;
  }

  interface $$Props {
    commands?: Command[];
    isOpen?: boolean;
    onClose?: () => void;
    class?: string;
  }

  let {
    commands = [],
    isOpen = false,
    onClose,
    class: className,
  } = $props<$$Props>();

  let searchQuery = $state('');
  let selectedIndex = $state(0);
  let inputRef: HTMLInputElement | null = $state(null);

  $effect(() => {
    if (isOpen && inputRef) {
      inputRef.focus();
      searchQuery = '';
      selectedIndex = 0;
    }
  });

  $effect(() => {
    const handleKeydown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'k') {
        e.preventDefault();
        if (isOpen && onClose) {
          onClose();
        }
      }
      if (!isOpen) return;

      if (e.key === 'Escape' && onClose) {
        onClose();
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, filteredCommands.length - 1);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          filteredCommands[selectedIndex].action();
          if (onClose) onClose();
        }
      }
    };

    document.addEventListener('keydown', handleKeydown);
    return () => document.removeEventListener('keydown', handleKeydown);
  });

  let filteredCommands = $derived(
    searchQuery
      ? commands.filter(
          (cmd) =>
            cmd.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
            cmd.description?.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : commands
  );

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }
</script>

{#if isOpen}
  <div
    class="palette-backdrop"
    role="presentation"
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={(e) => {
      if (e.key === 'Escape' && onClose) onClose();
    }}
  >
    <div class="palette {className || ''}">
      <div class="palette-search">
        <Icons.Search size={20} />
        <input
          bind:this={inputRef}
          type="text"
          placeholder="Search commands..."
          bind:value={searchQuery}
          class="palette-input"
        />
      </div>

      <div class="palette-results">
        {#if filteredCommands.length === 0}
          <div class="palette-empty">
            <Icons.AlertCircle size={32} />
            <p>No commands found</p>
          </div>
        {:else}
          {#each filteredCommands as cmd, idx (cmd.id)}
            {@const Icon = cmd.icon}
            <button
              type="button"
              class="palette-item"
              class:selected={idx === selectedIndex}
              onclick={() => {
                cmd.action();
                if (onClose) onClose();
              }}
            >
              <div class="item-content">
                {#if Icon}
                  <div class="item-icon">
                    <Icon size={16} />
                  </div>
                {/if}
                <div class="item-text">
                  <div class="item-label">{cmd.label}</div>
                  {#if cmd.description}
                    <div class="item-description">{cmd.description}</div>
                  {/if}
                </div>
              </div>
              {#if cmd.shortcut}
                <div class="item-shortcut">{cmd.shortcut}</div>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <div class="palette-footer">
        <span class="footer-hint">↑ ↓ to select • Enter to run • Esc to close</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .palette-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    z-index: 2000;
    padding-top: 80px;
    animation: fadeIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .palette {
    width: 90%;
    max-width: 600px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    max-height: 400px;
    animation: paletteSlideIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes paletteSlideIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .palette-search {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    color: var(--muted);
  }

  .palette-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-14);
    outline: none;
    font-family: var(--ff-sans);
  }

  .palette-input::placeholder {
    color: var(--muted);
  }

  .palette-results {
    flex: 1;
    overflow-y: auto;
  }

  .palette-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    padding: var(--s-8);
    color: var(--muted);
  }

  .palette-empty p {
    margin: 0;
    font-size: var(--fs-13);
  }

  .palette-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--s-3) var(--s-4);
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    border-left: 2px solid transparent;
  }

  .palette-item:hover,
  .palette-item.selected {
    background: var(--surface-2);
    border-left-color: var(--blue);
  }

  .item-content {
    display: flex;
    align-items: flex-start;
    gap: var(--s-3);
    flex: 1;
    min-width: 0;
  }

  .item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--text-2);
    margin-top: 2px;
  }

  .item-text {
    flex: 1;
    min-width: 0;
  }

  .item-label {
    font-weight: 500;
    font-size: var(--fs-13);
    color: var(--text);
  }

  .item-description {
    font-size: var(--fs-12);
    color: var(--muted);
    margin-top: 2px;
  }

  .item-shortcut {
    display: flex;
    gap: 4px;
    font-size: var(--fs-11);
    color: var(--muted);
    flex-shrink: 0;
    margin-left: var(--s-4);
  }

  .palette-footer {
    padding: var(--s-3) var(--s-4);
    border-top: 1px solid var(--hairline);
    font-size: var(--fs-11);
    color: var(--muted);
    text-align: center;
    flex-shrink: 0;
  }
</style>
