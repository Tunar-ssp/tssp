<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { api } from '$lib/api';
  import { formatBytes, formatDate } from '$lib/utils';

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
  }: $$Props = $props();

  function fileExt(target: any): string {
    return (target?.name?.split('.').pop() || '').toLowerCase();
  }

  const IMG = new Set(['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'avif', 'bmp', 'ico']);
  const VID = new Set(['mp4', 'webm', 'mov', 'mkv', 'avi', 'm4v', 'ogv']);
  const AUD = new Set(['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a', 'opus']);
  const PDF = new Set(['pdf']);
  const TXT = new Set([
    'txt', 'md', 'mdx', 'json', 'yaml', 'yml', 'toml', 'ini', 'env',
    'ts', 'tsx', 'js', 'jsx', 'mjs', 'cjs', 'py', 'rs', 'go', 'java',
    'kt', 'swift', 'rb', 'php', 'c', 'h', 'cpp', 'hpp', 'cs',
    'html', 'htm', 'svelte', 'vue', 'css', 'scss', 'less',
    'sh', 'bash', 'zsh', 'fish', 'sql', 'graphql', 'proto', 'log',
  ]);

  function getPreviewLens() {
    if (!file) return 'details';
    const ext = fileExt(file);
    const mime = file.mime_type || '';
    if (mime.startsWith('image/') || IMG.has(ext)) return 'image';
    if (mime.startsWith('video/') || VID.has(ext)) return 'video';
    if (mime.startsWith('audio/') || AUD.has(ext)) return 'audio';
    if (mime === 'application/pdf' || PDF.has(ext)) return 'pdf';
    if (TXT.has(ext) || mime.startsWith('text/') || mime === 'application/json') return 'text';
    return 'details';
  }

  let previewLens = $state<'image' | 'video' | 'audio' | 'pdf' | 'text' | 'details'>('details');
  let ext = $derived(fileExt(file));
  let mime = $derived(file?.mime_type || '');
  let isImg = $derived(mime.startsWith('image/') || IMG.has(ext));
  let isVid = $derived(mime.startsWith('video/') || VID.has(ext));
  let isAud = $derived(mime.startsWith('audio/') || AUD.has(ext));
  let isPdf = $derived(mime === 'application/pdf' || PDF.has(ext));
  let textPreview = $state('');
  let textLoading = $state(false);
  let textError = $state('');
  let imageZoom = $state<number | 'fit'>(100);

  $effect(() => {
    if (isOpen && file) {
      previewLens = getPreviewLens() as 'image' | 'video' | 'audio' | 'pdf' | 'text' | 'details';
    }
  });

  $effect(() => {
    if (isOpen && file && previewLens === 'text' && isTextPreviewable(file)) {
      void loadTextPreview();
    }
  });

  function isTextPreviewable(target: any) {
    if (!target) return false;
    const ext = fileExt(target);
    if (TXT.has(ext)) return true;
    const mime = target.mime_type || '';
    return (
      mime.startsWith('text/') ||
      mime === 'application/json' ||
      mime.includes('javascript') ||
      mime.includes('typescript')
    );
  }


  async function loadTextPreview() {
    if (!file) return;
    textLoading = true;
    textError = '';
    try {
      const result = await api.previewFile(file.id, 'bytes=0-65535');
      textPreview = result.hasRange ? `${result.text}\n\n... preview truncated at 64 KiB` : result.text;
    } catch (e) {
      textError = e instanceof Error ? e.message : 'Could not load preview';
      textPreview = '';
    } finally {
      textLoading = false;
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }
</script>

{#if isOpen && file}
  <div
    class="preview-backdrop"
    role="presentation"
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={(e) => {
      if (e.key === 'Escape' && onClose) onClose();
    }}
  >
    <div class="preview-modal {className || ''}" role="dialog" aria-modal="true">
      <div class="preview-header">
        <h2 class="preview-title">{file.name}</h2>
        {#if onClose}
          <button type="button" class="preview-close" onclick={onClose} aria-label="Close">
            <Icons.X size={20} />
          </button>
        {/if}
      </div>

      <div class="preview-lenses">
        {#if isImg}
          <button type="button" class="lens-tab" class:active={previewLens === 'image'} onclick={() => (previewLens = 'image')}>
            <Icons.Image size={16} /> Image
          </button>
        {/if}
        {#if isVid}
          <button type="button" class="lens-tab" class:active={previewLens === 'video'} onclick={() => (previewLens = 'video')}>
            <Icons.Video size={16} /> Video
          </button>
        {/if}
        {#if isAud}
          <button type="button" class="lens-tab" class:active={previewLens === 'audio'} onclick={() => (previewLens = 'audio')}>
            <Icons.Music size={16} /> Audio
          </button>
        {/if}
        {#if isPdf}
          <button type="button" class="lens-tab" class:active={previewLens === 'pdf'} onclick={() => (previewLens = 'pdf')}>
            <Icons.FileText size={16} /> PDF
          </button>
        {/if}
        {#if isTextPreviewable(file)}
          <button type="button" class="lens-tab" class:active={previewLens === 'text'} onclick={() => (previewLens = 'text')}>
            <Icons.FileText size={16} /> Text
          </button>
        {/if}
        <button type="button" class="lens-tab" class:active={previewLens === 'details'} onclick={() => (previewLens = 'details')}>
          <Icons.Info size={16} /> Details
        </button>
      </div>

      <div class="preview-body">
        {#if previewLens === 'image' && isImg}
          <div class="preview-image-container">
            <div class="preview-image-toolbar">
              <button onclick={() => { if (typeof imageZoom === 'number') imageZoom = Math.max(50, imageZoom - 10); }} title="Zoom out">−</button>
              <span class="zoom-level">{typeof imageZoom === 'number' ? imageZoom + '%' : 'Fit'}</span>
              <button onclick={() => { if (typeof imageZoom === 'number') imageZoom = Math.min(200, imageZoom + 10); }} title="Zoom in">+</button>
              <button onclick={() => imageZoom = 100} title="Reset zoom">Reset</button>
              <button onclick={() => imageZoom = 'fit'} title="Fit to window">Fit</button>
            </div>
            <div class="preview-image" style={typeof imageZoom === 'number' ? `zoom: ${imageZoom}%` : 'object-fit: contain'}>
              <img
                src={`/api/v1/files/${encodeURIComponent(file.id)}/content?disposition=inline`}
                alt={file.name}
                onerror={(e) => {
                  (e.target as HTMLImageElement).style.display = 'none';
                }}
              />
            </div>
          </div>
        {:else if previewLens === 'video' && isVid}
          <div class="preview-video">
            <video
              src={`/api/v1/files/${encodeURIComponent(file.id)}/content?disposition=inline`}
              controls
              autoplay
            >
              <track kind="captions" />
              Your browser does not support the video tag.
            </video>
          </div>
        {:else if previewLens === 'audio' && isAud}
          <div class="preview-audio">
            <audio
              src={`/api/v1/files/${encodeURIComponent(file.id)}/content?disposition=inline`}
              controls
              autoplay
            >
              Your browser does not support audio playback.
            </audio>
          </div>
        {:else if previewLens === 'pdf' && isPdf}
          <div class="preview-pdf">
            <iframe
              src={`/api/v1/files/${encodeURIComponent(file.id)}/content?disposition=inline`}
              title={file.name}
            ></iframe>
          </div>
        {:else if previewLens === 'text' && isTextPreviewable(file)}
          <div class="preview-text">
            {#if textLoading}
              <pre>Loading preview...</pre>
            {:else if textError}
              <pre>Preview failed: {textError}</pre>
            {:else}
              <pre>{textPreview}</pre>
            {/if}
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
            type="button"
            class="preview-action primary"
            onclick={() => onDownload(file.id)}
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

  .lens-tab:disabled {
    cursor: not-allowed;
    color: var(--dim);
    opacity: 0.45;
  }

  .lens-tab:disabled:hover {
    background: transparent;
    color: var(--dim);
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
    flex-direction: column;
  }

  .preview-image-container {
    flex: 1;
    width: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .preview-image-toolbar {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-3) var(--s-4);
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
  }

  .preview-image-toolbar button {
    padding: 4px 10px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    border-radius: var(--r-2);
    cursor: pointer;
    font-size: var(--fs-12);
    transition: all 0.2s;
  }

  .preview-image-toolbar button:hover {
    background: rgba(110, 168, 255, 0.1);
    border-color: var(--blue);
    color: var(--text);
  }

  .zoom-level {
    min-width: 50px;
    text-align: center;
    font-family: var(--ff-mono);
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .preview-image {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    overflow: auto;
    padding: var(--s-4);
  }

  .preview-image img {
    max-width: 100%;
    max-height: 100%;
    border-radius: var(--r-2);
  }

  .preview-video {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    padding: var(--s-6);
    background: #000;
  }

  .preview-video video {
    max-width: 100%;
    max-height: 100%;
  }

  .preview-audio {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    padding: 40px;
  }
  .preview-audio audio { width: min(640px, 90%); }

  .preview-pdf {
    width: 100%;
    height: 100%;
  }
  .preview-pdf iframe {
    width: 100%;
    height: 100%;
    border: none;
    background: #fff;
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
