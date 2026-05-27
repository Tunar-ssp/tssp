<script lang="ts">
  import { get } from 'svelte/store';
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import {
    api,
    type FileRecord,
    type FolderEntry,
    type VisibilityResponse,
  } from '$lib/api';
  import { driveStateManager } from '$lib/services/driveStateService';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { isDriveFolder, createWorkspaceFromFolder } from '$lib/services/driveWorkspaceIntegration';
  import FilePreviewModal from '$lib/components/FilePreviewModal.svelte';
  import SharingModal from '$lib/components/SharingModal.svelte';
  import MoveFileDialog from './MoveFileDialog.svelte';
  import DriveDetailsPanel from './DriveDetailsPanel.svelte';
  import TrashView from './TrashView.svelte';
  import { consumeSelectionIntent, preferences, setDefaultDriveView, selectionIntent } from '$lib/stores/ui';
  import { error, success, info } from '$lib/stores/notifications';
  import { formatBytes, formatRelative } from '$lib/utils';

  type DriveLens = 'all' | 'images' | 'videos' | 'documents' | 'public' | 'trash';
  type Status = Awaited<ReturnType<typeof api.getStatus>>;

  let files = $state<FileRecord[]>([]);
  let trash = $state<FileRecord[]>([]);
  let folderEntries = $state<FolderEntry[]>([]);
  let status = $state<Status | null>(null);
  let selectedFile = $state<FileRecord | null>(null);
  let previewFile = $state<FileRecord | null>(null);
  let shareFile = $state<FileRecord | null>(null);
  let moveDialogFile = $state<FileRecord | null>(null);
  let isMoveDialogOpen = $state(false);
  let isMoving = $state(false);
  let isLoading = $state(true);
  let isLoadingMore = $state(false);
  let trashLoading = $state(false);
  let nextCursor = $state<string | undefined>(undefined);
  let hasMore = $state(false);
  let currentFolder = $state('');
  let filterQuery = $state('');
  let activeLens = $state<DriveLens>('all');
  let viewMode = $state<'grid' | 'list'>(get(preferences).defaultDriveView || 'grid');
  let contextMenu = $state({
    visible: false,
    x: 0,
    y: 0,
    file: null as FileRecord | null,
  });

  onMount(() => {
    const handleExternalRefresh = () => {
      void loadLibrary(true);
      if (activeLens === 'trash') {
        void loadTrash();
      }
    };

    if (typeof document !== 'undefined') {
      document.addEventListener('tssp:drive-refresh', handleExternalRefresh as EventListener);
    }

    void loadLibrary(true).then(() => consumeIntent());

    return () => {
      if (typeof document !== 'undefined') {
        document.removeEventListener('tssp:drive-refresh', handleExternalRefresh as EventListener);
      }
    };
  });

  let isTrashView = $derived(activeLens === 'trash');

  let filteredLibraryFiles = $derived.by(() =>
    files
      .filter((file) => {
        if (currentFolder && (file.folder_path || '') !== currentFolder) return false;
        if (activeLens === 'images' && !file.mime_type.startsWith('image/')) return false;
        if (activeLens === 'videos' && !file.mime_type.startsWith('video/')) return false;
        if (
          activeLens === 'documents' &&
          (file.mime_type.startsWith('image/') ||
            file.mime_type.startsWith('video/') ||
            file.mime_type.includes('javascript') ||
            file.mime_type.includes('json') ||
            file.mime_type.startsWith('text/'))
        ) {
          return false;
        }
        if (activeLens === 'public' && file.visibility !== 'public') return false;

        const query = filterQuery.trim().toLowerCase();
        if (!query) return true;
        return (
          file.name.toLowerCase().includes(query) ||
          (file.folder_path || '').toLowerCase().includes(query) ||
          (file.tags || []).some((tag) => tag.toLowerCase().includes(query))
        );
      })
      .sort((left, right) => {
        const leftScore = left.pinned_at ? 1_000_000_000_000 + left.pinned_at : left.updated_at || left.uploaded_at;
        const rightScore = right.pinned_at ? 1_000_000_000_000 + right.pinned_at : right.updated_at || right.uploaded_at;
        return rightScore - leftScore;
      })
  );

  let filteredTrash = $derived.by(() =>
    trash
      .filter((file) => {
        const query = filterQuery.trim().toLowerCase();
        if (!query) return true;
        return (
          file.name.toLowerCase().includes(query) ||
          (file.folder_path || '').toLowerCase().includes(query) ||
          (file.tags || []).some((tag) => tag.toLowerCase().includes(query))
        );
      })
      .sort((left, right) => (right.updated_at || right.uploaded_at) - (left.updated_at || left.uploaded_at))
  );

  function dedupeFiles(items: FileRecord[]) {
    return Array.from(new Map(items.map((file) => [file.id, file])).values());
  }

  async function loadLibrary(reset = false) {
    if (reset) {
      isLoading = true;
    } else {
      isLoadingMore = true;
    }

    try {
      if (reset) {
        const [fileData, folderData, statusData] = await Promise.all([
          api.listFiles(72),
          api.listFolders(),
          api.getStatus(),
        ]);

        files = fileData.files || [];
        nextCursor = fileData.nextCursor;
        hasMore = !!fileData.nextCursor;
        folderEntries = folderData.folders || [];
        status = statusData;
      } else if (nextCursor) {
        const fileData = await api.listFiles(72, nextCursor);
        files = dedupeFiles([...files, ...(fileData.files || [])]);
        nextCursor = fileData.nextCursor;
        hasMore = !!fileData.nextCursor;
      }
    } catch (cause) {
      error('Drive Failed', cause instanceof Error ? cause.message : 'Could not load files');
    } finally {
      isLoading = false;
      isLoadingMore = false;
    }
  }

  async function loadTrash() {
    trashLoading = true;
    try {
      const data = await api.listTrash();
      trash = data.files || [];
    } catch (cause) {
      error('Trash Failed', cause instanceof Error ? cause.message : 'Could not load trash');
    } finally {
      trashLoading = false;
    }
  }

  async function consumeIntent() {
    const intent = consumeSelectionIntent();
    if (!intent || intent.kind !== 'file') return;

    let target = files.find((file) => file.id === intent.id) || null;
    if (!target) {
      try {
        target = await api.getFile(intent.id);
        files = dedupeFiles([target, ...files]);
      } catch {
        return;
      }
    }

    activeLens = 'all';
    currentFolder = target.folder_path || '';
    selectedFile = target;
  }

  function setLens(lens: DriveLens) {
    activeLens = lens;
    if (lens === 'trash') {
      void loadTrash();
    }
  }

  function setViewMode(mode: 'grid' | 'list') {
    viewMode = mode;
    setDefaultDriveView(mode);
  }

  function requestUpload() {
    if (typeof document !== 'undefined') {
      document.dispatchEvent(new CustomEvent('tssp:request-upload'));
    }
  }

  function folderCount(path: string) {
    return folderEntries.find((entry) => entry.path === path)?.file_count || 0;
  }

  function selectFile(file: FileRecord) {
    selectedFile = file;
  }

  function openPreview(file: FileRecord) {
    selectedFile = file;
    previewFile = file;
  }

  function updateFileInState(nextFile: FileRecord) {
    files = files.map((file) => (file.id === nextFile.id ? nextFile : file));
    if (selectedFile?.id === nextFile.id) selectedFile = nextFile;
    if (shareFile?.id === nextFile.id) shareFile = nextFile;
    if (previewFile?.id === nextFile.id) previewFile = nextFile;
    if (moveDialogFile?.id === nextFile.id) moveDialogFile = nextFile;
  }

  function downloadFile(file: FileRecord) {
    window.location.assign(`/api/v1/files/${encodeURIComponent(file.id)}/content`);
  }

  async function handleRefresh() {
    await loadLibrary(true);
    if (isTrashView) {
      await loadTrash();
    }
    info('Refreshed', 'Drive state reloaded from the server');
  }

  async function handleRename(file: FileRecord) {
    const nextName = prompt('Rename file', file.name)?.trim();
    if (!nextName || nextName === file.name) return;

    try {
      const success_op = await driveStateManager.renameFile(file, nextName);
      if (success_op) {
        updateFileInState({ ...file, name: nextName });
        success('Renamed', `${file.name} is now ${nextName}`);
      }
    } catch (cause) {
      error('Rename Failed', cause instanceof Error ? cause.message : 'Could not rename file');
    }
  }

  async function handlePin(file: FileRecord) {
    try {
      const success_op = await driveStateManager.togglePin(file);
      if (success_op) {
        const updated = { ...file, pinned_at: file.pinned_at ? undefined : Math.floor(Date.now() / 1000), pinned: !file.pinned_at };
        updateFileInState(updated);
        success(file.pinned_at ? 'Unpinned' : 'Pinned', file.pinned_at ? `${file.name} moved out of pinned` : `${file.name} will stay at the top`);
      }
    } catch (cause) {
      error('Pin Failed', cause instanceof Error ? cause.message : 'Could not update pin state');
    }
  }

  async function handleDelete(file: FileRecord) {
    if (!confirm(`Move "${file.name}" to trash?`)) return;

    try {
      const success_op = await driveStateManager.deleteFile(file);
      if (success_op) {
        files = files.filter((item) => item.id !== file.id);
        if (selectedFile?.id === file.id) selectedFile = null;
        await Promise.all([loadTrash(), refreshMetadata()]);
        success('Moved to Trash', `${file.name} can be restored from trash`);
      }
    } catch (cause) {
      error('Delete Failed', cause instanceof Error ? cause.message : 'Could not delete file');
    }
  }

  async function handleRestore(file: FileRecord) {
    try {
      const success_op = await driveStateManager.restoreFile(file);
      if (success_op) {
        trash = trash.filter((item) => item.id !== file.id);
        await Promise.all([loadLibrary(true), loadTrash()]);
        success('Restored', `${file.name} is back in Drive`);
      }
    } catch (cause) {
      error('Restore Failed', cause instanceof Error ? cause.message : 'Could not restore file');
    }
  }

  async function handlePermanentDelete(file: FileRecord) {
    if (!confirm(`Permanently purge "${file.name}" from trash?`)) return;

    try {
      const success_op = await driveStateManager.permanentlyDeleteFile(file);
      if (success_op) {
        trash = trash.filter((item) => item.id !== file.id);
        if (selectedFile?.id === file.id) selectedFile = null;
        success('Purged', `${file.name} was permanently deleted`);
      }
    } catch (cause) {
      error('Purge Failed', cause instanceof Error ? cause.message : 'Could not purge file');
    }
  }

  async function handlePurgeExpiredTrash() {
    const confirmed = confirm('Purge only trash items older than the configured retention period?');
    if (!confirmed) return;

    try {
      const success_op = await driveStateManager.emptyTrash();
      if (success_op) {
        await loadTrash();
        success('Expired Trash Purged', 'Only items older than retention were removed');
      }
    } catch (cause) {
      error('Trash Purge Failed', cause instanceof Error ? cause.message : 'Could not purge expired trash');
    }
  }

  async function handleMove(fileId: string, folderPath: string) {
    try {
      isMoving = true;
      const file = files.find(f => f.id === fileId);
      if (!file) {
        error('Move Failed', 'File not found');
        return;
      }
      const success_op = await driveStateManager.moveFile(file, folderPath);
      if (success_op) {
        await refreshMetadata();
        success('Moved', `File moved to ${folderPath || 'Bucket root'}`);
      }
    } catch (cause) {
      error('Move Failed', cause instanceof Error ? cause.message : 'Could not move file');
    } finally {
      isMoving = false;
    }
  }

  async function handleShareChange(fileId: string, isPublic: boolean): Promise<FileRecord | null> {
    try {
      const response = await driveStateManager.setFileVisibility(fileId, isPublic);
      if (response) {
        success(isPublic ? 'Public Link Enabled' : 'Made Private', response.name || 'File');
      }
      return response;
    } catch (cause) {
      error('Share Failed', cause instanceof Error ? cause.message : 'Could not update visibility');
      return null;
    }
  }

  async function handleToggleVisibility(isPublic: boolean) {
    if (!selectedFile) return;
    await handleShareChange(selectedFile.id, isPublic);
  }

  async function refreshMetadata() {
    try {
      const [folderData, statusData] = await Promise.all([api.listFolders(), api.getStatus()]);
      folderEntries = folderData.folders || [];
      status = statusData;
    } catch {
      // Metadata refresh should not replace the main action result with another error.
    }
  }

  function showContextMenu(event: MouseEvent, file: FileRecord) {
    event.preventDefault();
    contextMenu = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      file,
    };
  }

  async function handleOpenAsWorkspace(file: FileRecord) {
    try {
      const workspaceId = await createWorkspaceFromFolder(file.name, file.folder_path || '');
      success('Workspace Created', `Opened "${file.name}" as a workspace`);
      selectionIntent.set({ kind: 'workspace', id: workspaceId });
    } catch (cause) {
      error('Failed to Open Workspace', cause instanceof Error ? cause.message : 'Could not create workspace');
    }
  }

  interface MenuItem {
    icon?: any;
    label: string;
    action: () => void | Promise<void>;
    danger?: boolean;
  }

  function getContextItems(file: FileRecord): MenuItem[] {
    if (isTrashView) {
      return [
        { icon: Icons.RotateCcw, label: 'Restore', action: () => handleRestore(file) },
        { icon: Icons.Trash2, label: 'Purge permanently', action: () => handlePermanentDelete(file), danger: true },
      ];
    }

    const items: MenuItem[] = [
      { icon: Icons.Eye, label: 'Preview', action: () => openPreview(file) },
      { icon: Icons.Download, label: 'Download', action: () => downloadFile(file) },
      { icon: Icons.Share2, label: file.visibility === 'public' ? 'Manage sharing' : 'Share', action: () => { selectedFile = file; shareFile = file; } },
      { icon: Icons.Pin, label: file.pinned_at ? 'Unpin' : 'Pin', action: () => handlePin(file) },
      { icon: Icons.Pencil, label: 'Rename', action: () => handleRename(file) },
      { icon: Icons.FolderOpen, label: 'Move', action: () => { selectedFile = file; moveDialogFile = file; isMoveDialogOpen = true; } },
    ];

    if (isDriveFolder(file)) {
      items.push({ icon: Icons.Code, label: 'Open as Workspace', action: () => handleOpenAsWorkspace(file) });
    }

    items.push({ icon: Icons.Trash2, label: 'Move to trash', action: () => handleDelete(file), danger: true });

    return items;
  }

  const libraryFilters = [
    { id: 'all' as const, label: 'All files', icon: Icons.HardDrive },
    { id: 'images' as const, label: 'Images', icon: Icons.Image },
    { id: 'videos' as const, label: 'Videos', icon: Icons.Video },
    { id: 'documents' as const, label: 'Documents', icon: Icons.FileText },
    { id: 'public' as const, label: 'Public', icon: Icons.Globe },
    { id: 'trash' as const, label: 'Trash', icon: Icons.Trash2 },
  ];
</script>

<div class="drive-shell">
  <aside class="drive-sidebar">
    <button type="button" class="upload-cta" onclick={requestUpload}>
      <Icons.Upload size={15} />
      <span>Upload</span>
    </button>

    <div class="sidebar-group">
      <div class="group-label">Views</div>
      {#each libraryFilters as filter (filter.id)}
        {@const Icon = filter.icon}
        <button
          type="button"
          class="sidebar-item"
          class:active={activeLens === filter.id}
          onclick={() => setLens(filter.id)}
        >
          <Icon size={14} />
          <span>{filter.label}</span>
          {#if filter.id === 'public'}
            <small>{files.filter((file) => file.visibility === 'public').length}</small>
          {:else if filter.id === 'trash'}
            <small>{trash.length}</small>
          {/if}
        </button>
      {/each}
    </div>

    <div class="sidebar-group folders">
      <div class="group-label">Folders</div>
      <button
        type="button"
        class="sidebar-item"
        class:active={currentFolder === ''}
        onclick={() => {
          currentFolder = '';
          if (activeLens === 'trash') activeLens = 'all';
        }}
      >
        <Icons.FolderOpen size={14} />
        <span>Bucket root</span>
      </button>

      {#each folderEntries as folder (folder.path)}
        <button
          type="button"
          class="sidebar-item"
          class:active={currentFolder === folder.path}
          onclick={() => {
            currentFolder = folder.path;
            if (activeLens === 'trash') activeLens = 'all';
          }}
        >
          <Icons.Folder size={14} />
          <span>{folder.path}</span>
          <small>{folderCount(folder.path)}</small>
        </button>
      {/each}
    </div>

    <div class="sidebar-storage">
      <div class="group-label">Storage</div>
      <strong>{formatBytes(status?.storage_bytes_used || 0)}</strong>
      <span>{status?.file_count || 0} tracked objects</span>
    </div>
  </aside>

  <section class="drive-main">
    <header class="drive-header">
      <div>
        <div class="breadcrumbs">
          <span>Cloud Drive</span>
          {#if currentFolder}
            <Icons.ChevronRight size={12} />
            <span>{currentFolder}</span>
          {/if}
        </div>
        <h1>{isTrashView ? 'Trash' : 'Cloud Drive'}</h1>
        <p>
          {#if isTrashView}
            Restore or permanently purge deleted objects.
          {:else}
            Browse, upload, preview, share, and manage your local cloud objects.
          {/if}
        </p>
      </div>

      <div class="header-actions">
        <button type="button" class="ghost-btn" onclick={handleRefresh}>
          <Icons.RefreshCcw size={14} />
          Refresh
        </button>
        {#if isTrashView}
          <button type="button" class="danger-btn" onclick={handlePurgeExpiredTrash} disabled={trash.length === 0}>
            <Icons.Trash2 size={14} />
            Purge expired
          </button>
        {:else}
          <button type="button" class="accent-btn" onclick={requestUpload}>
            <Icons.Upload size={15} />
            Upload files
          </button>
        {/if}
      </div>
    </header>

    <div class="summary-grid">
      <article class="summary-card">
        <span>Objects</span>
        <strong>{status?.file_count || files.length}</strong>
      </article>
      <article class="summary-card">
        <span>Visible here</span>
        <strong>{isTrashView ? filteredTrash.length : filteredLibraryFiles.length}</strong>
      </article>
      <article class="summary-card">
        <span>Pinned</span>
        <strong>{status?.pinned_count || files.filter((file) => file.pinned_at).length}</strong>
      </article>
      <article class="summary-card">
        <span>Public</span>
        <strong>{files.filter((file) => file.visibility === 'public').length}</strong>
      </article>
    </div>

    <div class="toolbar">
      <div class="search-box">
        <Icons.Search size={15} />
        <input
          type="text"
          placeholder={isTrashView ? 'Search trash' : 'Search loaded files'}
          bind:value={filterQuery}
        />
      </div>

      <div class="toolbar-right">
        <div class="view-toggle">
          <button type="button" class:active={viewMode === 'grid'} onclick={() => setViewMode('grid')}>
            <Icons.Grid2x2 size={15} />
          </button>
          <button type="button" class:active={viewMode === 'list'} onclick={() => setViewMode('list')}>
            <Icons.List size={15} />
          </button>
        </div>
      </div>
    </div>

    <div class="drive-content">
      <div class="content-main">
        {#if isTrashView}
          <TrashView
            files={filteredTrash}
            isLoading={trashLoading}
            onRestore={handleRestore}
            onDelete={handlePermanentDelete}
            onEmptyTrash={handlePurgeExpiredTrash}
            onContextMenu={showContextMenu}
          />
        {:else if isLoading}
          <div class="loading-panel">
            <div class="spinner"></div>
            <strong>Loading Drive</strong>
            <p>Fetching files, folders, and storage state.</p>
          </div>
        {:else if filteredLibraryFiles.length === 0}
          <div class="empty-panel">
            <Icons.Cloud size={28} />
            <strong>No files in this view</strong>
            <p>Upload files or change the folder and lens filters.</p>
            <button type="button" class="accent-btn" onclick={requestUpload}>
              <Icons.Upload size={15} />
              Upload into Drive
            </button>
          </div>
        {:else if viewMode === 'grid'}
          <div class="file-grid">
            {#each filteredLibraryFiles as file (file.id)}
              <button
                type="button"
                class="file-card"
                class:selected={selectedFile?.id === file.id}
                onclick={() => selectFile(file)}
                ondblclick={() => openPreview(file)}
                oncontextmenu={(event) => showContextMenu(event, file)}
              >
                <div class="file-surface">
                  <FileIcon mimeType={file.mime_type} name={file.name} size={34} />
                  {#if file.visibility === 'public'}
                    <span class="inline-badge public">Public</span>
                  {/if}
                  {#if file.pinned_at}
                    <span class="inline-badge pinned"><Icons.Pin size={10} /></span>
                  {/if}
                </div>
                <div class="file-copy">
                  <strong>{file.name}</strong>
                  <span>{formatBytes(file.size_bytes)} · {formatRelative(file.updated_at || file.uploaded_at)}</span>
                </div>
              </button>
            {/each}
          </div>
        {:else}
          <div class="file-list">
            <div class="list-head">
              <span>Name</span>
              <span>Size</span>
              <span>Updated</span>
              <span>Folder</span>
              <span>State</span>
            </div>

            {#each filteredLibraryFiles as file (file.id)}
              <button
                type="button"
                class="list-row"
                class:selected={selectedFile?.id === file.id}
                onclick={() => selectFile(file)}
                ondblclick={() => openPreview(file)}
                oncontextmenu={(event) => showContextMenu(event, file)}
              >
                <div class="name-cell">
                  <FileIcon mimeType={file.mime_type} name={file.name} size={20} />
                  <span>{file.name}</span>
                </div>
                <span>{formatBytes(file.size_bytes)}</span>
                <span>{formatRelative(file.updated_at || file.uploaded_at)}</span>
                <span>{file.folder_path || 'Bucket root'}</span>
                <div class="state-cell">
                  {#if file.visibility === 'public'}
                    <span class="inline-badge public">Public</span>
                  {/if}
                  {#if file.pinned_at}
                    <span class="inline-badge pinned">Pinned</span>
                  {/if}
                  {#if file.visibility !== 'public' && !file.pinned_at}
                    <span class="muted-state">Private</span>
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        {/if}

        {#if !isTrashView && hasMore}
          <div class="load-more-row">
            <button type="button" class="ghost-btn" onclick={() => loadLibrary(false)} disabled={isLoadingMore}>
              {#if isLoadingMore}
                <div class="spinner small"></div>
                Loading more
              {:else}
                <Icons.ChevronsDown size={14} />
                Load more
              {/if}
            </button>
          </div>
        {/if}
      </div>

      {#if selectedFile}
        <DriveDetailsPanel
          file={selectedFile}
          onToggleVisibility={handleToggleVisibility}
          onShare={() => {
            if (selectedFile) shareFile = selectedFile;
          }}
          onMove={() => {
            if (selectedFile) {
              moveDialogFile = selectedFile;
              isMoveDialogOpen = true;
            }
          }}
        />
      {/if}
    </div>
  </section>
</div>

<ContextMenu
  bind:visible={contextMenu.visible}
  x={contextMenu.x}
  y={contextMenu.y}
  items={contextMenu.file ? getContextItems(contextMenu.file) : []}
/>

<FilePreviewModal
  file={previewFile}
  isOpen={previewFile !== null}
  onClose={() => (previewFile = null)}
  onDownload={(fileId) => {
    const target = files.find((file) => file.id === fileId) || previewFile;
    if (target) downloadFile(target);
  }}
/>

<SharingModal
  file={shareFile}
  isOpen={shareFile !== null}
  onClose={() => (shareFile = null)}
  onShare={handleShareChange}
/>

<MoveFileDialog
  file={moveDialogFile}
  folders={folderEntries.map((entry) => entry.path)}
  isOpen={isMoveDialogOpen}
  isMoving={isMoving}
  onMove={handleMove}
  onClose={() => {
    isMoveDialogOpen = false;
    moveDialogFile = null;
  }}
/>

<style>
  .drive-shell {
    flex: 1;
    display: grid;
    grid-template-columns: 240px minmax(0, 1fr);
    min-height: 0;
  }

  .drive-sidebar {
    border-right: 1px solid var(--hairline);
    background: rgba(9, 10, 14, 0.78);
    padding: 16px 10px 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow: auto;
  }

  .upload-cta,
  .accent-btn,
  .ghost-btn,
  .danger-btn,
  .sidebar-item,
  .view-toggle button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-family: inherit;
  }

  .upload-cta,
  .accent-btn {
    height: 38px;
    padding: 0 14px;
    border: 1px solid rgba(91, 227, 154, 0.18);
    border-radius: 12px;
    background: linear-gradient(135deg, rgba(91, 227, 154, 0.22), rgba(110, 168, 255, 0.18));
    color: var(--text);
    justify-content: center;
  }

  .sidebar-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .group-label {
    padding: 0 10px;
    font-size: 10px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--dim);
    font-family: var(--ff-mono);
  }

  .sidebar-item {
    width: 100%;
    min-height: 34px;
    padding: 0 10px;
    border: 1px solid transparent;
    border-radius: 10px;
    background: transparent;
    color: var(--text-2);
    justify-content: flex-start;
  }

  .sidebar-item span {
    flex: 1;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sidebar-item small {
    color: var(--dim);
    font-size: 11px;
    font-family: var(--ff-mono);
  }

  .sidebar-item:hover,
  .sidebar-item.active {
    border-color: var(--border);
    background: var(--surface);
    color: var(--text);
  }

  .sidebar-storage {
    margin-top: auto;
    padding: 14px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .sidebar-storage strong {
    font-size: 22px;
    color: var(--text);
  }

  .sidebar-storage span {
    font-size: 12px;
    color: var(--muted);
  }

  .drive-main {
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .drive-header {
    padding: 22px 24px 16px;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .breadcrumbs {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--muted);
    margin-bottom: 8px;
  }

  .drive-header h1 {
    margin: 0;
    font-size: 34px;
    line-height: 1;
    letter-spacing: -0.04em;
    font-family: var(--ff-display);
    color: var(--text);
  }

  .drive-header p {
    margin: 10px 0 0;
    color: var(--muted);
    font-size: 14px;
  }

  .header-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .ghost-btn,
  .danger-btn {
    height: 36px;
    padding: 0 13px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
  }

  .danger-btn {
    color: var(--danger);
    border-color: rgba(255, 107, 107, 0.2);
  }

  .summary-grid {
    padding: 0 24px 18px;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
  }

  .summary-card {
    min-height: 86px;
    padding: 14px 16px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.96), rgba(14, 15, 21, 0.94));
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .summary-card span {
    font-size: 11px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--muted);
    font-family: var(--ff-mono);
  }

  .summary-card strong {
    font-size: 26px;
    color: var(--text);
  }

  .toolbar {
    margin: 0 24px 16px;
    padding: 12px 14px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(16, 18, 24, 0.88);
    display: flex;
    gap: 14px;
    align-items: center;
    justify-content: space-between;
  }

  .search-box {
    flex: 1;
    min-width: 0;
    height: 42px;
    padding: 0 14px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
  }

  .search-box input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 14px;
  }

  .toolbar-right {
    display: flex;
    gap: 12px;
    align-items: center;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .view-toggle button {
    height: 32px;
    padding: 0 12px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
  }

  .view-toggle button.active {
    background: var(--surface-hi);
    border-color: var(--border-2);
    color: var(--text);
  }

  .view-toggle {
    display: inline-flex;
    gap: 8px;
  }

  .view-toggle button {
    width: 36px;
    padding: 0;
    justify-content: center;
    border-radius: 10px;
  }

  .drive-content {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 300px;
    gap: 0;
    overflow: hidden;
  }

  .content-main {
    min-width: 0;
    padding: 0 24px 130px;
    overflow: auto;
  }

  .loading-panel,
  .empty-panel {
    min-height: 320px;
    border: 1px dashed var(--border);
    border-radius: 18px;
    background: rgba(18, 20, 27, 0.42);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--muted);
    text-align: center;
  }

  .loading-panel strong,
  .empty-panel strong {
    color: var(--text);
  }

  .empty-panel p,
  .loading-panel p {
    margin: 0;
    max-width: 340px;
  }

  .file-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 12px;
  }

  .file-card {
    border: 1px solid var(--border);
    background: var(--surface);
    color: inherit;
    border-radius: 16px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    text-align: left;
    cursor: pointer;
  }

  .file-card:hover,
  .list-row:hover {
    background: var(--surface-2);
    border-color: var(--border-2);
  }

  .file-card.selected,
  .list-row.selected {
    border-color: var(--blue);
    box-shadow: 0 0 0 2px rgba(110, 168, 255, 0.12);
  }

  .file-surface {
    position: relative;
    aspect-ratio: 4 / 3;
    border-radius: 12px;
    background:
      radial-gradient(circle at top, rgba(255, 255, 255, 0.05), transparent 60%),
      linear-gradient(180deg, rgba(22, 24, 31, 0.96), rgba(14, 16, 21, 0.98));
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .inline-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 20px;
    padding: 0 8px;
    border-radius: 999px;
    font-size: 10px;
    font-family: var(--ff-mono);
  }

  .inline-badge.public {
    background: rgba(110, 168, 255, 0.14);
    color: var(--blue);
  }

  .inline-badge.pinned {
    background: rgba(255, 95, 162, 0.14);
    color: var(--pink);
  }

  .file-surface .inline-badge {
    position: absolute;
    top: 8px;
  }

  .file-surface .inline-badge.public {
    left: 8px;
  }

  .file-surface .inline-badge.pinned {
    right: 8px;
  }

  .file-copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-copy strong {
    color: var(--text);
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-copy span,
  .muted-state {
    color: var(--muted);
    font-size: 11px;
  }

  .file-list {
    border: 1px solid var(--border);
    border-radius: 16px;
    overflow: hidden;
    background: var(--surface);
  }

  .list-head,
  .list-row {
    display: grid;
    grid-template-columns: minmax(0, 2.2fr) 0.7fr 0.7fr 1fr 0.9fr;
    gap: 12px;
    align-items: center;
  }

  .list-head {
    padding: 11px 14px;
    border-bottom: 1px solid var(--hairline);
    font-size: 10px;
    color: var(--dim);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    font-family: var(--ff-mono);
  }

  .list-row {
    width: 100%;
    padding: 12px 14px;
    border: none;
    border-bottom: 1px solid var(--hairline);
    background: transparent;
    color: var(--text-2);
    text-align: left;
    cursor: pointer;
  }

  .list-row:last-child {
    border-bottom: none;
  }

  .name-cell,
  .state-cell {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    flex-wrap: wrap;
  }

  .name-cell span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .load-more-row {
    display: flex;
    justify-content: center;
    padding: 18px 0 0;
  }

  .spinner {
    width: 22px;
    height: 22px;
    border-radius: 999px;
    border: 2px solid rgba(255, 255, 255, 0.12);
    border-top-color: var(--blue);
    animation: spin 0.8s linear infinite;
  }

  .spinner.small {
    width: 14px;
    height: 14px;
    border-width: 2px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 1200px) {
    .drive-shell {
      grid-template-columns: 1fr;
    }

    .drive-sidebar {
      display: none;
    }

    .drive-content {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 760px) {
    .drive-header,
    .summary-grid,
    .content-main {
      padding-left: 16px;
      padding-right: 16px;
    }

    .summary-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .toolbar {
      margin-left: 16px;
      margin-right: 16px;
      flex-direction: column;
      align-items: stretch;
    }

    .toolbar-right {
      justify-content: space-between;
    }

    .file-grid {
      grid-template-columns: 1fr;
    }

    .list-head {
      display: none;
    }

    .list-row {
      grid-template-columns: 1fr;
      gap: 8px;
    }
  }
</style>
