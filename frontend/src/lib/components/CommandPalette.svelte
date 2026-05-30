<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { api, type SearchResult } from '$lib/api';
  import { currentView, navigateTo, commandQuery, activeOverlays } from '$lib/stores/ui';
  import { error as notifyError } from '$lib/stores/notifications';
  import { registerKeyboardShortcuts } from '$lib/utils';

  import { isAdmin, user } from '$lib/stores/auth';

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
  }: $$Props = $props();

  let searchQuery = $state('');
  let selectedIndex = $state(0);
  let inputRef: HTMLInputElement | null = $state(null);
  let searchResults = $state<SearchResult[]>([]);
  let isSearching = $state(false);
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  const appCommands = $derived<Command[]>([
    {
      id: 'app-home',
      label: 'Open Launcher',
      description: 'Go to the product home and app launcher',
      icon: Icons.LayoutGrid,
      action: () => navigateTo('home'),
    },
    {
      id: 'app-drive',
      label: 'Open Cloud Drive',
      description: 'Browse files, folders, previews, and uploads',
      icon: Icons.Cloud,
      action: () => navigateTo('drive'),
    },
    {
      id: 'app-notes',
      label: 'Open Notes',
      description: 'Review notes, tags, and pinned pages',
      icon: Icons.BookText,
      action: () => navigateTo('notes'),
    },
    {
      id: 'app-workspace',
      label: 'Open Workspace',
      description: 'Switch into the editor workspace',
      icon: Icons.Code2,
      action: () => navigateTo('workspace'),
    },
    ...($isAdmin ? [{
      id: 'app-admin',
      label: 'Open Admin',
      description: 'Inspect system health and operations',
      icon: Icons.Shield,
      action: () => navigateTo('admin'),
    }] : []),
  ]);

  $effect(() => {
    if (isOpen && inputRef) {
      searchQuery = $commandQuery || '';
      selectedIndex = 0;
      inputRef.focus();
    }
  });

  const handlePaletteKeydown = (e: KeyboardEvent) => {
    if (!$activeOverlays.isTop('command-palette')) return;

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.stopPropagation();
      e.preventDefault();
      onClose?.();
    } else if (e.key === 'Escape') {
      e.stopPropagation();
      e.preventDefault();
      onClose?.();
    } else if (e.key === 'ArrowDown') {
      e.stopPropagation();
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, visibleItems.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.stopPropagation();
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter') {
      e.stopPropagation();
      e.preventDefault();
      visibleItems[selectedIndex]?.action();
      onClose?.();
    }
  };

  $effect(() => {
    if (isOpen) {
      window.addEventListener('keydown', handlePaletteKeydown, true);
      return () => window.removeEventListener('keydown', handlePaletteKeydown, true);
    }
  });

  $effect(() => {
    if (!isOpen) return;
    if (searchTimer) clearTimeout(searchTimer);

    if (!searchQuery.trim()) {
      searchResults = [];
      isSearching = false;
      return;
    }

    isSearching = true;
    searchTimer = setTimeout(async () => {
      try {
        const response = await api.search(searchQuery, 12);
        searchResults = response.results || [];
      } catch (err) {
        searchResults = [];
        notifyError('Search Failed', err instanceof Error ? err.message : 'Could not search');
      } finally {
        isSearching = false;
      }
    }, 180);
  });

  const filteredCommands = $derived(
    searchQuery
      ? commands.filter(
          (cmd) =>
            cmd.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
            cmd.description?.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : commands
  );

  const resultCommands = $derived(
    searchResults.map((result) => ({
      id: `result-${result.type}-${result.id}`,
      label: result.name || result.title || result.id,
      description:
        result.type === 'file'
          ? `${result.folder_path || 'bucket root'} · ${result.visibility || 'private'}`
          : result.snippet || `${result.type} result`,
      shortcut: result.type.toUpperCase(),
      icon:
        result.type === 'file'
          ? Icons.File
          : result.type === 'note'
            ? Icons.NotebookPen
            : Icons.FolderGit2,
      action: () => {
        if (result.type === 'file') {
          navigateTo('drive', { kind: 'file', id: result.id });
        } else if (result.type === 'note') {
          navigateTo('notes', { kind: 'note', id: result.id });
        } else {
          navigateTo('workspace', { kind: 'workspace', id: result.id });
        }
      },
    }))
  );

  const visibleItems = $derived(
    searchQuery.trim()
      ? [...resultCommands, ...filteredCommands]
      : [...appCommands, ...commands]
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
          placeholder="Search files, notes, workspaces, commands..."
          bind:value={searchQuery}
          class="palette-input"
        />
        <span class="search-meta">{isSearching ? 'searching' : $currentView}</span>
      </div>

      <div class="palette-results">
        {#if visibleItems.length === 0}
          <div class="palette-empty">
            <Icons.AlertCircle size={32} />
            <p>No results for "{searchQuery}"</p>
          </div>
        {:else}
          <div class="result-section">{searchQuery.trim() ? 'Search results' : 'Apps and commands'}</div>
          {#each visibleItems as cmd, idx (cmd.id)}
            {@const Icon = cmd.icon}
            <button
              type="button"
              class="palette-item"
              class:selected={idx === selectedIndex}
              onclick={() => {
                cmd.action();
                onClose?.();
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
        <span class="footer-hint">↑ ↓ to select • Enter to open • Esc to close</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .palette-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.62);
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
    width: min(92vw, 720px);
    background:
      linear-gradient(180deg, rgba(20, 22, 29, 0.98), rgba(10, 11, 16, 0.98)),
      radial-gradient(circle at 0% 0%, rgba(91, 227, 154, 0.08), transparent 42%);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 28px;
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    max-height: min(76vh, 640px);
    animation: paletteSlideIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes paletteSlideIn {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(-8px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .palette-search {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 20px 22px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
    color: var(--muted);
  }

  .palette-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 20px;
    outline: none;
    font-family: var(--ff-sans);
  }

  .palette-input::placeholder {
    color: var(--muted);
  }

  .search-meta {
    font-family: var(--ff-mono);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.18em;
  }

  .palette-results {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
  }

  .result-section {
    padding: 8px 10px 10px;
    color: var(--muted);
    font-family: var(--ff-mono);
    font-size: 12px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
  }

  .palette-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 64px 24px;
    color: var(--muted);
  }

  .palette-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 14px 16px;
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    border-radius: 18px;
  }

  .palette-item:hover,
  .palette-item.selected {
    background: rgba(255, 255, 255, 0.05);
  }

  .item-content {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }

  .item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: 14px;
    flex-shrink: 0;
    color: var(--text-2);
    background: rgba(255, 255, 255, 0.06);
  }

  .item-text {
    flex: 1;
    min-width: 0;
  }

  .item-label {
    font-weight: 600;
    font-size: 15px;
  }

  .item-description {
    font-size: 13px;
    color: var(--muted);
    margin-top: 3px;
  }

  .item-shortcut {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 56px;
    padding: 4px 8px;
    border-radius: 10px;
    font-size: 11px;
    color: var(--muted);
    background: rgba(255, 255, 255, 0.06);
    font-family: var(--ff-mono);
  }

  .palette-footer {
    padding: 14px 18px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    font-size: 12px;
    color: var(--muted);
    text-align: center;
    flex-shrink: 0;
  }
</style>
