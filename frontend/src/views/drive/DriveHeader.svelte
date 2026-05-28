<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    title?: string;
    subtitle?: string;
    currentFolder?: string;
    isTrashView?: boolean;
    fileCount?: number;
    visibleCount?: number;
    pinnedCount?: number;
    publicCount?: number;
    selectedCount?: number;
    usedBytes?: number;
    imageCount?: number;
    videoCount?: number;
    documentCount?: number;
    onRefresh?: () => void;
    onUpload?: () => void;
    onNewFolder?: () => void;
    onPurgeTrash?: () => void;
    trashEmpty?: boolean;
  }

  let {
    title = 'Cloud Drive',
    subtitle,
    currentFolder,
    isTrashView = false,
    fileCount = 0,
    visibleCount = 0,
    pinnedCount = 0,
    publicCount = 0,
    selectedCount = 0,
    usedBytes = 0,
    imageCount = 0,
    videoCount = 0,
    documentCount = 0,
    onRefresh,
    onUpload,
    onNewFolder,
    onPurgeTrash,
    trashEmpty = false,
  }: Props = $props();

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }
</script>

<header class="drive-header">
  <div>
    <div class="breadcrumbs">
      <span>Cloud Drive</span>
      {#if currentFolder}
        <Icons.ChevronRight size={12} />
        <span>{currentFolder}</span>
      {/if}
    </div>
    <h1>{title}</h1>
    <p>
      {#if isTrashView}
        Restore or permanently purge deleted objects.
      {:else}
        {subtitle || 'Browse, upload, preview, share, and manage your local cloud objects.'}
      {/if}
    </p>
  </div>

  <div class="header-actions">
    <button type="button" class="ghost-btn" onclick={onRefresh}>
      <Icons.RefreshCcw size={14} />
      Refresh
    </button>
    {#if isTrashView}
      <button type="button" class="danger-btn" onclick={onPurgeTrash} disabled={trashEmpty}>
        <Icons.Trash2 size={14} />
        Purge expired
      </button>
    {:else}
      {#if onNewFolder}
        <button type="button" class="ghost-btn" onclick={onNewFolder} title="New folder">
          <Icons.FolderPlus size={14} />
          New folder
        </button>
      {/if}
      <button type="button" class="accent-btn" onclick={onUpload}>
        <Icons.Upload size={15} />
        Upload files
      </button>
    {/if}
  </div>
</header>

<div class="summary-grid">
  {#if selectedCount > 0}
    <article class="summary-card selected">
      <span>Selected</span>
      <strong>{selectedCount}</strong>
    </article>
  {/if}
  <article class="summary-card">
    <span>Objects</span>
    <strong>{fileCount}</strong>
  </article>
  <article class="summary-card">
    <span>Visible here</span>
    <strong>{visibleCount}</strong>
  </article>
  {#if !isTrashView && (imageCount > 0 || videoCount > 0 || documentCount > 0)}
    <article class="summary-card">
      <span>File Types</span>
      <div class="file-type-icons">
        {#if imageCount > 0}
          <span title="{imageCount} images">
            <Icons.Image size={14} />
            {imageCount}
          </span>
        {/if}
        {#if videoCount > 0}
          <span title="{videoCount} videos">
            <Icons.Video size={14} />
            {videoCount}
          </span>
        {/if}
        {#if documentCount > 0}
          <span title="{documentCount} documents">
            <Icons.FileText size={14} />
            {documentCount}
          </span>
        {/if}
      </div>
    </article>
  {/if}
  <article class="summary-card">
    <span>Pinned</span>
    <strong>{pinnedCount}</strong>
  </article>
  <article class="summary-card">
    <span>Public</span>
    <strong>{publicCount}</strong>
  </article>
</div>

<style>
  .drive-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    padding: 0 0 24px;
  }

  .breadcrumbs {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--muted);
    margin-bottom: 8px;
  }

  h1 {
    margin: 0 0 4px;
    font-size: 28px;
    font-weight: 600;
    color: var(--text);
  }

  p {
    margin: 0;
    font-size: 14px;
    color: var(--text-2);
  }

  .header-actions {
    display: flex;
    gap: 12px;
    flex-shrink: 0;
  }

  .ghost-btn,
  .accent-btn,
  .danger-btn {
    padding: 8px 12px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: all 0.2s;
  }

  .ghost-btn {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text-2);
  }

  .ghost-btn:hover {
    border-color: var(--blue);
    color: var(--blue);
  }

  .accent-btn {
    background: var(--blue);
    color: white;
  }

  .accent-btn:hover {
    background: var(--blue-hover);
  }

  .danger-btn {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .danger-btn:hover:not(:disabled) {
    background: var(--danger);
    color: white;
  }

  .danger-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    padding: 0 0 24px;
  }

  .summary-card {
    padding: 16px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .summary-card span {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
  }

  .summary-card strong {
    font-size: 20px;
    color: var(--text);
  }

  .summary-card.selected {
    background: rgba(59, 130, 246, 0.1);
    border-color: var(--blue);
  }

  .file-type-icons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .file-type-icons span {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 12px;
    color: var(--text);
    padding: 2px 6px;
    background: var(--surface);
    border-radius: 3px;
  }

  @media (max-width: 1200px) {
    .summary-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (max-width: 760px) {
    .drive-header {
      flex-direction: column;
      gap: 12px;
      padding: 0;
    }

    .header-actions {
      width: 100%;
      flex-direction: column;
    }

    .header-actions button {
      width: 100%;
      justify-content: center;
    }

    .summary-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    h1 {
      font-size: 20px;
    }
  }
</style>
