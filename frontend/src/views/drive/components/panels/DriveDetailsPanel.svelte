<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord } from '$lib/api';

  interface Props {
    file: FileRecord | null;
    isLoading?: boolean;
    onToggleVisibility?: (isPublic: boolean) => Promise<void>;
    onShare?: () => void;
    onDownload?: (file: FileRecord) => void;
    onRename?: (file: FileRecord) => void;
  }

  let { file, isLoading = false, onToggleVisibility, onShare, onDownload, onRename }: Props = $props();

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

  // Human-friendly type label derived from file extension.
  const EXT_LABELS: Record<string, string> = {
    png: 'PNG image', jpg: 'JPEG image', jpeg: 'JPEG image', gif: 'GIF image',
    webp: 'WebP image', svg: 'SVG image', avif: 'AVIF image', bmp: 'Bitmap image',
    ico: 'Icon', heic: 'HEIC image',
    mp4: 'MP4 video', webm: 'WebM video', mov: 'QuickTime video', mkv: 'Matroska video',
    avi: 'AVI video', m4v: 'MP4 video',
    mp3: 'MP3 audio', wav: 'WAV audio', flac: 'FLAC audio', aac: 'AAC audio',
    m4a: 'M4A audio', ogg: 'OGG audio', opus: 'Opus audio',
    pdf: 'PDF document', doc: 'Word document', docx: 'Word document',
    xls: 'Spreadsheet', xlsx: 'Spreadsheet', csv: 'CSV', ppt: 'Presentation',
    pptx: 'Presentation', txt: 'Plain text', md: 'Markdown', json: 'JSON',
    zip: 'ZIP archive', tar: 'TAR archive', gz: 'Gzip archive', rar: 'RAR archive',
  };

  function friendlyType(target: FileRecord): string {
    const ext = (target.name.split('.').pop() || '').toLowerCase();
    if (EXT_LABELS[ext]) return EXT_LABELS[ext];
    if (ext) return `${ext.toUpperCase()} file`;
    const mime = target.mime_type || '';
    if (mime && mime !== 'application/octet-stream') return mime;
    return 'Unknown';
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
          <span class="value" title={file.name}>{file.name}</span>
        </div>
        <div class="detail-item">
          <span class="label">Size</span>
          <span class="value">{formatBytes(file.size_bytes)}</span>
        </div>
        <div class="detail-item">
          <span class="label">Type</span>
          <span class="value">{friendlyType(file)}</span>
        </div>
        <div class="detail-item">
          <span class="label">Folder</span>
          <span class="value">{file.folder_path || 'Root /'}</span>
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
        <button class="action-btn" onclick={() => onDownload?.(file)}>
          <Icons.Download size={14} />
          Download
        </button>
        <button class="action-btn" onclick={() => onRename?.(file)}>
          <Icons.Pencil size={14} />
          Rename
        </button>
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
      </div>
    </div>
  {/if}
</aside>

<style>
  .details-panel {
    flex-shrink: 0;
    width: 260px;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-left: 1px solid var(--border);
    overflow-y: auto;
  }

  .panel-header {
    padding: var(--s-4, 14px);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .panel-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3, 10px);
    color: var(--muted);
    padding: var(--s-4, 14px);
  }

  .details-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--s-4, 14px);
    padding: var(--s-4, 14px);
  }

  .detail-group {
    display: flex;
    flex-direction: column;
    gap: var(--s-2, 6px);
  }

  .group-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: var(--s-1, 3px);
    letter-spacing: 0.4px;
  }

  .detail-item {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    font-size: 12px;
    gap: var(--s-2, 6px);
  }

  .label {
    color: var(--muted);
    font-weight: 500;
    flex-shrink: 0;
  }

  .value {
    color: var(--text);
    text-align: right;
    word-break: break-word;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .visibility-badge,
  .status-badge {
    display: inline-block;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 11px;
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
    gap: 4px;
  }

  .tag {
    display: inline-block;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--surface-2);
    color: var(--text-2);
    font-size: 11px;
  }

  .actions-group {
    display: flex;
    flex-direction: column;
    gap: var(--s-2, 6px);
    margin-top: auto;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 7px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface-2);
    color: var(--text-2);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .action-btn:hover:not(:disabled) {
    background: var(--surface-3);
    color: var(--text);
    border-color: var(--border-2);
  }
  .action-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
