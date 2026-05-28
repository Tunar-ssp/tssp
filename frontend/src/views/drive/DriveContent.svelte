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
    selectedFileIds?: Set<string>;
    clipboardFileIds?: Set<string>;
    clipboardOperation?: 'copy' | 'cut' | null;
    hasMore?: boolean;
    isLoadingMore?: boolean;
    showThumbnails?: boolean;
    onSelectFile?: (file: FileRecord, event?: MouseEvent) => void;
    onPreviewFile?: (file: FileRecord) => void;
    onContextMenu?: (event: MouseEvent, file: FileRecord) => void;
    onLoadMore?: () => void;
    onRestore?: (file: FileRecord) => void;
    onDelete?: (file: FileRecord) => void;
    onPurgeTrash?: () => void;
    onUpload?: () => void;
    onDownload?: (file: FileRecord) => void;
    onCopy?: (file: FileRecord) => void;
    onDropFiles?: (fileIds: string[], targetFileId: string) => Promise<void>;
  }

  let {
    files = [],
    trash = [],
    isLoading = false,
    isTrashView = false,
    viewMode = 'grid',
    selectedFileId,
    selectedFileIds = new Set(),
    clipboardFileIds = new Set(),
    clipboardOperation = null,
    hasMore = false,
    isLoadingMore = false,
    showThumbnails = true,
    onSelectFile = () => {},
    onPreviewFile = () => {},
    onContextMenu = () => {},
    onLoadMore = () => {},
    onRestore = () => {},
    onDelete = () => {},
    onPurgeTrash = () => {},
    onUpload = () => {},
    onDownload = () => {},
    onCopy = () => {},
    onDropFiles = async (fileIds: string[], targetFileId: string) => {},
  }: Props = $props();

  let displayFiles = $derived(isTrashView ? trash : files);
  let draggedOverFile = $state<string | null>(null);
  let renamingFileId = $state<string | null>(null);
  let renameValue = $state<string>('');

  interface OnDragEvent extends DragEvent {
    dataTransfer: DataTransfer | null;
  }

  function startRename(file: FileRecord) {
    renamingFileId = file.id;
    renameValue = file.name;
  }

  function finishRename(file: FileRecord) {
    if (renameValue && renameValue !== file.name) {
      // This would trigger the rename callback
      // For now we'll just close the rename mode
    }
    renamingFileId = null;
    renameValue = '';
  }

  function handleRenameKeydown(e: KeyboardEvent, file: FileRecord) {
    if (e.key === 'Enter') {
      e.preventDefault();
      finishRename(file);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      renamingFileId = null;
      renameValue = '';
    }
  }

  function handleDragStart(e: DragEvent, file: FileRecord) {
    if (!e.dataTransfer) return;
    const filesToDrag = selectedFileIds.has(file.id) ? Array.from(selectedFileIds) : [file.id];
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('application/json', JSON.stringify({ fileIds: filesToDrag }));
  }

  function handleDragOver(e: DragEvent, file: FileRecord) {
    if (!e.dataTransfer) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    draggedOverFile = file.id;
  }

  function handleDragLeave(e: DragEvent, file: FileRecord) {
    if (draggedOverFile === file.id) {
      draggedOverFile = null;
    }
  }

  async function handleDrop(e: DragEvent, targetFile: FileRecord) {
    if (!e.dataTransfer) return;
    e.preventDefault();
    draggedOverFile = null;

    try {
      const data = e.dataTransfer.getData('application/json');
      const { fileIds } = JSON.parse(data) as { fileIds: string[] };
      if (fileIds.length > 0) {
        await onDropFiles?.(fileIds, targetFile.id);
      }
    } catch {
      // Ignore invalid drop data
    }
  }
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
          <div
            class="file-card"
            class:selected={selectedFileId === file.id}
            class:multi-selected={selectedFileIds.has(file.id)}
            class:clipboard-copy={clipboardFileIds.has(file.id) && clipboardOperation === 'copy'}
            class:clipboard-cut={clipboardFileIds.has(file.id) && clipboardOperation === 'cut'}
            class:drag-over={draggedOverFile === file.id}
            onclick={(e) => onSelectFile?.(file, e)}
            ondblclick={() => onPreviewFile?.(file)}
            oncontextmenu={(event) => onContextMenu?.(event, file)}
            ondragstart={(e) => handleDragStart(e, file)}
            ondragover={(e) => handleDragOver(e, file)}
            ondragleave={(e) => handleDragLeave(e, file)}
            ondrop={(e) => void handleDrop(e, file)}
            draggable="true"
            role="button"
            tabindex="0"
            onkeydown={(e) => {
              if (e.key === 'Enter') onSelectFile?.(file);
            }}
          >
            <div class="file-surface">
              {#if showThumbnails && file.name.match(/\.(jpg|jpeg|png|gif|webp|svg)$/i)}
                <img src={`/api/v1/files/${encodeURIComponent(file.id)}/content`} alt={file.name} class="file-thumbnail" loading="lazy" />
              {:else if showThumbnails && file.name.match(/\.(mp4|webm|mov|mkv|avi|flv|wmv)$/i)}
                <div class="video-thumbnail">
                  <Icons.Play size={24} />
                </div>
              {:else}
                <FileIcon mimeType={file.mime_type} name={file.name} size={34} />
              {/if}
              {#if file.visibility === 'public'}
                <span class="inline-badge public">Public</span>
              {/if}
              {#if file.pinned_at}
                <span class="inline-badge pinned"><Icons.Pin size={10} /></span>
              {/if}
              <div class="quick-actions">
                <button
                  type="button"
                  class="quick-action"
                  title="Preview"
                  onclick={(e) => {
                    e.stopPropagation();
                    onPreviewFile?.(file);
                  }}
                >
                  <Icons.Eye size={14} />
                </button>
                <button
                  type="button"
                  class="quick-action"
                  title="Download"
                  onclick={(e) => {
                    e.stopPropagation();
                    onDownload?.(file);
                  }}
                >
                  <Icons.Download size={14} />
                </button>
                <button
                  type="button"
                  class="quick-action"
                  title="Copy"
                  onclick={(e) => {
                    e.stopPropagation();
                    onCopy?.(file);
                  }}
                >
                  <Icons.Copy size={14} />
                </button>
              </div>
            </div>
            <div class="file-copy">
              {#if renamingFileId === file.id}
                <input
                  type="text"
                  class="rename-input"
                  value={renameValue}
                  onchange={(e) => renameValue = e.currentTarget.value}
                  onblur={() => finishRename(file)}
                  onkeydown={(e) => handleRenameKeydown(e, file)}
                  autofocus
                />
              {:else}
                <strong
                  ondblclick={() => startRename(file)}
                  class="filename-text"
                >
                  {file.name}
                </strong>
              {/if}
              <span>{formatBytes(file.size_bytes)} · {formatRelative(file.updated_at || file.uploaded_at)}</span>
            </div>
          </div>
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
            class:multi-selected={selectedFileIds.has(file.id)}
            class:clipboard-copy={clipboardFileIds.has(file.id) && clipboardOperation === 'copy'}
            class:clipboard-cut={clipboardFileIds.has(file.id) && clipboardOperation === 'cut'}
            class:drag-over={draggedOverFile === file.id}
            onclick={(e) => onSelectFile?.(file, e)}
            ondblclick={() => onPreviewFile?.(file)}
            oncontextmenu={(event) => onContextMenu?.(event, file)}
            ondragstart={(e) => handleDragStart(e, file)}
            ondragover={(e) => handleDragOver(e, file)}
            ondragleave={(e) => handleDragLeave(e, file)}
            ondrop={(e) => void handleDrop(e, file)}
            draggable="true"
          >
            <div
              class="list-row-content"
            >
              <div class="name-cell">
                <FileIcon mimeType={file.mime_type} name={file.name} size={20} />
                {#if renamingFileId === file.id}
                  <input
                    type="text"
                    class="rename-input"
                    value={renameValue}
                    onchange={(e) => renameValue = e.currentTarget.value}
                    onblur={() => finishRename(file)}
                    onkeydown={(e) => handleRenameKeydown(e, file)}
                    autofocus
                  />
                {:else}
                  <span
                    ondblclick={() => startRename(file)}
                    class="filename-text"
                  >
                    {file.name}
                  </span>
                {/if}
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
            </div>
            <div class="list-row-actions">
              <div
                class="row-action"
                role="button"
                tabindex="0"
                title="Preview"
                onclick={(e) => {
                  e.stopPropagation();
                  onPreviewFile?.(file);
                }}
                onkeydown={(e) => {
                  if (e.key === 'Enter') onPreviewFile?.(file);
                }}
              >
                <Icons.Eye size={14} />
              </div>
              <div
                class="row-action"
                role="button"
                tabindex="0"
                title="Download"
                onclick={(e) => {
                  e.stopPropagation();
                  onDownload?.(file);
                }}
                onkeydown={(e) => {
                  if (e.key === 'Enter') onDownload?.(file);
                }}
              >
                <Icons.Download size={14} />
              </div>
              <div
                class="row-action"
                role="button"
                tabindex="0"
                title="Copy"
                onclick={(e) => {
                  e.stopPropagation();
                  onCopy?.(file);
                }}
                onkeydown={(e) => {
                  if (e.key === 'Enter') onCopy?.(file);
                }}
              >
                <Icons.Copy size={14} />
              </div>
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
    user-select: none;
  }

  .file-card:hover {
    border-color: var(--blue-soft);
    background: rgba(59, 130, 246, 0.05);
  }

  .file-card.selected {
    border-color: var(--blue);
    background: rgba(59, 130, 246, 0.1);
  }

  .file-card.multi-selected {
    border-color: var(--blue);
    background: rgba(59, 130, 246, 0.15);
    box-shadow: inset 0 0 0 2px rgba(59, 130, 246, 0.4);
  }

  .file-card.clipboard-copy {
    border-color: var(--green-soft, #5be39a);
    background: rgba(91, 227, 154, 0.08);
    opacity: 0.85;
  }

  .file-card.clipboard-cut {
    border-color: var(--orange-soft, #ff8a3d);
    background: rgba(255, 138, 61, 0.08);
    opacity: 0.7;
  }

  .file-card.drag-over {
    border-color: var(--blue);
    background: rgba(59, 130, 246, 0.15);
    box-shadow: inset 0 0 0 2px var(--blue);
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

  .quick-actions {
    position: absolute;
    bottom: 6px;
    left: 6px;
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity 150ms;
  }

  .file-card:hover .quick-actions {
    opacity: 1;
  }

  .quick-action {
    padding: 6px;
    background: rgba(0, 0, 0, 0.6);
    border: none;
    border-radius: 4px;
    color: white;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: background 150ms;
  }

  .quick-action:hover {
    background: rgba(0, 0, 0, 0.8);
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

  .filename-text {
    cursor: text;
    transition: color 150ms;
  }

  .filename-text:hover {
    color: var(--blue);
  }

  .rename-input {
    width: 100%;
    padding: 2px 4px;
    font-size: 13px;
    border: 1px solid var(--blue);
    border-radius: 3px;
    background: var(--bg);
    color: var(--text);
    font-weight: 500;
  }

  .rename-input:focus {
    outline: none;
    border-color: var(--blue);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
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
    display: flex;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--hairline);
    background: transparent;
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

  .list-row.multi-selected {
    background: rgba(59, 130, 246, 0.15);
    border-left: 3px solid var(--blue);
    padding-left: 13px;
  }

  .list-row.clipboard-copy {
    background: rgba(91, 227, 154, 0.08);
    border-left: 3px solid #5be39a;
    padding-left: 13px;
    opacity: 0.85;
  }

  .list-row.clipboard-cut {
    background: rgba(255, 138, 61, 0.08);
    border-left: 3px solid #ff8a3d;
    padding-left: 13px;
    opacity: 0.7;
  }

  .list-row.drag-over {
    background: rgba(59, 130, 246, 0.15);
    border-left: 3px solid var(--blue);
    padding-left: 13px;
  }

  .list-row:last-child {
    border-bottom: none;
  }

  .list-row-content {
    display: grid;
    grid-template-columns: 2fr 1fr 1.5fr 1.5fr 1fr;
    gap: 12px;
    flex: 1;
    min-width: 0;
    padding: 0;
    text-align: left;
    align-items: center;
  }

  .list-row-actions {
    display: flex;
    gap: 8px;
    opacity: 0;
    transition: opacity 150ms;
    flex-shrink: 0;
  }

  .list-row:hover .list-row-actions {
    opacity: 1;
  }

  .row-action {
    padding: 6px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-2);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: all 150ms;
  }

  .row-action:hover {
    border-color: var(--blue);
    color: var(--blue);
    background: rgba(59, 130, 246, 0.05);
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

  .file-thumbnail {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 4px;
  }

  .video-thumbnail {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, rgba(30, 30, 30, 0.8), rgba(50, 50, 50, 0.8));
    color: var(--text-2);
    border-radius: 4px;
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
