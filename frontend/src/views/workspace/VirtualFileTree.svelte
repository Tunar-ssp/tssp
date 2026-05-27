<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import FileTreeItem from './FileTreeItem.svelte';

  interface TreeItem {
    path: string;
    name: string;
    is_dir: boolean;
    children?: TreeItem[];
    size_bytes?: number;
  }

  interface $$Props {
    items?: TreeItem[];
    activeFilePath?: string | null;
    onSelectFile?: (path: string) => void;
    onExpandFolder?: (path: string) => void;
    onCreateFile?: (path: string) => void;
    onCreateFolder?: (path: string) => void;
    onDeleteFile?: (path: string) => void;
    onRenameFile?: (path: string, newName: string) => void;
  }

  let {
    items = [],
    activeFilePath = null,
    onSelectFile = () => {},
    onExpandFolder = () => {},
    onCreateFile = () => {},
    onCreateFolder = () => {},
    onDeleteFile = () => {},
    onRenameFile = () => {},
  }: $$Props = $props();

  let expandedDirs = $state<Record<string, boolean>>({});
  let searchQuery = $state('');
  let contextMenu = $state({ visible: false, x: 0, y: 0, path: '' });

  function formatSize(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    return `${size.toFixed(1)}${units[unitIndex]}`;
  }

  function getFileIcon(path: string, isDir: boolean) {
    if (isDir) return '📁';
    const ext = path.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'js': case 'jsx': return '📜';
      case 'ts': case 'tsx': return '📘';
      case 'py': return '🐍';
      case 'rs': return '⚙️';
      case 'go': return '🐹';
      case 'md': return '📝';
      case 'html': return '🌐';
      case 'css': return '🎨';
      case 'json': return '⚡';
      case 'sql': return '🗄️';
      case 'sh': case 'bash': return '💻';
      default: return '📄';
    }
  }

  function toggleFolder(path: string) {
    if (expandedDirs[path]) {
      delete expandedDirs[path];
      expandedDirs = expandedDirs;
    } else {
      expandedDirs[path] = true;
      onExpandFolder(path);
    }
  }

  function filterItems(items: TreeItem[], query: string): TreeItem[] {
    if (!query.trim()) return items;
    const lowerQuery = query.toLowerCase();
    return items.filter((item) => {
      const nameMatch = item.name.toLowerCase().includes(lowerQuery);
      const childrenMatch = item.is_dir && item.children
        ? filterItems(item.children, query).length > 0
        : false;
      return nameMatch || childrenMatch;
    }).map((item) => {
      if (item.is_dir && item.children) {
        return {
          ...item,
          children: filterItems(item.children, query),
        };
      }
      return item;
    });
  }

  function handleContextMenu(e: MouseEvent, path: string) {
    e.preventDefault();
    contextMenu = {
      visible: true,
      x: e.clientX,
      y: e.clientY,
      path,
    };
  }

  function handleSelectFile(path: string) {
    onSelectFile(path);
    searchQuery = '';
  }

  let filteredItems = $derived(filterItems(items, searchQuery));

  type RenderedItem = TreeItem & { depth: number; expanded: boolean; children: RenderedItem[] };

  function renderTree(items: TreeItem[], depth = 0): RenderedItem[] {
    return items.map((item): RenderedItem => ({
      ...item,
      depth,
      expanded: expandedDirs[item.path] || false,
      children: item.is_dir && expandedDirs[item.path] && item.children
        ? renderTree(item.children, depth + 1)
        : [],
    }));
  }

  let flattenedItems = $derived.by(() => {
    const result: any[] = [];
    const stack = renderTree(filteredItems);

    while (stack.length > 0) {
      const item = stack.shift();
      if (!item) continue;
      result.push(item);
      if (item.is_dir && item.expanded && item.children) {
        stack.unshift(...(item.children as any[]).reverse());
      }
    }
    return result;
  });
</script>

<div class="virtual-file-tree">
  <div class="tree-header">
    <h3>Explorer</h3>
    <div class="tree-actions">
      <button
        type="button"
        class="tree-action-btn"
        title="Create File"
        aria-label="Create new file"
      >
        <Icons.FilePlus size={14} />
      </button>
      <button
        type="button"
        class="tree-action-btn"
        title="Create Folder"
        aria-label="Create new folder"
      >
        <Icons.FolderPlus size={14} />
      </button>
    </div>
  </div>

  <div class="tree-search">
    <Icons.Search size={14} />
    <input
      type="text"
      placeholder="Search files..."
      bind:value={searchQuery}
      class="search-input"
      aria-label="Search files"
    />
    {#if searchQuery}
      <button
        type="button"
        onclick={() => searchQuery = ''}
        class="search-clear"
        aria-label="Clear search"
      >
        <Icons.X size={14} />
      </button>
    {/if}
  </div>

  <div class="tree-content">
    {#if filteredItems.length === 0}
      <div class="empty-state">
        <Icons.Inbox size={32} />
        <p>No files</p>
      </div>
    {:else}
      <div class="tree-list">
        {#each flattenedItems as item (item.path)}
          <div
            class="tree-row"
            style="padding-left: {(item.depth || 0) * 16}px"
            class:active={activeFilePath === item.path}
            oncontextmenu={(e) => handleContextMenu(e, item.path)}
          >
            {#if item.is_dir}
              <button
                type="button"
                class="tree-toggle"
                onclick={() => toggleFolder(item.path)}
                aria-label={item.expanded ? 'Collapse folder' : 'Expand folder'}
              >
                {item.expanded ? '▼' : '▶'}
              </button>
            {:else}
              <span class="tree-toggle"></span>
            {/if}

            <span class="tree-icon">{getFileIcon(item.path, item.is_dir)}</span>

            <button
              type="button"
              class="tree-label"
              onclick={() => handleSelectFile(item.path)}
              title={item.path}
            >
              {item.name}
            </button>

            {#if !item.is_dir && item.size_bytes}
              <span class="tree-size">{formatSize(item.size_bytes)}</span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Context Menu -->
  {#if contextMenu.visible}
    <div
      class="context-menu"
      style="left: {contextMenu.x}px; top: {contextMenu.y}px"
      onmouseleave={() => contextMenu.visible = false}
    >
      <button class="context-item">
        <Icons.Copy size={14} />
        Copy Path
      </button>
      <button class="context-item">
        <Icons.Edit size={14} />
        Rename
      </button>
      <button class="context-item danger">
        <Icons.Trash2 size={14} />
        Delete
      </button>
    </div>
  {/if}
</div>

<style>
  .virtual-file-tree {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
    border-right: 1px solid var(--border);
    overflow: hidden;
  }

  .tree-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-3);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .tree-header h3 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }

  .tree-actions {
    display: flex;
    gap: var(--s-1);
  }

  .tree-action-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    border-radius: var(--r-1);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .tree-action-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .tree-search {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    border: none;
    background: var(--bg);
    color: var(--text);
    font-size: var(--fs-12);
    padding: var(--s-1) var(--s-2);
    border-radius: var(--r-1);
    outline: none;
  }

  .search-input::placeholder {
    color: var(--muted);
  }

  .search-input:focus {
    border: 1px solid var(--blue);
  }

  .search-clear {
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
  }

  .tree-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    height: 100%;
    color: var(--muted);
  }

  .tree-list {
    padding: var(--s-1);
  }

  .tree-row {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    padding: var(--s-1) var(--s-2);
    border-radius: var(--r-1);
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    user-select: none;
  }

  .tree-row:hover {
    background: var(--surface-2);
  }

  .tree-row.active {
    background: var(--blue-soft);
  }

  .tree-toggle {
    width: 16px;
    height: 16px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .tree-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .tree-label {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-12);
    text-align: left;
    cursor: pointer;
    padding: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tree-label:focus {
    outline: none;
  }

  .tree-size {
    font-size: 10px;
    color: var(--muted);
    flex-shrink: 0;
    padding: 0 var(--s-1);
    border-left: 1px solid var(--hairline);
  }

  .context-menu {
    position: fixed;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    box-shadow: var(--shadow-card);
    z-index: 1000;
    min-width: 140px;
  }

  .context-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-12);
    cursor: pointer;
    text-align: left;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .context-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .context-item.danger:hover {
    background: rgba(255, 107, 107, 0.1);
    color: var(--danger);
  }
</style>
