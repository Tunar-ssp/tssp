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
  }: $$Props = $props();

  let queueState = $state<{ items: any[]; totalUploadingCount: number }>({
    items: [],
    totalUploadingCount: 0,
  });

  let isMinimized = $state(false);
  let isDismissed = $state(false);
  let previousCount = 0;

  const unsubscribe = uploadQueue.subscribe((value) => {
    queueState = value;
  });
  onDestroy(unsubscribe);

  const queueUploads = $derived(
    uploads.length
      ? uploads
      : queueState.items.map((item): Upload => {
          const mappedStatus: 'pending' | 'uploading' | 'success' | 'error' =
            item.status === 'completed'
              ? 'success'
              : item.status === 'failed'
                ? 'error'
                : item.status === 'uploading'
                  ? 'uploading'
                  : 'pending';
          return {
            id: item.id,
            name: item.filename,
            progress: item.uploadedBytes,
            total: item.fileSize,
            status: mappedStatus,
            error: item.lastError,
          };
        }),
  );

  let activeUploads = $derived(
    queueUploads.filter((upload) => upload.status === 'pending' || upload.status === 'uploading'),
  );

  let terminalUploads = $derived(
    queueUploads.filter((upload) => upload.status === 'success' || upload.status === 'error'),
  );

  $effect(() => {
    const count = queueUploads.length;
    if (count > previousCount) {
      isDismissed = false;
    }
    previousCount = count;
  });

  function formatBytes(bytes: number) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  function getProgress(upload: Upload) {
    if (!upload.total) return 0;
    return Math.min((upload.progress / upload.total) * 100, 100);
  }

  async function handleCancel(uploadId: string) {
    if (onCancel) {
      onCancel(uploadId);
      return;
    }
    await uploadQueue.cancelItem(uploadId);
  }

  async function handleRetry(uploadId: string) {
    if (onRetry) {
      onRetry(uploadId);
      return;
    }
    await uploadQueue.retryItem(uploadId);
  }

  async function handleRemove(uploadId: string) {
    await uploadQueue.removeItem(uploadId);
  }

  async function handleClearTerminal() {
    await uploadQueue.clearTerminal();
    if (activeUploads.length === 0) {
      isDismissed = true;
    }
  }
</script>

{#if queueUploads.length > 0 && !isDismissed}
  <div class="upload-queue {className || ''}" class:minimized={isMinimized}>
    <div class="queue-header">
      <h3>
        <Icons.Upload size={16} />
        Uploads
      </h3>
      <div class="header-actions">
        <span class="queue-count">{queueUploads.length}</span>
        {#if terminalUploads.length > 0}
          <button
            class="clear-btn"
            onclick={handleClearTerminal}
            title="Clear completed and failed uploads"
            aria-label="Clear completed and failed uploads"
          >
            <Icons.CheckCheck size={14} />
          </button>
        {/if}
        <button
          class="minimize-btn"
          onclick={() => (isMinimized = !isMinimized)}
          title={isMinimized ? 'Show uploads' : 'Hide uploads'}
          aria-label={isMinimized ? 'Show uploads' : 'Hide uploads'}
        >
          {#if isMinimized}
            <Icons.ChevronUp size={16} />
          {:else}
            <Icons.ChevronDown size={16} />
          {/if}
        </button>
        <button
          class="minimize-btn"
          onclick={() => (isDismissed = true)}
          title="Dismiss upload panel"
          aria-label="Dismiss upload panel"
        >
          <Icons.X size={16} />
        </button>
      </div>
    </div>

    {#if !isMinimized}
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
            {#if upload.status === 'uploading' || upload.status === 'pending'}
              <button
                class="action-btn"
                onclick={() => handleCancel(upload.id)}
                title="Cancel"
              >
                <Icons.X size={14} />
              </button>
            {:else if upload.status === 'error'}
              <button
                class="action-btn"
                onclick={() => handleRetry(upload.id)}
                title="Retry"
              >
                <Icons.RotateCw size={14} />
              </button>
              <button
                class="action-btn"
                onclick={() => handleRemove(upload.id)}
                title="Remove"
              >
                <Icons.Trash2 size={14} />
              </button>
            {:else if upload.status === 'success'}
              <button
                class="action-btn"
                onclick={() => handleRemove(upload.id)}
                title="Dismiss"
              >
                <Icons.Check size={14} />
              </button>
            {/if}
          </div>
        </div>

        {#if upload.status === 'error' && upload.error}
          <div class="item-error">{upload.error}</div>
        {/if}
      {/each}
    </div>
    {/if}
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

  .upload-queue.minimized .queue-header {
    border-bottom: none;
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

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .queue-count {
    padding: 2px 8px;
    background: var(--blue);
    color: white;
    border-radius: var(--r-1);
    font-size: var(--fs-11);
    font-weight: 600;
  }

  .minimize-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    border-radius: var(--r-1);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .minimize-btn:hover {
    background: var(--surface);
    color: var(--text);
  }

  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    background: rgba(91, 227, 154, 0.12);
    color: var(--green);
    cursor: pointer;
    border-radius: var(--r-1);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .clear-btn:hover {
    background: rgba(91, 227, 154, 0.18);
    color: var(--text);
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
