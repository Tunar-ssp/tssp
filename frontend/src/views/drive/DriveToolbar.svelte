<script lang="ts">
  /**
   * Drive toolbar with filters, view mode, and lens controls
   * Extracted from DriveView for modularity and reusability
   */

  import * as Icons from 'lucide-svelte';

  interface Props {
    activeLens: 'all' | 'images' | 'videos' | 'documents' | 'public' | 'trash';
    viewMode: 'grid' | 'list';
    filterQuery: string;
    itemCount: number;
    onLensChange?: (lens: 'all' | 'images' | 'videos' | 'documents' | 'public' | 'trash') => void;
    onViewModeChange?: (mode: 'grid' | 'list') => void;
    onFilterChange?: (query: string) => void;
    onRefresh?: () => void;
    onUpload?: () => void;
    isLoading?: boolean;
  }

  let {
    activeLens,
    viewMode,
    filterQuery,
    itemCount,
    onLensChange,
    onViewModeChange,
    onFilterChange,
    onRefresh,
    onUpload,
    isLoading = false,
  }: Props = $props();

  const lenses = [
    { id: 'all' as const, label: 'All', icon: Icons.Files },
    { id: 'images' as const, label: 'Images', icon: Icons.Image },
    { id: 'videos' as const, label: 'Videos', icon: Icons.Play },
    { id: 'documents' as const, label: 'Docs', icon: Icons.File },
    { id: 'public' as const, label: 'Public', icon: Icons.Share2 },
    { id: 'trash' as const, label: 'Trash', icon: Icons.Trash2 },
  ];
</script>

<div class="toolbar">
  <div class="toolbar-left">
    <div class="search-box">
      <Icons.Search size={16} />
      <input
        type="text"
        placeholder="Search files..."
        value={filterQuery}
        oninput={(e) => onFilterChange?.((e.target as HTMLInputElement).value)}
      />
    </div>
  </div>

  <div class="toolbar-center">
    <div class="lens-buttons">
      {#each lenses as lens}
        <button
          class="lens-btn"
          class:active={activeLens === lens.id}
          onclick={() => onLensChange?.(lens.id)}
          title={lens.label}
          aria-label={lens.label}
        >
          <svelte:component this={lens.icon} size={16} />
          <span class="lens-label">{lens.label}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="toolbar-right">
    <div class="view-toggles">
      <button
        class="view-toggle"
        class:active={viewMode === 'grid'}
        onclick={() => onViewModeChange?.('grid')}
        title="Grid view"
        aria-label="Grid view"
      >
        <Icons.LayoutGrid size={16} />
      </button>
      <button
        class="view-toggle"
        class:active={viewMode === 'list'}
        onclick={() => onViewModeChange?.('list')}
        title="List view"
        aria-label="List view"
      >
        <Icons.List size={16} />
      </button>
    </div>

    <button class="action-btn" onclick={onRefresh} disabled={isLoading} title="Refresh">
      <span class:spinning={isLoading}>
        <Icons.RotateCw size={16} />
      </span>
    </button>

    <button class="action-btn primary" onclick={onUpload} title="Upload files">
      <Icons.Upload size={16} />
      <span>Upload</span>
    </button>
  </div>

  <div class="toolbar-status">
    <span class="item-count">{itemCount} items</span>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--bg);
    flex-wrap: wrap;
  }

  .toolbar-left {
    flex: 1;
    min-width: 200px;
  }

  .toolbar-center {
    flex: 0 1 auto;
  }

  .toolbar-right {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .toolbar-status {
    flex: 0 0 auto;
    color: var(--muted);
    font-size: 12px;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-secondary);
  }

  .search-box input {
    flex: 1;
    border: none;
    background: none;
    outline: none;
    color: var(--text);
    font-size: 13px;
  }

  .search-box input::placeholder {
    color: var(--muted);
  }

  .lens-buttons {
    display: flex;
    gap: 4px;
  }

  .lens-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-secondary);
    color: var(--text);
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .lens-btn:hover {
    background-color: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.3);
  }

  .lens-btn.active {
    background-color: rgba(59, 130, 246, 0.15);
    border-color: rgba(59, 130, 246, 0.5);
    color: var(--text);
  }

  .lens-label {
    display: none;
  }

  @media (min-width: 900px) {
    .lens-label {
      display: inline;
    }
  }

  .view-toggles {
    display: flex;
    gap: 4px;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    background: var(--bg-secondary);
  }

  .view-toggle {
    padding: 6px;
    border: none;
    background: none;
    color: var(--muted);
    cursor: pointer;
    transition: all 0.2s;
    border-radius: 4px;
  }

  .view-toggle:hover {
    color: var(--text);
    background: rgba(0, 0, 0, 0.05);
  }

  .view-toggle.active {
    background-color: var(--bg);
    color: var(--text);
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-secondary);
    color: var(--text);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
  }

  .action-btn:hover:not(:disabled) {
    background-color: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.3);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-btn.primary {
    background-color: rgba(59, 130, 246, 0.15);
    border-color: rgba(59, 130, 246, 0.3);
    color: #3b82f6;
    font-weight: 500;
  }

  .action-btn.primary:hover:not(:disabled) {
    background-color: rgba(59, 130, 246, 0.25);
  }

  .spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 768px) {
    .toolbar {
      gap: 8px;
      padding: 8px 12px;
    }

    .toolbar-left {
      min-width: 150px;
      flex: 1 1 100%;
      order: 1;
    }

    .toolbar-center {
      flex: 1 1 100%;
      order: 3;
    }

    .toolbar-right {
      flex: 1 1 100%;
      order: 2;
    }

    .lens-buttons {
      flex-wrap: wrap;
      gap: 2px;
    }

    .lens-btn {
      padding: 4px 8px;
      font-size: 11px;
    }
  }
</style>
