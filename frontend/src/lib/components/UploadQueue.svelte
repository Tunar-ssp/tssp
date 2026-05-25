<script lang="ts">
  import { onDestroy } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import Bar from './Bar.svelte';
  import { uploadQueue } from '../stores/uploadQueue';

  interface Upload {
    id: string;
    name: string;
    progress: number;
    total: number;
    status: 'pending' | 'uploading' | 'success' | 'error';
    error?: string;
  }

  interface $$Props {
    uploads?: Upload[];
    onCancel?: (uploadId: string) => void;
    onRetry?: (uploadId: string) => void;
    class?: string;
  }

  let {
    uploads = [],
    onCancel,
    onRetry,
    class: className,
  } = $props<$$Props>();

  let queueState = $state<{ items: any[]; totalUploadingCount: number }>({
    items: [],
    totalUploadingCount: 0,
  });

  const unsubscribe = uploadQueue.subscribe((value) => {
    queueState = value;
  });
  onDestroy(unsubscribe);

  const queueUploads = $derived(
    uploads.length
      ? uploads
      : queueState.items.map((item) => ({
          id: item.id,
          name: item.filename,
          progress: item.uploadedBytes,
          total: item.fileSize,
          status:
            item.status === 'completed'
              ? 'success'
              : item.status === 'failed'
                ? 'error'
                : item.status === 'uploading'
                  ? 'uploading'
                  : 'pending',
          error: item.lastError,
        })),
  );

  function formatBytes(bytes: number) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  function getProgress(upload: Upload) {
    return (upload.progress / upload.total) * 100;
  }
</script>

{#if queueUploads.length > 0}
  <div class="upload-queue {className || ''}">
    <div class="queue-header">
      <h3>
        <Icons.Upload size={16} />
        Uploads
      </h3>
      <span class="queue-count">{queueUploads.length}</span>
    </div>

    <div class="queue-items">
      {#each queueUploads as upload (upload.id)}
        <div class="upload-item" class:error={upload.status === 'error'}>
          <div class="item-info">
            <div class="item-icon">
              {#if upload.status === 'uploading'}
                <Icons.Loader2 size={16} class="spin" />
              {:else if upload.status === 'success'}
                <Icons.CheckCircle2 size={16} />
              {:else if upload.status === 'error'}
                <Icons.AlertCircle size={16} />
              {:else}
                <Icons.File size={16} />
              {/if}
            </div>
            <div class="item-details">
              <div class="item-name">{upload.name}</div>
              <div class="item-progress">
                <Bar value={getProgress(upload)} tone="ok" />
                <span class="item-text">
                  {formatBytes(upload.progress)} / {formatBytes(upload.total)}
                </span>
              </div>
            </div>
          </div>

          <div class="item-actions">
            {#if upload.status === 'uploading'}
              <button
                class="action-btn"
                onclick={() => onCancel?.(upload.id)}
                title="Cancel"
              >
                <Icons.X size={14} />
              </button>
            {:else if upload.status === 'error' && onRetry}
              <button
                class="action-btn"
                onclick={() => onRetry(upload.id)}
                title="Retry"
              >
                <Icons.RotateCw size={14} />
              </button>
            {/if}
          </div>
        </div>

        {#if upload.status === 'error' && upload.error}
          <div class="item-error">{upload.error}</div>
        {/if}
      {/each}
    </div>
  </div>
{/if}

<style>
  .upload-queue {
    position: fixed;
    bottom: 120px;
    right: 20px;
    width: 100%;
    max-width: 340px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    box-shadow: var(--shadow-card);
    z-index: 50;
    overflow: hidden;
  }

  .queue-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-3) var(--s-4);
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
  }

  .queue-header h3 {
    margin: 0;
    display: flex;
    align-items: center;
    gap: var(--s-2);
    font-size: var(--fs-13);
    font-weight: 600;
    color: var(--text);
  }

  .queue-count {
    padding: 2px 8px;
    background: var(--blue);
    color: white;
    border-radius: var(--r-1);
    font-size: var(--fs-11);
    font-weight: 600;
  }

  .queue-items {
    max-height: 300px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .upload-item {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-3);
    border-bottom: 1px solid var(--hairline);
  }

  .upload-item:last-child {
    border-bottom: none;
  }

  .upload-item.error {
    background: rgba(255, 107, 107, 0.05);
  }

  .item-info {
    display: flex;
    align-items: flex-start;
    gap: var(--s-2);
    flex: 1;
    min-width: 0;
  }

  .item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    color: var(--text-2);
  }

  .item-icon :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .item-details {
    flex: 1;
    min-width: 0;
  }

  .item-name {
    font-size: var(--fs-12);
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-progress {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-top: 4px;
  }

  .item-progress :global(.bar) {
    flex: 1;
    height: 4px;
  }

  .item-text {
    font-size: var(--fs-10);
    color: var(--muted);
    white-space: nowrap;
  }

  .item-actions {
    display: flex;
    gap: var(--s-1);
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

  .item-error {
    padding: var(--s-2) var(--s-3) var(--s-2) calc(var(--s-3) + 24px + var(--s-2));
    font-size: var(--fs-11);
    color: var(--danger);
    background: rgba(255, 107, 107, 0.1);
  }
</style>
