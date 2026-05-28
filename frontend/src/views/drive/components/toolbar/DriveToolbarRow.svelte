<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    filterQuery?: string;
    isTrashView?: boolean;
    viewMode?: 'grid' | 'list';
    sortBy?: 'name' | 'date' | 'size';
    sortOrder?: 'asc' | 'desc';
    onFilterChange?: (query: string) => void;
    onViewModeChange?: (mode: 'grid' | 'list') => void;
    onSortChange?: (sortBy: 'name' | 'date' | 'size') => void;
    onSortOrderChange?: (order: 'asc' | 'desc') => void;
  }

  let {
    filterQuery = '',
    isTrashView = false,
    viewMode = 'grid',
    sortBy = 'date',
    sortOrder = 'desc',
    onFilterChange,
    onViewModeChange,
    onSortChange,
    onSortOrderChange,
  }: Props = $props();

  let showSortMenu = $state(false);
</script>

<div class="toolbar">
  <div class="search-box">
    <Icons.Search size={15} />
    <input
      type="text"
      placeholder={isTrashView ? 'Search trash' : 'Search loaded files'}
      value={filterQuery}
      onchange={(e) => onFilterChange?.(e.currentTarget.value)}
    />
  </div>

  <div class="toolbar-right">
    <div class="sort-control">
      <button
        type="button"
        class="sort-btn"
        title={`Sort by ${sortBy} (${sortOrder})`}
        onclick={() => showSortMenu = !showSortMenu}
      >
        <Icons.ArrowUpDown size={15} />
        <span>{sortBy}</span>
        {#if sortOrder === 'asc'}
          <Icons.ArrowUp size={12} />
        {:else}
          <Icons.ArrowDown size={12} />
        {/if}
      </button>
      {#if showSortMenu}
        <div class="sort-menu">
          <button
            type="button"
            class:active={sortBy === 'date'}
            onclick={() => {
              onSortChange?.('date');
              showSortMenu = false;
            }}
          >
            <Icons.Clock size={14} />
            Date
          </button>
          <button
            type="button"
            class:active={sortBy === 'name'}
            onclick={() => {
              onSortChange?.('name');
              showSortMenu = false;
            }}
          >
            <Icons.Type size={14} />
            Name
          </button>
          <button
            type="button"
            class:active={sortBy === 'size'}
            onclick={() => {
              onSortChange?.('size');
              showSortMenu = false;
            }}
          >
            <Icons.HardDrive size={14} />
            Size
          </button>
          <div class="sort-divider"></div>
          <button
            type="button"
            class:active={sortOrder === 'asc'}
            onclick={() => {
              onSortOrderChange?.('asc');
              showSortMenu = false;
            }}
          >
            <Icons.ArrowUp size={14} />
            Ascending
          </button>
          <button
            type="button"
            class:active={sortOrder === 'desc'}
            onclick={() => {
              onSortOrderChange?.('desc');
              showSortMenu = false;
            }}
          >
            <Icons.ArrowDown size={14} />
            Descending
          </button>
        </div>
      {/if}
    </div>
    <div class="view-toggle">
      <button
        type="button"
        class:active={viewMode === 'grid'}
        onclick={() => onViewModeChange?.('grid')}
      >
        <Icons.Grid2x2 size={15} />
      </button>
      <button
        type="button"
        class:active={viewMode === 'list'}
        onclick={() => onViewModeChange?.('list')}
      >
        <Icons.List size={15} />
      </button>
    </div>
  </div>
</div>

<style>
  .toolbar {
    margin: 0 24px 16px;
    padding: 12px 14px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(16, 18, 24, 0.88);
    display: flex;
    gap: 14px;
    align-items: center;
    justify-content: space-between;
  }

  .search-box {
    flex: 1;
    min-width: 0;
    height: 42px;
    padding: 0 14px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
  }

  .search-box input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 14px;
  }

  .toolbar-right {
    display: flex;
    gap: 12px;
    align-items: center;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .sort-control {
    position: relative;
  }

  .sort-btn {
    width: auto;
    height: 32px;
    padding: 0 12px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    font-family: inherit;
    font-size: 12px;
    font-weight: 500;
    transition: all 150ms;
  }

  .sort-btn:hover {
    border-color: var(--border-2);
    color: var(--text);
  }

  .sort-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    z-index: 100;
    min-width: 160px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.3);
  }

  .sort-menu button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 13px;
    transition: background 150ms;
    text-align: left;
  }

  .sort-menu button:first-child {
    border-radius: 7px 7px 0 0;
  }

  .sort-menu button:hover {
    background: var(--surface);
    color: var(--text);
  }

  .sort-menu button.active {
    background: rgba(59, 130, 246, 0.1);
    color: var(--blue);
    font-weight: 500;
  }

  .sort-divider {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }

  .sort-menu button:last-child {
    border-radius: 0 0 7px 7px;
  }

  .view-toggle {
    display: inline-flex;
    gap: 8px;
  }

  .view-toggle button {
    width: 36px;
    height: 32px;
    padding: 0;
    justify-content: center;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-family: inherit;
    transition: all 150ms;
  }

  .view-toggle button.active {
    background: var(--surface-hi);
    border-color: var(--border-2);
    color: var(--text);
  }

  @media (max-width: 760px) {
    .toolbar {
      margin-left: 16px;
      margin-right: 16px;
      flex-direction: column;
      align-items: stretch;
    }

    .toolbar-right {
      justify-content: space-between;
    }
  }
</style>
