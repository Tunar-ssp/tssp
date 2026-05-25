<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    file?: any;
    isOpen?: boolean;
    onClose?: () => void;
    onDownload?: (fileId: string) => void;
    class?: string;
  }

  let {
    file,
    isOpen = false,
    onClose,
    onDownload,
    class: className,
  } = $props<$$Props>();

  function canPreview(mimeType: string) {
    return (
      mimeType.startsWith('image/') ||
      mimeType.startsWith('text/') ||
      mimeType === 'application/json' ||
      mimeType === 'application/pdf'
    );
  }

  function getPreviewLens() {
    if (!file) return 'details';
    if (file.mime_type.startsWith('image/')) return 'image';
    if (file.mime_type.startsWith('text/')) return 'text';
    return 'details';
  }

  let previewLens = $derived(getPreviewLens());

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }

  function formatBytes(bytes: number) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  }
</script>

{#if isOpen && file}
  <div class="preview-backdrop" on:click={handleBackdropClick}>
    <div class="preview-modal {className || ''}">
      <div class="preview-header">
        <h2 class="preview-title">{file.name}</h2>
        {#if onClose}
          <button class="preview-close" on:click={onClose} aria-label="Close">
            <Icons.X size={20} />
          </button>
        {/if}
      </div>

      <div class="preview-lenses">
        <button
          class="lens-tab"
          class:active={previewLens === 'image'}
          on:click={() => (previewLens = 'image')}
        >
          <Icons.Image size={16} />
          Image
        </button>
        <button
          class="lens-tab"
          class:active={previewLens === 'text'}
          on:click={() => (previewLens = 'text')}
        >
          <Icons.FileText size={16} />
          Text
        </button>
        <button
          class="lens-tab"
          class:active={previewLens === 'details'}
          on:click={() => (previewLens = 'details')}
        >
          <Icons.Info size={16} />
          Details
        </button>
      </div>

      <div class="preview-body">
        {#if previewLens === 'image' && file.mime_type.startsWith('image/')}
          <div class="preview-image">
            <img
              src={`/api/files/${file.id}/content`}
              alt={file.name}
              on:error={(e) => {
                (e.target as HTMLImageElement).style.display = 'none';
              }}
            />
          </div>
        {:else if previewLens === 'text' && file.mime_type.startsWith('text/')}
          <div class="preview-text">
            <pre>{`Loading preview...`}</pre>
          </div>
        {:else}
          <div class="preview-details">
            <div class="detail-row">
              <span class="label">Name</span>
              <span class="value">{file.name}</span>
            </div>
            <div class="detail-row">
              <span class="label">Size</span>
              <span class="value">{formatBytes(file.size_bytes)}</span>
            </div>
            <div class="detail-row">
              <span class="label">Type</span>
              <span class="value">{file.mime_type}</span>
            </div>
            <div class="detail-row">
              <span class="label">Created</span>
              <span class="value">{formatDate(file.created_at || file.uploaded_at)}</span>
            </div>
            <div class="detail-row">
              <span class="label">Modified</span>
              <span class="value">{formatDate(file.updated_at || file.uploaded_at)}</span>
            </div>
            {#if file.pinned_at}
              <div class="detail-row">
                <span class="label">Status</span>
                <span class="value">
                  <Icons.Pin size={14} style="color: var(--orange)" />
                  Pinned
                </span>
              </div>
            {/if}
            {#if file.public}
              <div class="detail-row">
                <span class="label">Status</span>
                <span class="value">
                  <Icons.Share2 size={14} style="color: var(--green)" />
                  Public
                </span>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <div class="preview-footer">
        {#if onDownload}
          <button
            class="preview-action primary"
            on:click={() => onDownload(file.id)}
          >
            <Icons.Download size={16} />
            Download
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .preview-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
    animation: fadeIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .preview-modal {
    width: 90%;
    max-width: 800px;
    height: 90%;
    max-height: 700px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    animation: modalSlideIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes modalSlideIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-6);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .preview-title {
    margin: 0;
    font-size: var(--fs-18);
    font-weight: 600;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preview-close {
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-2);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .preview-close:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .preview-lenses {
    display: flex;
    gap: var(--s-1);
    padding: var(--s-3) var(--s-6);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    overflow-x: auto;
  }

  .lens-tab {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-4);
    border: none;
    background: transparent;
    color: var(--text-2);
    border-radius: var(--r-2);
    cursor: pointer;
    font-size: var(--fs-12);
    font-weight: 500;
    transition: all var(--duration-quick) var(--ease-smooth);
    white-space: nowrap;
  }

  .lens-tab:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .lens-tab.active {
    background: var(--blue-subtle);
    color: var(--blue);
  }

  .preview-body {
    flex: 1;
    overflow: auto;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .preview-image {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    padding: var(--s-6);
  }

  .preview-image img {
    max-width: 100%;
    max-height: 100%;
    border-radius: var(--r-2);
  }

  .preview-text {
    width: 100%;
    padding: var(--s-6);
    overflow: auto;
  }

  .preview-text pre {
    margin: 0;
    font-family: var(--ff-mono);
    font-size: var(--fs-12);
    color: var(--text-2);
    line-height: var(--lh-normal);
  }

  .preview-details {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
    padding: var(--s-6);
  }

  .detail-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
    padding: var(--s-3);
    background: var(--surface-2);
    border-radius: var(--r-2);
  }

  .detail-row .label {
    font-weight: 500;
    color: var(--text-2);
    min-width: 100px;
  }

  .detail-row .value {
    font-family: var(--ff-mono);
    color: var(--text);
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .preview-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--s-3);
    padding: var(--s-4) var(--s-6);
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }

  .preview-action {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-4);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
    font-size: var(--fs-13);
    font-weight: 500;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .preview-action:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .preview-action.primary {
    background: var(--blue);
    color: #0a1228;
    border-color: var(--blue);
  }

  .preview-action.primary:hover {
    opacity: 0.9;
  }
</style>
