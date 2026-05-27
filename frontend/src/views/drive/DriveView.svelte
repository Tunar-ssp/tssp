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
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { isDriveFolder, createWorkspaceFromFolder } from '$lib/services/driveWorkspaceIntegration';
  import FilePreviewModal from '$lib/components/FilePreviewModal.svelte';
  import SharingModal from '$lib/components/SharingModal.svelte';
  import MoveFileDialog from './components/modals/MoveFileDialog.svelte';
  import DriveDetailsPanel from './components/panels/DriveDetailsPanel.svelte';
  import DriveHeader from './DriveHeader.svelte';
  import DriveSidebar from './DriveSidebar.svelte';
  import DriveContent from './DriveContent.svelte';
  import DriveToolbarRow from './components/toolbar/DriveToolbarRow.svelte';
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
  <DriveSidebar
    filters={libraryFilters}
    {activeLens}
    folders={folderEntries}
    {currentFolder}
    publicCount={files.filter((f) => f.visibility === 'public').length}
    trashCount={trash.length}
    usedBytes={status?.storage_bytes_used || 0}
    totalObjects={status?.file_count || files.length}
    onLensChange={setLens}
    onFolderChange={(path) => {
      currentFolder = path;
      if (activeLens === 'trash') activeLens = 'all';
    }}
  />

  <section class="drive-main">
    <DriveHeader
      title={isTrashView ? 'Trash' : 'Cloud Drive'}
      {currentFolder}
      {isTrashView}
      fileCount={status?.file_count || files.length}
      visibleCount={isTrashView ? filteredTrash.length : filteredLibraryFiles.length}
      pinnedCount={status?.pinned_count || files.filter((f) => f.pinned_at).length}
      publicCount={files.filter((f) => f.visibility === 'public').length}
      onRefresh={handleRefresh}
      onUpload={requestUpload}
      onPurgeTrash={handlePurgeExpiredTrash}
      trashEmpty={trash.length === 0}
    />

    <DriveToolbarRow
      filterQuery={filterQuery}
      isTrashView={isTrashView}
      viewMode={viewMode}
      onFilterChange={(query) => filterQuery = query}
      onViewModeChange={setViewMode}
    />

    <DriveContent
      files={filteredLibraryFiles}
      trash={filteredTrash}
      {isLoading}
      {isTrashView}
      {viewMode}
      selectedFileId={selectedFile?.id}
      {hasMore}
      {isLoadingMore}
      onSelectFile={selectFile}
      onPreviewFile={openPreview}
      onContextMenu={showContextMenu}
      onLoadMore={() => loadLibrary(false)}
      onRestore={handleRestore}
      onDelete={handlePermanentDelete}
      onPurgeTrash={handlePurgeExpiredTrash}
      onUpload={requestUpload}
    />

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

  .drive-main {
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }




  @media (max-width: 1200px) {
    .drive-shell {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 760px) {
    .drive-shell {
      grid-template-columns: 1fr;
    }
  }
</style>
