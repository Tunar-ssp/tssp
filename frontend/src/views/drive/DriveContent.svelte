<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import TrashView from './TrashView.svelte';
  import type { FileRecord } from '$lib/api';
  import { formatBytes, formatRelative } from '$lib/utils';

  interface Props {
    files?: FileRecord[];
    trash?: FileRecord[];
    isLoading?: boolean;
    isTrashView?: boolean;
    viewMode?: 'grid' | 'list';
    selectedFileId?: string;
    hasMore?: boolean;
    isLoadingMore?: boolean;
    onSelectFile?: (file: FileRecord) => void;
    onPreviewFile?: (file: FileRecord) => void;
    onContextMenu?: (event: MouseEvent, file: FileRecord) => void;
    onLoadMore?: () => void;
    onRestore?: (file: FileRecord) => void;
    onDelete?: (file: FileRecord) => void;
    onPurgeTrash?: () => void;
    onUpload?: () => void;
  }

  let {
    files = [],
    trash = [],
    isLoading = false,
    isTrashView = false,
    viewMode = 'grid',
    selectedFileId,
    hasMore = false,
    isLoadingMore = false,
    onSelectFile = () => {},
    onPreviewFile = () => {},
    onContextMenu = () => {},
    onLoadMore = () => {},
    onRestore = () => {},
    onDelete = () => {},
    onPurgeTrash = () => {},
    onUpload = () => {},
  }: Props = $props();

  let displayFiles = $derived(isTrashView ? trash : files);
</script>

<div class="drive-content">
  <div class="content-main">
    {#if isTrashView}
      <TrashView
        files={trash}
        {isLoading}
        onRestore={onRestore}
        onDelete={onDelete}
        onEmptyTrash={onPurgeTrash}
        onContextMenu={onContextMenu}
      />
    {:else if isLoading}
      <div class="loading-panel">
        <div class="spinner"></div>
        <strong>Loading Drive</strong>
        <p>Fetching files, folders, and storage state.</p>
      </div>
    {:else if displayFiles.length === 0}
      <div class="empty-panel">
        <Icons.Cloud size={28} />
        <strong>No files in this view</strong>
        <p>Upload files or change the folder and lens filters.</p>
        <button type="button" class="accent-btn" onclick={onUpload}>
          <Icons.Upload size={15} />
          Upload into Drive
        </button>
      </div>
    {:else if viewMode === 'grid'}
      <div class="file-grid">
        {#each displayFiles as file (file.id)}
          <button
            type="button"
            class="file-card"
            class:selected={selectedFileId === file.id}
            onclick={() => onSelectFile?.(file)}
            ondblclick={() => onPreviewFile?.(file)}
            oncontextmenu={(event) => onContextMenu?.(event, file)}
          >
            <div class="file-surface">
              <FileIcon mimeType={file.mime_type} name={file.name} size={34} />
              {#if file.visibility === 'public'}
                <span class="inline-badge public">Public</span>
              {/if}
              {#if file.pinned_at}
                <span class="inline-badge pinned"><Icons.Pin size={10} /></span>
              {/if}
            </div>
            <div class="file-copy">
              <strong>{file.name}</strong>
              <span>{formatBytes(file.size_bytes)} · {formatRelative(file.updated_at || file.uploaded_at)}</span>
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <div class="file-list">
        <div class="list-head">
          <span>Name</span>
          <span>Size</span>
          <span>Updated</span>
          <span>Folder</span>
          <span>State</span>
        </div>

        {#each displayFiles as file (file.id)}
          <button
            type="button"
            class="list-row"
            class:selected={selectedFileId === file.id}
            onclick={() => onSelectFile?.(file)}
            ondblclick={() => onPreviewFile?.(file)}
            oncontextmenu={(event) => onContextMenu?.(event, file)}
          >
            <div class="name-cell">
              <FileIcon mimeType={file.mime_type} name={file.name} size={20} />
              <span>{file.name}</span>
            </div>
            <span>{formatBytes(file.size_bytes)}</span>
            <span>{formatRelative(file.updated_at || file.uploaded_at)}</span>
            <span>{file.folder_path || 'Bucket root'}</span>
            <div class="state-cell">
              {#if file.visibility === 'public'}
                <span class="inline-badge public">Public</span>
              {/if}
              {#if file.pinned_at}
                <span class="inline-badge pinned">Pinned</span>
              {/if}
              {#if file.visibility !== 'public' && !file.pinned_at}
                <span class="muted-state">Private</span>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    {/if}

    {#if !isTrashView && hasMore}
      <div class="load-more-row">
        <button type="button" class="ghost-btn" onclick={onLoadMore} disabled={isLoadingMore}>
          {#if isLoadingMore}
            <div class="spinner small"></div>
            Loading more
          {:else}
            <Icons.ChevronsDown size={14} />
            Load more
          {/if}
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .drive-content {
    flex: 1;
    overflow: hidden;
    display: grid;
    grid-template-columns: 1fr;
    min-height: 0;
  }

  .content-main {
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: 0 24px 24px;
  }

  .loading-panel,
  .empty-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 60px 24px;
    color: var(--muted);
    text-align: center;
  }

  .empty-panel strong,
  .loading-panel strong {
    color: var(--text);
  }

  .accent-btn {
    padding: 8px 14px;
    background: var(--blue);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: background 0.2s;
    margin-top: 12px;
  }

  .accent-btn:hover {
    background: var(--blue-hover);
  }

  .file-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 12px;
    padding: 12px 0;
  }

  .file-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-secondary);
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
    position: relative;
  }

  .file-card:hover {
    border-color: var(--blue-soft);
    background: rgba(59, 130, 246, 0.05);
  }

  .file-card.selected {
    border-color: var(--blue);
    background: rgba(59, 130, 246, 0.1);
  }

  .file-surface {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 80px;
    background: var(--bg);
    border-radius: 6px;
    position: relative;
  }

  .inline-badge {
    position: absolute;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    background: rgba(0, 0, 0, 0.6);
    color: white;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    bottom: 6px;
    right: 6px;
  }

  .inline-badge.public {
    background: rgba(91, 227, 154, 0.2);
    color: #5be39a;
  }

  .inline-badge.pinned {
    background: rgba(255, 193, 7, 0.2);
    color: #ffc107;
  }

  .file-copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .file-copy strong {
    font-size: 13px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-copy span {
    font-size: 11px;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .list-head {
    display: grid;
    grid-template-columns: 2fr 1fr 1.5fr 1.5fr 1fr;
    gap: 12px;
    padding: 12px 16px;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .list-row {
    display: grid;
    grid-template-columns: 2fr 1fr 1.5fr 1.5fr 1fr;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--hairline);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: background 0.15s;
    text-align: left;
    align-items: center;
  }

  .list-row:hover {
    background: var(--surface-2);
  }

  .list-row.selected {
    background: rgba(59, 130, 246, 0.1);
  }

  .list-row:last-child {
    border-bottom: none;
  }

  .name-cell,
  .state-cell {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    flex-wrap: wrap;
  }

  .name-cell span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .load-more-row {
    display: flex;
    justify-content: center;
    padding: 18px 0 0;
  }

  .ghost-btn {
    padding: 8px 12px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text-2);
    transition: all 0.2s;
  }

  .ghost-btn:hover:not(:disabled) {
    border-color: var(--blue);
    color: var(--blue);
  }

  .ghost-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner {
    width: 22px;
    height: 22px;
    border-radius: 999px;
    border: 2px solid rgba(255, 255, 255, 0.12);
    border-top-color: var(--blue);
    animation: spin 0.8s linear infinite;
  }

  .spinner.small {
    width: 14px;
    height: 14px;
    border-width: 2px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .muted-state {
    font-size: 11px;
    color: var(--muted);
  }

  @media (max-width: 768px) {
    .file-grid {
      grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    }

    .list-head {
      display: none;
    }

    .list-row {
      grid-template-columns: 1fr;
      gap: 8px;
    }
  }
</style>
