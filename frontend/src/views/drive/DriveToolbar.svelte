<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    filterQuery: string;
    isSearching: boolean;
    viewMode: 'list' | 'grid';
    onSearch: (query: string) => void;
    onViewModeChange: (mode: 'list' | 'grid') => void;
  }

  let { filterQuery, isSearching, viewMode, onSearch, onViewModeChange }: Props = $props();
</script>

<div class="toolbar">
  <div class="search-box">
    <Icons.Search size={16} />
    <input
      type="text"
      placeholder="Search files... (server-side)"
      value={filterQuery}
      oninput={(e) => onSearch((e.target as HTMLInputElement).value)}
      class="search-input"
    />
    {#if isSearching}
      <div class="searching-indicator">
        <div class="spinner"></div>
      </div>
    {/if}
  </div>

  <div class="view-controls">
    <button
      type="button"
      class="view-btn"
      class:active={viewMode === 'grid'}
      onclick={() => onViewModeChange('grid')}
      title="Grid view"
    >
      <Icons.Grid2x2 size={16} />
    </button>
    <button
      type="button"
      class="view-btn"
      class:active={viewMode === 'list'}
      onclick={() => onViewModeChange('list')}
      title="List view"
    >
      <Icons.List size={16} />
    </button>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-3) var(--s-4);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    flex: 1;
    max-width: 400px;
    padding: var(--s-2) var(--s-3);
    background: var(--surface-2);
    border-radius: var(--r-2);
    color: var(--muted);
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-13);
    outline: none;
  }

  .search-input::placeholder {
    color: var(--muted);
  }

  .searching-indicator {
    display: flex;
    align-items: center;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .view-controls {
    display: flex;
    gap: var(--s-1);
    flex-shrink: 0;
  }

  .view-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-2);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
  }

  .view-btn:hover {
    background: var(--surface-2);
  }

  .view-btn.active {
    background: var(--blue);
    border-color: var(--blue);
    color: white;
  }
</style>
