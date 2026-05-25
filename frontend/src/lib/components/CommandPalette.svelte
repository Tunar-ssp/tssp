<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { commandPaletteOpen } from '$lib/stores/ui';
  import { files } from '$lib/stores/drive';
  import { notes } from '$lib/stores/notes';
  import { workspaces } from '$lib/stores/workspace';
  import { currentView } from '$lib/stores/ui';
  import { onMount, onDestroy } from 'svelte';

  let searchQuery = '';
  let selectedIndex = 0;
  let results: any[] = [];

  interface SearchResult {
    type: 'file' | 'note' | 'workspace' | 'action';
    id: string;
    title: string;
    subtitle?: string;
    icon: any;
    action?: () => void;
  }

  function getSearchResults(): SearchResult[] {
    const query = searchQuery.toLowerCase();
    if (!query.trim()) {
      return [
        { type: 'action', id: 'home', title: 'Home', subtitle: 'Go to launcher', icon: Icons.Home, action: () => { $currentView = 'home'; close(); } },
        { type: 'action', id: 'drive', title: 'Drive', subtitle: 'Cloud Drive', icon: Icons.HardDrive, action: () => { $currentView = 'drive'; close(); } },
        { type: 'action', id: 'notes', title: 'Notes', subtitle: 'Notes app', icon: Icons.BookOpen, action: () => { $currentView = 'notes'; close(); } },
        { type: 'action', id: 'workspace', title: 'Workspace', subtitle: 'Code editor', icon: Icons.Code2, action: () => { $currentView = 'workspace'; close(); } },
        { type: 'action', id: 'operations', title: 'Operations', subtitle: 'Admin console', icon: Icons.Settings, action: () => { $currentView = 'operations'; close(); } },
      ];
    }

    const allResults: SearchResult[] = [];

    // Search files
    $files.forEach(f => {
      if (f.name.toLowerCase().includes(query)) {
        allResults.push({
          type: 'file',
          id: f.id,
          title: f.name,
          subtitle: `${(f.size_bytes / 1024 / 1024).toFixed(1)} MB`,
          icon: Icons.File,
          action: () => { $currentView = 'drive'; close(); },
        });
      }
    });

    // Search notes
    $notes.forEach(n => {
      if (n.title.toLowerCase().includes(query) || n.body.toLowerCase().includes(query)) {
        allResults.push({
          type: 'note',
          id: n.id,
          title: n.title || 'Untitled',
          subtitle: n.body.slice(0, 50),
          icon: Icons.BookOpen,
          action: () => { $currentView = 'notes'; close(); },
        });
      }
    });

    // Search workspaces
    $workspaces.forEach(w => {
      if (w.name.toLowerCase().includes(query)) {
        allResults.push({
          type: 'workspace',
          id: w.id,
          title: w.name,
          subtitle: w.language,
          icon: Icons.Code2,
          action: () => { $currentView = 'workspace'; close(); },
        });
      }
    });

    return allResults;
  }

  $: results = getSearchResults();
  $: selectedIndex = Math.min(selectedIndex, Math.max(0, results.length - 1));

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % results.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + results.length) % results.length;
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (results[selectedIndex]) {
        results[selectedIndex].action?.();
      }
    }
  }

  function close() {
    $commandPaletteOpen = false;
    searchQuery = '';
    selectedIndex = 0;
  }

  function openPalette() {
    if ($commandPaletteOpen) {
      close();
    }
  }

  onMount(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        $commandPaletteOpen = !$commandPaletteOpen;
        if ($commandPaletteOpen) {
          searchQuery = '';
          selectedIndex = 0;
          setTimeout(() => {
            const input = document.querySelector('.palette-input') as HTMLInputElement;
            input?.focus();
          });
        }
      }
    };

    window.addEventListener('keydown', handleGlobalKeyDown);
    return () => window.removeEventListener('keydown', handleGlobalKeyDown);
  });
</script>

{#if $commandPaletteOpen}
  <div class="overlay" on:click={close}>
    <div class="palette" on:click|stopPropagation>
      <div class="palette-header">
        <Icons.Search size={20} />
        <input
          class="palette-input"
          type="text"
          placeholder="Search files, notes, workspaces, or actions..."
          bind:value={searchQuery}
          on:keydown={handleKeyDown}
          autofocus
        />
        <span class="palette-hint">ESC</span>
      </div>

      <div class="palette-results">
        {#if results.length === 0}
          <div class="empty-state">
            <Icons.Search size={32} />
            <p>No results found</p>
            {#if searchQuery}
              <p class="hint">Try searching for files, notes, or workspaces</p>
            {/if}
          </div>
        {:else}
          {#each results as result, i (result.id)}
            <button
              class="result-item {i === selectedIndex ? 'selected' : ''}"
              on:click={() => result.action?.()}
            >
              <div class="result-icon">
                <svelte:component this={result.icon} size={16} />
              </div>
              <div class="result-content">
                <div class="result-title">{result.title}</div>
                {#if result.subtitle}
                  <div class="result-subtitle">{result.subtitle}</div>
                {/if}
              </div>
              <div class="result-type">
                <span class="badge {result.type}">
                  {#if result.type === 'file'}File{:else if result.type === 'note'}Note{:else if result.type === 'workspace'}Code{:else}Action{/if}
                </span>
              </div>
            </button>
          {/each}
        {/if}
      </div>

      <div class="palette-footer">
        <div class="hint-group">
          <span class="hint-key">↑↓</span>
          <span class="hint-text">Navigate</span>
        </div>
        <div class="hint-group">
          <span class="hint-key">Enter</span>
          <span class="hint-text">Select</span>
        </div>
        <div class="hint-group">
          <span class="hint-key">ESC</span>
          <span class="hint-text">Close</span>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 80px;
    z-index: 100;
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from { background: rgba(0, 0, 0, 0); }
    to { background: rgba(0, 0, 0, 0.5); }
  }

  .palette {
    width: 90%;
    max-width: 600px;
    border-radius: var(--r-5);
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    max-height: 60vh;
    animation: slideUp 0.2s ease-out;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(-20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .palette-header {
    padding: 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--text-2);
  }

  .palette-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-14);
    outline: none;
  }

  .palette-input::placeholder {
    color: var(--muted);
  }

  .palette-hint {
    font-size: 11px;
    color: var(--muted);
    padding: 4px 8px;
    background: var(--surface-2);
    border-radius: 4px;
    flex-shrink: 0;
  }

  .palette-results {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 40px 20px;
    color: var(--muted);
  }

  .empty-state p {
    margin: 0;
  }

  .empty-state .hint {
    font-size: var(--fs-12);
    color: var(--dim);
  }

  .result-item {
    padding: 12px 16px;
    border: none;
    background: transparent;
    border-bottom: 1px solid var(--hairline);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 12px;
    text-align: left;
    transition: background 0.15s;
  }

  .result-item:hover,
  .result-item.selected {
    background: var(--surface-2);
  }

  .result-icon {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    border-radius: var(--r-2);
    background: var(--surface-3);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-2);
  }

  .result-content {
    flex: 1;
    min-width: 0;
  }

  .result-title {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-subtitle {
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .result-type {
    flex-shrink: 0;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .badge.file {
    background: rgba(110, 168, 255, 0.1);
    color: var(--blue);
  }

  .badge.note {
    background: rgba(251, 191, 36, 0.1);
    color: var(--warning);
  }

  .badge.workspace {
    background: rgba(91, 227, 154, 0.1);
    color: var(--green);
  }

  .badge.action {
    background: rgba(163, 148, 255, 0.1);
    color: var(--violet);
  }

  .palette-footer {
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    display: flex;
    gap: 20px;
    font-size: 11px;
    background: var(--bg);
    color: var(--muted);
  }

  .hint-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .hint-key {
    padding: 2px 6px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 3px;
    font-weight: 500;
    color: var(--text-2);
  }

  .hint-text {
    color: var(--muted);
  }
</style>
