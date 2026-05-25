<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord } from '$lib/api';

  interface Props {
    file: FileRecord | null;
    isLoading?: boolean;
    onToggleVisibility?: (isPublic: boolean) => Promise<void>;
    onMove?: () => void;
    onShare?: () => void;
  }

  let { file, isLoading = false, onToggleVisibility, onMove, onShare }: Props = $props();

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }
</script>

<aside class="details-panel">
  <div class="panel-header">
    <h3>File Details</h3>
  </div>

  {#if !file}
    <div class="empty-state">
      <Icons.File size={40} />
      <p>Select a file to view details</p>
    </div>
  {:else}
    <div class="details-content">
      <div class="detail-group">
        <div class="detail-item">
          <span class="label">Name</span>
          <span class="value">{file.name}</span>
        </div>
        <div class="detail-item">
          <span class="label">Size</span>
          <span class="value">{formatBytes(file.size_bytes)}</span>
        </div>
        <div class="detail-item">
          <span class="label">Folder</span>
          <span class="value">{file.folder_path || 'Bucket root'}</span>
        </div>
        <div class="detail-item">
          <span class="label">Type</span>
          <span class="value">{file.mime_type}</span>
        </div>
        <div class="detail-item">
          <span class="label">Uploaded</span>
          <span class="value">{formatDate(file.uploaded_at)}</span>
        </div>
      </div>

      {#if file.tags && file.tags.length > 0}
        <div class="detail-group">
          <span class="group-label">Tags</span>
          <div class="tag-list">
            {#each file.tags as tag}
              <span class="tag">{tag}</span>
            {/each}
          </div>
        </div>
      {/if}

      <div class="detail-group">
        <div class="detail-item">
          <span class="label">Visibility</span>
          <div class="visibility-badge" class:public={file.visibility === 'public'}>
            {file.visibility === 'public' ? 'Public' : 'Private'}
          </div>
        </div>
        {#if file.pinned_at}
          <div class="detail-item">
            <span class="label">Status</span>
            <div class="status-badge">Pinned</div>
          </div>
        {/if}
      </div>

      <div class="actions-group">
        <button
          class="action-btn"
          onclick={() => onToggleVisibility?.(file.visibility !== 'public')}
          disabled={isLoading}
        >
          <Icons.Lock size={14} />
          {file.visibility === 'public' ? 'Make Private' : 'Make Public'}
        </button>
        <button class="action-btn" onclick={() => onShare?.()}>
          <Icons.Share2 size={14} />
          Share
        </button>
        <button class="action-btn" onclick={() => onMove?.()}>
          <Icons.FolderOpen size={14} />
          Move
        </button>
      </div>
    </div>
  {/if}
</aside>

<style>
  .details-panel {
    flex-shrink: 0;
    width: 280px;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-left: 1px solid var(--border);
    overflow-y: auto;
  }

  .panel-header {
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .panel-header h3 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    color: var(--muted);
    padding: var(--s-4);
  }

  .details-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
    padding: var(--s-4);
  }

  .detail-group {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .group-label {
    font-size: var(--fs-11);
    font-weight: 600;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: var(--s-1);
  }

  .detail-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: var(--fs-12);
    gap: var(--s-2);
  }

  .label {
    color: var(--muted);
    font-weight: 500;
  }

  .value {
    color: var(--text);
    text-align: right;
    word-break: break-word;
  }

  .visibility-badge,
  .status-badge {
    display: inline-block;
    padding: var(--s-1) var(--s-2);
    border-radius: var(--r-1);
    font-size: var(--fs-11);
    font-weight: 500;
    background: var(--surface-2);
    color: var(--text-2);
  }

  .visibility-badge.public {
    background: rgba(34, 197, 94, 0.1);
    color: var(--green);
  }

  .status-badge {
    background: rgba(59, 130, 246, 0.1);
    color: var(--blue);
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-1);
  }

  .tag {
    display: inline-block;
    padding: var(--s-1) var(--s-2);
    border-radius: var(--r-1);
    background: var(--surface-2);
    color: var(--text-2);
    font-size: var(--fs-11);
  }

  .actions-group {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    margin-top: auto;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
    color: var(--text-2);
    font-size: var(--fs-12);
    cursor: pointer;
    transition: all 0.2s;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--surface-3);
    color: var(--text);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
