<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    activeLens?: 'all' | 'images' | 'videos' | 'documents' | 'public' | 'trash';
    viewMode?: 'grid' | 'list';
    filterQuery?: string;
    onLensChange?: (lens: string) => void;
    onViewModeChange?: (mode: 'grid' | 'list') => void;
    onFilterChange?: (query: string) => void;
  }

  let {
    activeLens = 'all',
    viewMode = 'grid',
    filterQuery = '',
    onLensChange = () => {},
    onViewModeChange = () => {},
    onFilterChange = () => {},
  }: $$Props = $props();

  const lenses = [
    { id: 'all', label: 'All', icon: Icons.HardDrive },
    { id: 'images', label: 'Images', icon: Icons.Image },
    { id: 'videos', label: 'Videos', icon: Icons.Video },
    { id: 'documents', label: 'Documents', icon: Icons.FileText },
    { id: 'public', label: 'Public', icon: Icons.Globe },
    { id: 'trash', label: 'Trash', icon: Icons.Trash2 },
  ];
</script>

<div class="drive-filters">
  <div class="filter-search">
    <Icons.Search size={16} />
    <input
      type="text"
      placeholder="Search files..."
      value={filterQuery}
      oninput={(e) => onFilterChange((e.target as HTMLInputElement).value)}
    />
    {#if filterQuery}
      <button
        type="button"
        class="clear-btn"
        onclick={() => onFilterChange('')}
        aria-label="Clear search"
      >
        <Icons.X size={14} />
      </button>
    {/if}
  </div>

  <div class="filter-lenses">
    {#each lenses as lens}
      <button
        type="button"
        class="lens-btn"
        class:active={activeLens === lens.id}
        onclick={() => onLensChange(lens.id)}
        title={lens.label}
      >
        <svelte:component this={lens.icon} size={16} />
        <span class="lens-label">{lens.label}</span>
      </button>
    {/each}
  </div>

  <div class="view-controls">
    <button
      type="button"
      class="view-btn"
      class:active={viewMode === 'grid'}
      onclick={() => onViewModeChange('grid')}
      title="Grid view"
    >
      <Icons.Grid size={16} />
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
  .drive-filters {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .filter-search {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg);
  }

  .filter-search input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    outline: none;
    font-size: 13px;
  }

  .filter-search input::placeholder {
    color: var(--muted);
  }

  .clear-btn {
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.2s;
  }

  .clear-btn:hover {
    color: var(--text);
  }

  .filter-lenses {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding-bottom: 4px;
  }

  .lens-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 12px;
    white-space: nowrap;
    transition: all 0.2s;
    flex-shrink: 0;
  }

  .lens-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .lens-btn.active {
    background: var(--blue-soft);
    border-color: var(--blue);
    color: var(--blue);
  }

  .lens-label {
    display: none;
  }

  @media (min-width: 600px) {
    .lens-label {
      display: inline;
    }
  }

  .view-controls {
    display: flex;
    gap: 4px;
    margin-left: auto;
  }

  .view-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .view-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .view-btn.active {
    background: var(--blue-soft);
    border-color: var(--blue);
    color: var(--blue);
  }
</style>
