<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord } from '$lib/api';

  interface $$Props {
    files?: FileRecord[];
    currentFolder?: string;
    viewMode?: 'grid' | 'list';
    selectedFile?: FileRecord | null;
    onSelectFile?: (file: FileRecord) => void;
    onContextMenu?: (event: MouseEvent, file: FileRecord) => void;
    onPreview?: (file: FileRecord) => void;
  }

  let {
    files = [],
    currentFolder = '',
    viewMode = 'grid',
    selectedFile = null,
    onSelectFile = () => {},
    onContextMenu = () => {},
    onPreview = () => {},
  }: $$Props = $props();

  let folderBreadcrumbs = $derived.by(() => {
    if (!currentFolder) return [];
    return currentFolder.split('/').filter(Boolean);
  });

  function getParentFolder(): string {
    if (!currentFolder) return '';
    const parts = currentFolder.split('/').filter(Boolean);
    parts.pop();
    return parts.join('/');
  }

  function formatBytes(bytes: number): string {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    const value = bytes / 1024 ** index;
    return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
  }

  function getFolderCount(path: string): number {
    return files.filter((f) => f.folder_path === path).length;
  }
</script>

<div class="folder-browser">
  {#if currentFolder}
    <div class="breadcrumb">
      <button
        type="button"
        class="breadcrumb-btn"
        onclick={() => (currentFolder = getParentFolder())}
      >
        <Icons.ChevronLeft size={16} />
        Back
      </button>
      <span class="current-folder">{currentFolder.split('/').pop()}</span>
    </div>
  {/if}

  {#if files.length === 0}
    <div class="empty-state">
      <Icons.FolderOpen size={48} />
      <p>No files in this folder</p>
    </div>
  {:else}
    <div class="file-list" class:grid-view={viewMode === 'grid'}>
      {#each files as file (file.id)}
        <div
          class="file-item"
          class:selected={selectedFile?.id === file.id}
          onclick={() => onSelectFile(file)}
          oncontextmenu={(e) => onContextMenu(e, file)}
        >
          {#if viewMode === 'grid'}
            <div class="file-icon-large">
              {file.folder_path ? '📁' : '📄'}
            </div>
            <div class="file-info">
              <span class="file-name">{file.name}</span>
              <span class="file-meta">
                {file.folder_path ? `${getFolderCount(file.folder_path)} items` : formatBytes(file.size_bytes)}
              </span>
            </div>
          {:else}
            <div class="file-list-item">
              <span class="file-icon">{file.folder_path ? '📁' : '📄'}</span>
              <span class="file-name">{file.name}</span>
              <span class="file-size">{formatBytes(file.size_bytes)}</span>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .folder-browser {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 16px;
  }

  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border-bottom: 1px solid var(--border);
  }

  .breadcrumb-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    border: none;
    background: transparent;
    color: var(--blue);
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.2s;
  }

  .breadcrumb-btn:hover {
    background: var(--blue-soft);
  }

  .current-folder {
    color: var(--text);
    font-weight: 500;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 60px 20px;
    color: var(--muted);
    text-align: center;
  }

  .file-list {
    flex: 1;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 12px;
    padding: 16px;
  }

  .file-list.grid-view {
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  }

  .file-item {
    display: flex;
    flex-direction: column;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: transparent;
    cursor: pointer;
    transition: all 0.2s;
  }

  .file-item:hover {
    background: var(--surface-2);
    border-color: var(--blue);
  }

  .file-item.selected {
    background: var(--blue-soft);
    border-color: var(--blue);
  }

  .file-icon-large {
    font-size: 32px;
    margin-bottom: 8px;
  }

  .file-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta {
    font-size: 11px;
    color: var(--muted);
  }

  .file-list-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    border-radius: 6px;
    width: 100%;
  }

  .file-icon {
    font-size: 18px;
    flex-shrink: 0;
  }

  .file-size {
    margin-left: auto;
    font-size: 12px;
    color: var(--muted);
  }
</style>
