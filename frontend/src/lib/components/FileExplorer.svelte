<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface FileItem {
    id: string;
    name: string;
    type: 'file' | 'folder';
    path?: string;
    children?: FileItem[];
    expanded?: boolean;
  }

  interface $$Props {
    files?: FileItem[];
    selectedFileId?: string | null;
    onSelectFile?: (id: string) => void;
    onCreateFile?: () => void;
    onCreateFolder?: () => void;
    onDeleteFile?: (id: string) => void;
    onRenameFile?: (id: string) => void;
  }

  let {
    files = [],
    selectedFileId = null,
    onSelectFile = () => {},
    onCreateFile = () => {},
    onCreateFolder = () => {},
    onDeleteFile = () => {},
    onRenameFile = () => {},
  }: $$Props = $props();

  let expanded: Record<string, boolean> = $state({});

  function toggleFolder(id: string) {
    expanded[id] = !expanded[id];
  }

  function flattenFiles(items: FileItem[], depth = 0): any[] {
    let result: any[] = [];
    items.forEach((item) => {
      const isOpen = expanded[item.id];
      result.push({
        ...item,
        depth,
        isOpen,
        hasChildren: item.children && item.children.length > 0,
      });
      if (isOpen && item.children) {
        result = result.concat(flattenFiles(item.children, depth + 1));
      }
    });
    return result;
  }

  let flattenedFiles = $derived(flattenFiles(files));
</script>

<div class="file-explorer">
  <div class="explorer-header">
    <h3>Files</h3>
    <div class="explorer-actions">
      <button
        class="action-btn"
        title="New file"
        onclick={onCreateFile}
      >
        <Icons.Plus size={14} />
      </button>
      <button
        class="action-btn"
        title="New folder"
        onclick={onCreateFolder}
      >
        <Icons.FolderPlus size={14} />
      </button>
    </div>
  </div>

  <div class="explorer-tree">
    {#each flattenedFiles as item (item.id)}
      <div
        class="file-item"
        style="padding-left: {8 + item.depth * 16}px"
      >
        {#if item.type === 'folder'}
          <button
            class="folder-toggle"
            onclick={() => toggleFolder(item.id)}
            title={item.isOpen ? 'Collapse' : 'Expand'}
          >
            <Icons.ChevronRight
              size={16}
              style="transform: rotate({item.isOpen ? 90 : 0}deg); transition: transform 0.2s;"
            />
          </button>
          <Icons.Folder size={16} class="folder-icon" />
          <span class="item-name">{item.name}</span>
        {:else}
          <div class="file-toggle"></div>
          <Icons.File size={16} class="file-icon" />
          <button
            class="file-button"
            class:active={selectedFileId === item.id}
            onclick={() => onSelectFile(item.id)}
            title={item.name}
          >
            {item.name}
          </button>
        {/if}
      </div>
    {/each}

    {#if flattenedFiles.length === 0}
      <div class="explorer-empty">
        <Icons.FolderOpen size={24} />
        <p>No files yet</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .file-explorer {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-right: 1px solid var(--border);
    overflow: hidden;
  }

  .explorer-header {
    padding: var(--s-3) var(--s-4);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .explorer-header h3 {
    margin: 0;
    font-size: var(--fs-13);
    font-weight: 600;
    color: var(--text-2);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .explorer-actions {
    display: flex;
    gap: var(--s-2);
  }

  .action-btn {
    width: 28px;
    height: 28px;
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

  .action-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .explorer-tree {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-1) 0;
  }

  .explorer-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    padding: var(--s-6);
    color: var(--muted);
    text-align: center;
  }

  .explorer-empty p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-2);
    cursor: pointer;
    transition: background var(--duration-quick) var(--ease-smooth);
  }

  .file-item:hover {
    background: var(--surface-2);
  }

  .folder-toggle {
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
    flex-shrink: 0;
  }

  .folder-toggle:hover {
    color: var(--text);
  }

  .file-toggle {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .folder-icon {
    flex-shrink: 0;
    color: var(--orange);
  }

  .file-icon {
    flex-shrink: 0;
    color: var(--muted);
  }

  .item-name {
    font-size: var(--fs-12);
    font-weight: 600;
    color: var(--text-2);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-button {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-12);
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-button:hover {
    color: var(--text);
  }

  .file-button.active {
    color: var(--blue);
    font-weight: 600;
  }
</style>
