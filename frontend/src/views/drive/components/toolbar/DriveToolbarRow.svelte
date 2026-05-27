<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    filterQuery?: string;
    isTrashView?: boolean;
    viewMode?: 'grid' | 'list';
    onFilterChange?: (query: string) => void;
    onViewModeChange?: (mode: 'grid' | 'list') => void;
  }

  let {
    filterQuery = '',
    isTrashView = false,
    viewMode = 'grid',
    onFilterChange,
    onViewModeChange,
  }: Props = $props();
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
