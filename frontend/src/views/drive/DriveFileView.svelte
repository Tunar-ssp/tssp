<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord } from '$lib/api';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import FileGrid from '$lib/components/FileGrid.svelte';

  interface Props {
    files: FileRecord[];
    viewMode: 'list' | 'grid';
    isLoading: boolean;
    isEmpty: boolean;
    onSelectFile: (file: FileRecord) => void;
    onContextMenu: (event: MouseEvent, file: FileRecord) => void;
  }

  let { files, viewMode, isLoading, isEmpty, onSelectFile, onContextMenu }: Props = $props();

  function handleFileKeydown(event: KeyboardEvent, file: FileRecord) {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    onSelectFile(file);
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
</script>

<div class="file-view">
  {#if isLoading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading files...</p>
    </div>
  {:else if isEmpty}
    <div class="empty-state">
      <Icons.File size={48} />
      <h3>No files</h3>
      <p>Upload files to get started</p>
    </div>
  {:else if viewMode === 'grid'}
    <FileGrid {files} onSelectFile={onSelectFile} />
  {:else}
    <div class="files-list">
      {#each files as file (file.id)}
        <div
          class="file-row"
          role="button"
          tabindex="0"
          onclick={() => onSelectFile(file)}
          onkeydown={(e) => handleFileKeydown(e, file)}
          oncontextmenu={(e) => onContextMenu(e, file)}
        >
          <div class="file-info">
            <div class="file-icon">
              <FileIcon mimeType={file.mime_type} name={file.name} size={20} />
            </div>
            <div class="file-details">
              <div class="file-name">{file.name}</div>
              <div class="file-meta">
                {formatBytes(file.size_bytes)} • {new Date((file.updated_at || file.uploaded_at) * 1000).toLocaleDateString()}
                {#if file.folder_path}
                  • {file.folder_path}
                {/if}
              </div>
            </div>
          </div>

          {#if file.pinned_at}
            <div class="file-badge">
              <Icons.Pin size={14} />
            </div>
          {/if}

          {#if file.visibility === 'public'}
            <div class="file-badge public">
              <Icons.Globe size={14} />
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .file-view {
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
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .files-list {
    display: flex;
    flex-direction: column;
  }

  .file-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-3);
    border-bottom: 1px solid var(--border);
    cursor: pointer;
    transition: background 0.15s;
  }

  .file-row:hover {
    background: var(--surface-2);
  }

  .file-info {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    flex: 1;
    min-width: 0;
  }

  .file-icon {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .file-details {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta {
    font-size: var(--fs-11);
    color: var(--muted);
    margin-top: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-badge {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--r-2);
    background: var(--surface-3);
    color: var(--text-2);
  }

  .file-badge.public {
    background: rgba(34, 197, 94, 0.1);
    color: var(--green);
  }
</style>
