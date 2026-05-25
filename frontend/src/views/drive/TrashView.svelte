<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord } from '$lib/api';
  import FileIcon from '$lib/components/FileIcon.svelte';

  interface Props {
    files: FileRecord[];
    isLoading: boolean;
    onRestore: (file: FileRecord) => void;
    onDelete: (file: FileRecord) => void;
    onEmptyTrash: () => void;
    onContextMenu: (event: MouseEvent, file: FileRecord) => void;
  }

  let { files, isLoading, onRestore, onDelete, onEmptyTrash, onContextMenu }: Props = $props();

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
</script>

<div class="trash-view">
  {#if isLoading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading trash...</p>
    </div>
  {:else if files.length === 0}
    <div class="empty-state">
      <Icons.Trash2 size={48} />
      <h3>Trash is empty</h3>
      <p>Deleted files will appear here</p>
    </div>
  {:else}
    <div class="trash-list">
      {#each files as file (file.id)}
        <div
          class="trash-item"
          oncontextmenu={(e) => onContextMenu(e, file)}
          role="button"
          tabindex="0"
        >
          <div class="item-info">
            <div class="file-icon">
              <FileIcon mimeType={file.mime_type} name={file.name} size={20} />
            </div>
            <div class="item-details">
              <div class="item-name">{file.name}</div>
              <div class="item-meta">
                {formatBytes(file.size_bytes)} • {new Date((file.updated_at || file.uploaded_at) * 1000).toLocaleDateString()}
              </div>
            </div>
          </div>

          <div class="item-actions">
            <button
              type="button"
              class="action-btn restore"
              onclick={(e) => {
                e.stopPropagation();
                onRestore(file);
              }}
              title="Restore file"
            >
              <Icons.RotateCcw size={14} />
            </button>
            <button
              type="button"
              class="action-btn delete"
              onclick={(e) => {
                e.stopPropagation();
                onDelete(file);
              }}
              title="Delete permanently"
            >
              <Icons.Trash2 size={14} />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .trash-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: auto;
  }

  .loading-state,
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    color: var(--muted);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--surface-2);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.6s linear inline;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .trash-list {
    display: flex;
    flex-direction: column;
  }

  .trash-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-3);
    border-bottom: 1px solid var(--border);
    cursor: pointer;
    transition: background 0.15s;
  }

  .trash-item:hover {
    background: var(--surface-2);
  }

  .item-info {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    flex: 1;
    min-width: 0;
  }

  .file-icon {
    flex-shrink: 0;
  }

  .item-details {
    flex: 1;
    min-width: 0;
  }

  .item-name {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-meta {
    font-size: var(--fs-11);
    color: var(--muted);
    margin-top: 2px;
  }

  .item-actions {
    display: flex;
    gap: var(--s-2);
    flex-shrink: 0;
  }

  .action-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-2);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
  }

  .action-btn:hover {
    background: var(--surface-2);
  }

  .action-btn.delete:hover {
    border-color: var(--danger);
    background: rgba(239, 68, 68, 0.1);
    color: var(--danger);
  }

  .action-btn.restore:hover {
    border-color: var(--blue);
    background: rgba(59, 130, 246, 0.1);
    color: var(--blue);
  }
</style>
