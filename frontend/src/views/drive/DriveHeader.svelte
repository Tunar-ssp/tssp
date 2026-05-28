<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    title?: string;
    currentFolder?: string;
    isTrashView?: boolean;
    fileCount?: number;
    visibleCount?: number;
    pinnedCount?: number;
    publicCount?: number;
    selectedCount?: number;
    imageCount?: number;
    videoCount?: number;
    documentCount?: number;
    onRefresh?: () => void;
    onUpload?: () => void;
    onUploadFolder?: () => void;
    onNewFolder?: () => void;
    onPurgeTrash?: () => void;
    onNavigateFolder?: (path: string) => void;
    trashEmpty?: boolean;
  }

  let {
    title = 'Cloud Drive',
    currentFolder = '',
    isTrashView = false,
    fileCount = 0,
    visibleCount = 0,
    pinnedCount = 0,
    publicCount = 0,
    selectedCount = 0,
    onRefresh,
    onUpload,
    onUploadFolder,
    onNewFolder,
    onPurgeTrash,
    onNavigateFolder,
    trashEmpty = false,
  }: Props = $props();

  // Build clickable breadcrumb segments from the current folder path.
  let segments = $derived(
    currentFolder
      ? currentFolder.split('/').filter(Boolean).map((name, i, arr) => ({
          name,
          path: arr.slice(0, i + 1).join('/'),
        }))
      : []
  );
</script>

<header class="drive-header">
  <nav class="crumbs" aria-label="Folder path">
    <button
      type="button"
      class="crumb"
      class:active={!isTrashView && segments.length === 0}
      onclick={() => onNavigateFolder?.('')}
    >
      <Icons.HardDrive size={14} />
      <span>{isTrashView ? 'Trash' : 'Drive'}</span>
    </button>
    {#if !isTrashView}
      {#each segments as seg (seg.path)}
        <Icons.ChevronRight size={13} class="crumb-sep" />
        <button
          type="button"
          class="crumb"
          class:active={currentFolder === seg.path}
          onclick={() => onNavigateFolder?.(seg.path)}
        >
          {seg.name}
        </button>
      {/each}
    {/if}

    <span class="crumb-stats">
      {#if selectedCount > 0}
        <span class="stat selected">{selectedCount} selected</span>
      {:else}
        <span class="stat">{visibleCount} of {fileCount}</span>
        {#if pinnedCount > 0}<span class="stat"><Icons.Pin size={11} /> {pinnedCount}</span>{/if}
        {#if publicCount > 0}<span class="stat"><Icons.Globe size={11} /> {publicCount}</span>{/if}
      {/if}
    </span>
  </nav>

  <div class="header-actions">
    <button type="button" class="ghost-btn icon-only" onclick={onRefresh} title="Refresh">
      <Icons.RefreshCcw size={15} />
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
          <span>New folder</span>
        </button>
      {/if}
      {#if onUploadFolder}
        <button type="button" class="ghost-btn icon-only" onclick={onUploadFolder} title="Upload an entire folder">
          <Icons.FolderUp size={14} />
        </button>
      {/if}
      <button type="button" class="accent-btn" onclick={onUpload}>
        <Icons.Upload size={15} />
        <span>Upload</span>
      </button>
    {/if}
  </div>
</header>

<style>
  .drive-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 0 12px;
  }

  .crumbs {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
    flex-wrap: wrap;
  }

  .crumb {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border: none;
    background: transparent;
    color: var(--text-2);
    border-radius: 8px;
    cursor: pointer;
    font-size: 15px;
    font-weight: 500;
    transition: background 0.15s, color 0.15s;
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .crumb:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .crumb.active {
    color: var(--text);
    font-weight: 600;
  }

  .crumbs :global(.crumb-sep) {
    color: var(--dim);
    flex-shrink: 0;
  }

  .crumb-stats {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-left: 10px;
    padding-left: 12px;
    border-left: 1px solid var(--hairline);
  }

  .stat {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .stat.selected {
    color: var(--blue);
    font-weight: 600;
  }

  .header-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .ghost-btn,
  .accent-btn,
  .danger-btn {
    height: 34px;
    padding: 0 12px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: all 0.15s;
  }

  .ghost-btn.icon-only,
  .accent-btn.icon-only {
    width: 34px;
    padding: 0;
    justify-content: center;
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

  @media (max-width: 760px) {
    .ghost-btn span,
    .accent-btn span {
      display: none;
    }

    .ghost-btn,
    .accent-btn {
      width: 34px;
      padding: 0;
      justify-content: center;
    }

    .crumb-stats {
      display: none;
    }
  }
</style>
