<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord } from '$lib/api';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import { formatBytes, formatRelative } from '$lib/utils';

  interface Props {
    files: FileRecord[];
    isLoading: boolean;
    onRestore: (file: FileRecord) => void;
    onDelete: (file: FileRecord) => void;
    onEmptyTrash: () => void;
    onContextMenu: (event: MouseEvent, file: FileRecord) => void;
  }

  let { files, isLoading, onRestore, onDelete, onEmptyTrash, onContextMenu }: Props = $props();
</script>

<div class="trash-view">
  {#if isLoading}
    <div class="state-panel">
      <div class="spinner"></div>
      <strong>Loading Trash</strong>
      <p>Fetching deleted files…</p>
    </div>
  {:else if files.length === 0}
    <div class="state-panel">
      <Icons.Trash2 size={40} />
      <strong>Trash is empty</strong>
      <p>Deleted files will appear here</p>
    </div>
  {:else}
    <div class="file-list">
      <div class="list-head">
        <span>Name</span>
        <span>Size</span>
        <span>Deleted</span>
        <span class="col-actions-head">
          <button type="button" class="purge-btn" onclick={onEmptyTrash} title="Purge all expired trash">
            <Icons.Trash2 size={12} />
            Purge expired
          </button>
        </span>
      </div>

      {#each files as file (file.id)}
        <div
          class="list-row"
          oncontextmenu={(e) => onContextMenu(e, file)}
          role="row"
          tabindex="0"
        >
          <div class="name-cell">
            <div class="thumb-wrap">
              <FileIcon mimeType={file.mime_type} name={file.name} size={20} />
            </div>
            <span class="row-name">{file.name}</span>
          </div>
          <span class="col-size">{formatBytes(file.size_bytes)}</span>
          <span class="col-date">{formatRelative(file.updated_at || file.uploaded_at)}</span>
          <div class="row-actions">
            <button
              type="button"
              title="Restore"
              onclick={(e) => { e.stopPropagation(); onRestore(file); }}
            >
              <Icons.RotateCcw size={13} />
            </button>
            <button
              type="button"
              title="Delete permanently"
              class="danger"
              onclick={(e) => { e.stopPropagation(); onDelete(file); }}
            >
              <Icons.Trash2 size={13} />
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
    overflow: hidden;
  }

  /* ── States ─────────────────────────────────────────── */
  .state-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 60px 24px;
    color: var(--muted);
    text-align: center;
    flex: 1;
  }
  .state-panel strong { color: var(--text); }
  .state-panel p { font-size: 13px; margin: 0; }

  /* ── List ────────────────────────────────────────────── */
  .file-list {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .list-head {
    display: grid;
    grid-template-columns: minmax(0,3fr) 80px 100px 80px;
    gap: 0;
    padding: 12px 16px;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    user-select: none;
    align-items: center;
  }

  .col-actions-head {
    display: flex;
    align-items: center;
    justify-content: flex-end;
  }

  .list-row {
    display: grid;
    grid-template-columns: minmax(0,3fr) 80px 100px 80px;
    gap: 0;
    padding: 0 16px;
    height: 44px;
    align-items: center;
    border-bottom: 1px solid var(--hairline);
    cursor: default;
    transition: background 0.1s;
    user-select: none;
  }
  .list-row:hover { background: var(--surface-2); }
  .list-row:last-child { border-bottom: none; }

  .col-size, .col-date { font-size: 12px; color: var(--muted); }

  .name-cell {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .thumb-wrap {
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    overflow: hidden;
    background: var(--surface);
  }

  .row-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    color: var(--text-2);
  }

  .row-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.12s;
    flex-shrink: 0;
  }
  .list-row:hover .row-actions { opacity: 1; }

  .row-actions button {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.1s, color 0.1s;
  }
  .row-actions button:hover {
    background: var(--surface-3);
    color: var(--blue);
  }
  .row-actions button.danger:hover {
    background: rgba(239, 68, 68, 0.12);
    color: var(--danger);
  }

  .purge-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 500;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s;
  }
  .purge-btn:hover {
    border-color: var(--danger);
    color: var(--danger);
    background: rgba(239, 68, 68, 0.06);
  }

  .spinner {
    width: 22px; height: 22px;
    border-radius: 999px;
    border: 2px solid rgba(255,255,255,0.12);
    border-top-color: var(--blue);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
