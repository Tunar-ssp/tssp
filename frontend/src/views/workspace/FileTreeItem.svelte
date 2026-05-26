<script lang="ts">
  import FileTreeItem from './FileTreeItem.svelte';

  interface TreeItem {
    path: string;
    is_dir: boolean;
    children?: TreeItem[];
  }

  interface Props {
    item: TreeItem;
    expanded: boolean;
    active: boolean;
    depth: number;
    onToggle: (path: string) => void;
    onSelect: (path: string) => void;
    onDelete: (path: string) => void;
    expandedDirs?: Record<string, boolean>;
    activeFilePath?: string | null;
  }

  let {
    item,
    expanded,
    active,
    depth,
    onToggle,
    onSelect,
    onDelete,
    expandedDirs = {},
    activeFilePath = null
  }: Props = $props();

  function getFileName(path: string): string {
    return path.split('/').pop() || path;
  }
</script>

<div class="tree-item" style="padding-left: {depth * 20}px">
  <div class="tree-row" class:active>
    {#if item.is_dir}
      <button
        class="tree-toggle"
        onclick={() => onToggle(item.path)}
        title="Toggle folder"
      >
        {expanded ? '▼' : '▶'}
      </button>
      <span class="tree-icon">📁</span>
    {:else}
      <span class="tree-toggle"></span>
      <span class="tree-icon">📄</span>
    {/if}

    <button
      type="button"
      class="tree-label"
      onclick={() => onSelect(item.path)}
      title="Select file"
    >
      {getFileName(item.path)}
    </button>

    {#if !item.is_dir}
      <button
        class="tree-action delete"
        onclick={() => onDelete(item.path)}
        title="Delete file"
      >
        ✕
      </button>
    {/if}
  </div>

  {#if item.is_dir && expanded && item.children}
    {#each item.children as child (child.path)}
      <FileTreeItem
        item={child}
        expanded={expandedDirs[child.path] || false}
        active={activeFilePath === child.path}
        depth={depth + 1}
        {onToggle}
        {onSelect}
        {onDelete}
        {expandedDirs}
        {activeFilePath}
      />
    {/each}
  {/if}
</div>

<style module>
  .tree-item {
    user-select: none;
  }

  .tree-row {
    display: flex;
    align-items: center;
    padding: 4px 4px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .tree-row:hover {
    background: var(--bg-tertiary);
  }

  .tree-row.active {
    background: var(--accent-color);
    color: white;
  }

  .tree-toggle {
    width: 16px;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    color: inherit;
    font-size: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .tree-icon {
    margin: 0 4px;
    font-size: 12px;
  }

  .tree-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 0;
    font-size: inherit;
    text-align: left;
  }

  .tree-label:focus {
    outline: 2px solid var(--accent-color);
    outline-offset: -2px;
  }

  .tree-action {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 2px 4px;
    font-size: 12px;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .tree-row:hover .tree-action {
    opacity: 1;
  }

  .tree-action.delete {
    color: var(--error-color);
  }
</style>
