<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface FileNode {
    id: string;
    name: string;
    type: 'file' | 'folder';
    children?: FileNode[];
    isExpanded?: boolean;
  }

  interface $$Props {
    files?: FileNode[];
    onSelectFile?: (fileId: string, name: string) => void;
    onCreateFile?: (parentId: string) => void;
    onCreateFolder?: (parentId: string) => void;
    onDeleteFile?: (fileId: string) => void;
    class?: string;
  }

  let {
    files = [],
    onSelectFile,
    onCreateFile,
    onCreateFolder,
    onDeleteFile,
    class: className,
  } = $props<$$Props>();

  let expandedFolders = $state<Set<string>>(new Set());

  function toggleFolder(fileId: string) {
    if (expandedFolders.has(fileId)) {
      expandedFolders.delete(fileId);
    } else {
      expandedFolders.add(fileId);
    }
    expandedFolders = expandedFolders;
  }

  function getFileIcon(fileName: string) {
    if (fileName.endsWith('.ts') || fileName.endsWith('.tsx')) return Icons.Code2;
    if (fileName.endsWith('.js') || fileName.endsWith('.jsx')) return Icons.Code2;
    if (fileName.endsWith('.json')) return Icons.FileJson;
    if (fileName.endsWith('.css') || fileName.endsWith('.scss')) return Icons.Palette;
    if (fileName.endsWith('.html')) return Icons.Code2;
    if (fileName.endsWith('.md')) return Icons.FileText;
    return Icons.File;
  }
</script>

<div class="file-explorer {className || ''}">
  <div class="explorer-header">
    <h3>Files</h3>
    <div class="explorer-actions">
      <button class="action-btn" on:click={() => onCreateFile?.('root')} title="New file">
        <Icons.FileText size={14} />
      </button>
      <button class="action-btn" on:click={() => onCreateFolder?.('root')} title="New folder">
        <Icons.FolderPlus size={14} />
      </button>
    </div>
  </div>

  <div class="explorer-tree">
    {#each files as file (file.id)}
      <div class="tree-item" style="--depth: 0">
        <div class="item-row">
          {#if file.type === 'folder'}
            <button
              class="expand-btn"
              on:click={() => toggleFolder(file.id)}
              title={expandedFolders.has(file.id) ? 'Collapse' : 'Expand'}
            >
              <Icons.ChevronRight
                size={14}
                style="transform: rotate({expandedFolders.has(file.id) ? 90 : 0}deg)"
              />
            </button>
            <Icons.Folder size={14} />
          {:else}
            <div class="spacer"></div>
            <svelte:component this={getFileIcon(file.name)} size={14} />
          {/if}

          <span class="item-name" on:click={() => onSelectFile?.(file.id, file.name)}>
            {file.name}
          </span>

          <button
            class="delete-btn"
            on:click={() => onDeleteFile?.(file.id)}
            title="Delete"
          >
            <Icons.Trash2 size={12} />
          </button>
        </div>

        {#if file.type === 'folder' && expandedFolders.has(file.id) && file.children}
          {#each file.children as child (child.id)}
            <div class="tree-item nested">
              <div class="item-row">
                {#if child.type === 'folder'}
                  <button class="expand-btn" on:click={() => toggleFolder(child.id)}>
                    <Icons.ChevronRight
                      size={14}
                      style="transform: rotate({expandedFolders.has(child.id) ? 90 : 0}deg)"
                    />
                  </button>
                  <Icons.Folder size={14} />
                {:else}
                  <div class="spacer"></div>
                  <svelte:component this={getFileIcon(child.name)} size={14} />
                {/if}

                <span class="item-name" on:click={() => onSelectFile?.(child.id, child.name)}>
                  {child.name}
                </span>

                <button
                  class="delete-btn"
                  on:click={() => onDeleteFile?.(child.id)}
                  title="Delete"
                >
                  <Icons.Trash2 size={12} />
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .file-explorer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
    border-right: 1px solid var(--border);
  }

  .explorer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .explorer-header h3 {
    margin: 0;
    font-size: var(--fs-13);
    font-weight: 600;
    color: var(--text);
  }

  .explorer-actions {
    display: flex;
    gap: var(--s-2);
  }

  .action-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-1);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .action-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .explorer-tree {
    flex: 1;
    overflow-y: auto;
  }

  .tree-item {
    padding-left: calc(var(--depth) * 16px);
  }

  .tree-item.nested {
    --depth: 1;
  }

  .item-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-2);
    color: var(--text-2);
    font-size: var(--fs-12);
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .item-row:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .expand-btn {
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: inherit;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .spacer {
    width: 20px;
    height: 20px;
  }

  .item-name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .delete-btn {
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-1);
    opacity: 0;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .item-row:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: rgba(255, 107, 107, 0.1);
    color: var(--danger);
  }
</style>
