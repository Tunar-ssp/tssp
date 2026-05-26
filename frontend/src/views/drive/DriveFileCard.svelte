<script lang="ts">
  /**
   * Reusable file card component for grid and list views
   * Extracted to reduce duplication between DriveGrid and DriveList
   */

  import * as Icons from 'lucide-svelte';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import type { FileRecord } from '$lib/api';
  import { generateImageThumbnail, isImageFile } from '$lib/services/imageThumbnailService';
  import { formatBytes, formatDate } from '$lib/utils';

  interface Props {
    file: FileRecord;
    isSelected?: boolean;
    layout?: 'grid' | 'list';
    onSelect?: (file: FileRecord) => void;
    onPreview?: (file: FileRecord) => void;
    onContextMenu?: (event: MouseEvent, file: FileRecord) => void;
    onDownload?: (file: FileRecord) => void;
    onShare?: (file: FileRecord) => void;
  }

  let {
    file,
    isSelected = false,
    layout = 'grid',
    onSelect,
    onPreview,
    onContextMenu,
    onDownload,
    onShare,
  }: Props = $props();

  let thumbnailUrl = $state<string | null>(null);
  let thumbnailError = $state(false);

  $effect(() => {
    if (isImageFile(file.mime_type) && layout === 'grid') {
      generateImageThumbnail(
        file.id,
        `/api/v1/files/${encodeURIComponent(file.id)}/content?disposition=inline`,
        'medium'
      )
        .then((url) => {
          thumbnailUrl = url;
        })
        .catch((error) => {
          console.warn('[DriveFileCard] Failed to generate thumbnail:', error);
          thumbnailError = true;
        });
    }
  });
</script>

{#if layout === 'grid'}
  <div
    class="file-card grid-card"
    class:selected={isSelected}
    onclick={() => onSelect?.(file)}
    oncontextmenu={(e) => onContextMenu?.(e, file)}
    role="button"
    tabindex="0"
  >
    <div class="card-thumbnail">
      {#if thumbnailUrl && !thumbnailError}
        <img src={thumbnailUrl} alt={file.name} class="thumbnail-image" />
      {:else}
        <FileIcon filename={file.name} mime={file.mime_type} size={48} />
      {/if}
    </div>
    <div class="card-info">
      <div class="card-name" title={file.name}>{file.name}</div>
      <div class="card-meta">
        <span class="file-size">{formatBytes(file.size_bytes)}</span>
        {#if file.visibility === 'public'}
          <span class="badge public">Public</span>
        {/if}
      </div>
    </div>
    <div class="card-actions">
      <button class="icon-btn" onclick={(e) => { e.stopPropagation(); onPreview?.(file); }} title="Preview">
        <Icons.Eye size={16} />
      </button>
      <button class="icon-btn" onclick={(e) => { e.stopPropagation(); onDownload?.(file); }} title="Download">
        <Icons.Download size={16} />
      </button>
    </div>
  </div>
{:else}
  <div
    class="file-card list-card"
    class:selected={isSelected}
    onclick={() => onSelect?.(file)}
    oncontextmenu={(e) => onContextMenu?.(e, file)}
    role="button"
    tabindex="0"
  >
    <div class="list-thumbnail">
      <FileIcon filename={file.name} mime={file.mime_type} size={24} />
    </div>
    <div class="list-info">
      <div class="list-name">{file.name}</div>
      <div class="list-meta">{formatDate(file.updated_at || file.created_at)}</div>
    </div>
    <div class="list-details">
      <span class="file-type">{file.mime_type}</span>
      <span class="file-size">{formatBytes(file.size_bytes)}</span>
    </div>
    <div class="list-actions">
      <button class="icon-btn" onclick={(e) => { e.stopPropagation(); onPreview?.(file); }} title="Preview">
        <Icons.Eye size={14} />
      </button>
      <button class="icon-btn" onclick={(e) => { e.stopPropagation(); onDownload?.(file); }} title="Download">
        <Icons.Download size={14} />
      </button>
    </div>
  </div>
{/if}

<style>
  .file-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-secondary);
    cursor: pointer;
    transition: all 0.2s;
  }

  .file-card:hover {
    border-color: rgba(59, 130, 246, 0.3);
    background-color: rgba(59, 130, 246, 0.05);
  }

  .file-card.selected {
    border-color: rgba(59, 130, 246, 0.6);
    background-color: rgba(59, 130, 246, 0.1);
  }

  .grid-card {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .card-thumbnail {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 80px;
    background: var(--bg);
    border-radius: 6px;
    overflow: hidden;
  }

  .thumbnail-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .card-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .card-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-meta {
    display: flex;
    gap: 6px;
    font-size: 11px;
    color: var(--muted);
    align-items: center;
  }

  .badge {
    padding: 2px 6px;
    border-radius: 3px;
    font-weight: 500;
  }

  .badge.public {
    background-color: rgba(91, 227, 154, 0.1);
    color: #5be39a;
  }

  .card-actions {
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .grid-card:hover .card-actions {
    opacity: 1;
  }

  .list-card {
    padding: 8px 12px;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .list-thumbnail {
    flex-shrink: 0;
  }

  .list-info {
    flex: 1;
    min-width: 0;
  }

  .list-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .list-meta {
    font-size: 11px;
    color: var(--muted);
  }

  .list-details {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: var(--muted);
    flex-shrink: 0;
  }

  .list-actions {
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .list-card:hover .list-actions {
    opacity: 1;
  }

  .icon-btn {
    padding: 4px;
    border: none;
    background: none;
    color: var(--muted);
    cursor: pointer;
    transition: color 0.2s;
    border-radius: 4px;
  }

  .icon-btn:hover {
    color: var(--text);
    background: rgba(0, 0, 0, 0.05);
  }

  @media (max-width: 768px) {
    .list-details {
      display: none;
    }
  }
</style>
