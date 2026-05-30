<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    currentFolder?: string;
    isTrashView?: boolean;
    fileCount?: number;
    visibleCount?: number;
    selectedCount?: number;
    onRefresh?: () => void;
    onUploadFolder?: () => void;
    onNewFolder?: () => void;
    onPurgeTrash?: () => void;
    onNavigateFolder?: (path: string) => void;
    onToggleSidebar?: () => void;
    sidebarOpen?: boolean;
    trashEmpty?: boolean;
  }

  let {
    currentFolder = '',
    isTrashView = false,
    fileCount = 0,
    visibleCount = 0,
    selectedCount = 0,
    onRefresh,
    onUploadFolder,
    onNewFolder,
    onPurgeTrash,
    onNavigateFolder,
    onToggleSidebar,
    sidebarOpen = true,
    trashEmpty = false,
  }: Props = $props();

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
  <div class="header-left">
    {#if !sidebarOpen}
      <button type="button" class="icon-btn" onclick={onToggleSidebar} title="Show sidebar">
        <Icons.PanelLeft size={15} />
      </button>
    {/if}
    <nav class="crumbs" aria-label="Folder path">
      <button
        type="button"
        class="crumb root"
        class:active={segments.length === 0}
        onclick={() => onNavigateFolder?.('')}
      >
        {#if isTrashView}
          <Icons.Trash2 size={14} />
          <span>Trash</span>
        {:else}
          <Icons.HardDrive size={14} />
          <span>Drive</span>
        {/if}
      </button>
      {#each segments as seg (seg.path)}
        <Icons.ChevronRight size={12} class="sep" />
        <button
          type="button"
          class="crumb"
          class:active={currentFolder === seg.path}
          onclick={() => onNavigateFolder?.(seg.path)}
        >
          {seg.name}
        </button>
      {/each}
    </nav>
    <span class="file-count">
      {#if selectedCount > 0}
        <span class="sel-count">{selectedCount} selected</span>
      {:else}
        {visibleCount} of {fileCount}
      {/if}
    </span>
  </div>

  <div class="header-right">
    <button type="button" class="icon-btn" onclick={onRefresh} title="Refresh">
      <Icons.RefreshCcw size={14} />
    </button>
    {#if isTrashView}
      <button type="button" class="ghost-btn danger" onclick={onPurgeTrash} disabled={trashEmpty}>
        <Icons.Trash2 size={13} />
        Purge expired
      </button>
    {:else}
      {#if onUploadFolder}
        <button type="button" class="icon-btn" onclick={onUploadFolder} title="Upload folder">
          <Icons.FolderUp size={14} />
        </button>
      {/if}
      <button type="button" class="ghost-btn" onclick={onNewFolder}>
        <Icons.FolderPlus size={13} />
        New folder
      </button>
    {/if}
  </div>
</header>

<style>
  .drive-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 16px 6px;
    border-bottom: 1px solid var(--border);
    min-height: 44px;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .crumbs {
    display: flex;
    align-items: center;
    gap: 2px;
    min-width: 0;
  }

  .crumb {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--text-2);
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: background 0.12s, color 0.12s;
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .crumb:hover { background: var(--surface-2); color: var(--text); }
  .crumb.active { color: var(--text); font-weight: 600; }
  .crumb.root { font-weight: 600; }

  .crumbs :global(.sep) { color: var(--dim); flex-shrink: 0; }

  .file-count {
    font-size: 12px;
    color: var(--muted);
    padding-left: 8px;
    border-left: 1px solid var(--hairline);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .sel-count { color: var(--blue); font-weight: 600; }

  .header-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .icon-btn {
    width: 32px;
    height: 32px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    border-radius: 7px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.12s;
  }
  .icon-btn:hover { border-color: var(--border-2); color: var(--text); background: var(--surface-2); }

  .ghost-btn {
    height: 32px;
    padding: 0 12px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    border-radius: 7px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: all 0.12s;
  }
  .ghost-btn:hover:not(:disabled) { border-color: var(--blue); color: var(--blue); }
  .ghost-btn.danger { color: var(--danger); border-color: rgba(239,68,68,0.3); }
  .ghost-btn.danger:hover:not(:disabled) { background: rgba(239,68,68,0.1); }
  .ghost-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  @media (max-width: 640px) {
    .ghost-btn { width: 32px; padding: 0; justify-content: center; }
    .file-count { display: none; }
  }
</style>
