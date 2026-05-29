<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    isTrashView?: boolean;
    viewMode?: 'grid' | 'list';
    sortBy?: 'name' | 'date' | 'size';
    sortOrder?: 'asc' | 'desc';
    onViewModeChange?: (mode: 'grid' | 'list') => void;
    onSortChange?: (sortBy: 'name' | 'date' | 'size') => void;
    onSortOrderChange?: (order: 'asc' | 'desc') => void;
  }

  let {
    isTrashView = false,
    viewMode = 'list',
    sortBy = 'date',
    sortOrder = 'desc',
    onViewModeChange,
    onSortChange,
    onSortOrderChange,
  }: Props = $props();

  let showSortMenu = $state(false);
</script>

<div class="toolbar">
  <div class="toolbar-left">
    <div class="sort-control">
      <button
        type="button"
        class="sort-btn"
        title="Sort options"
        onclick={() => showSortMenu = !showSortMenu}
      >
        <Icons.ArrowUpDown size={13} />
        <span>{sortBy}</span>
        {#if sortOrder === 'asc'}
          <Icons.ArrowUp size={11} />
        {:else}
          <Icons.ArrowDown size={11} />
        {/if}
      </button>
      {#if showSortMenu}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="sort-menu" onmouseleave={() => showSortMenu = false}>
          <button type="button" class:active={sortBy === 'date'} onclick={() => { onSortChange?.('date'); showSortMenu = false; }}>
            <Icons.Clock size={13} /> Date
          </button>
          <button type="button" class:active={sortBy === 'name'} onclick={() => { onSortChange?.('name'); showSortMenu = false; }}>
            <Icons.Type size={13} /> Name
          </button>
          <button type="button" class:active={sortBy === 'size'} onclick={() => { onSortChange?.('size'); showSortMenu = false; }}>
            <Icons.HardDrive size={13} /> Size
          </button>
          <div class="sort-divider"></div>
          <button type="button" class:active={sortOrder === 'asc'} onclick={() => { onSortOrderChange?.('asc'); showSortMenu = false; }}>
            <Icons.ArrowUp size={13} /> Ascending
          </button>
          <button type="button" class:active={sortOrder === 'desc'} onclick={() => { onSortOrderChange?.('desc'); showSortMenu = false; }}>
            <Icons.ArrowDown size={13} /> Descending
          </button>
        </div>
      {/if}
    </div>
  </div>

  <div class="view-toggle">
    <button
      type="button"
      title="Grid view"
      class:active={viewMode === 'grid'}
      onclick={() => onViewModeChange?.('grid')}
    >
      <Icons.Grid2x2 size={14} />
    </button>
    <button
      type="button"
      title="List view"
      class:active={viewMode === 'list'}
      onclick={() => onViewModeChange?.('list')}
    >
      <Icons.List size={14} />
    </button>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 16px 8px;
    gap: 8px;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sort-control { position: relative; }

  .sort-btn {
    height: 30px;
    padding: 0 10px;
    border-radius: 7px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    display: inline-flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
    font-family: inherit;
    font-size: 12px;
    font-weight: 500;
    transition: all 120ms;
  }
  .sort-btn:hover { border-color: var(--border-2); color: var(--text); }

  .sort-menu {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    z-index: 200;
    min-width: 150px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.3);
    padding: 4px;
  }

  .sort-menu button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 10px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 12px;
    border-radius: 5px;
    transition: background 100ms;
    text-align: left;
  }
  .sort-menu button:hover { background: var(--surface); color: var(--text); }
  .sort-menu button.active { color: var(--blue); font-weight: 600; }

  .sort-divider { height: 1px; background: var(--border); margin: 4px 0; }

  .view-toggle {
    display: inline-flex;
    border: 1px solid var(--border);
    border-radius: 7px;
    overflow: hidden;
  }

  .view-toggle button {
    width: 30px;
    height: 30px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 120ms;
  }
  .view-toggle button + button { border-left: 1px solid var(--border); }
  .view-toggle button.active { background: var(--surface-hi); color: var(--text); }
  .view-toggle button:hover:not(.active) { background: var(--surface-2); }
</style>
