<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import DriveFolderRail from './DriveFolderRail.svelte';
  import DriveFileView from './DriveFileView.svelte';
  import DriveDetailsPanel from './DriveDetailsPanel.svelte';
  import DriveToolbar from './DriveToolbar.svelte';
  import TrashView from './TrashView.svelte';
  import MoveFileDialog from './MoveFileDialog.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import FilePreviewModal from '$lib/components/FilePreviewModal.svelte';
  import SharingModal from '$lib/components/SharingModal.svelte';
  import UploadQueue from '$lib/components/UploadQueue.svelte';
  import type { FileRecord } from '$lib/api';
  import * as DriveService from '$lib/services/driveService';
  import { success, error } from '$lib/stores/notifications';
  import { onMount } from 'svelte';

  // State
  let files: FileRecord[] = $state([]);
  let trash: FileRecord[] = $state([]);
  let folders: string[] = $state([]);
  let selectedFile: FileRecord | null = $state(null);
  let isLoading = $state(true);
  let trashLoading = $state(false);
  let currentTab: 'files' | 'trash' = $state('files');
  let viewMode: 'list' | 'grid' = $state(
    (typeof localStorage !== 'undefined' ? (localStorage.getItem('driveViewMode') as any) : 'grid') || 'grid'
  );
  let filterQuery = $state('');
  let searchResults: FileRecord[] = $state([]);
  let isSearching = $state(false);
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;

  // UI state
  let contextMenu = $state({ visible: false, x: 0, y: 0, file: null as FileRecord | null });
  let previewFile: FileRecord | null = $state(null);
  let shareFile: FileRecord | null = $state(null);
  let fileInput: HTMLInputElement = $state(undefined)!;
  let moveDialogFile: FileRecord | null = $state(null);
  let isMoveDialogOpen = $state(false);
  let isMoving = $state(false);

  // Computed
  let displayFiles = $derived(filterQuery.trim() ? searchResults : files);
  let isEmpty = $derived(displayFiles.length === 0 && !isLoading);

  onMount(async () => {
    await initializeDrive();
  });

  async function initializeDrive() {
    isLoading = true;
    try {
      const [loadedFiles, loadedFolders] = await Promise.all([
        DriveService.loadFiles(),
        DriveService.loadFolders(),
      ]);
      files = loadedFiles;
      folders = loadedFolders;
    } catch {
      error('Failed to load drive');
    } finally {
      isLoading = false;
    }
  }

  function setViewMode(mode: 'list' | 'grid') {
    viewMode = mode;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('driveViewMode', mode);
    }
  }

  function setFolder(path: string) {
    // Folder filtering happens at the store level in drive.ts
    selectedFile = null;
  }

  async function handleSearch(query: string) {
    filterQuery = query;
    
    if (searchTimeout) clearTimeout(searchTimeout);

    if (query.trim()) {
      isSearching = true;
      searchTimeout = setTimeout(async () => {
        try {
          searchResults = await DriveService.searchFiles(query);
        } finally {
          isSearching = false;
        }
      }, 300);
    } else {
      searchResults = [];
      isSearching = false;
    }
  }

  function showContextMenu(event: MouseEvent, file: FileRecord) {
    event.preventDefault();
    event.stopPropagation();
    contextMenu = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      file,
    };
  }

  function getContextItems(file: FileRecord) {
    if (currentTab === 'trash') {
      return [
        { label: 'Restore', action: () => handleRestoreFile(file) },
        { label: 'Delete permanently', action: () => handleDeletePermanent(file), danger: true },
      ];
    }
    return [
      { label: 'Preview', action: () => (previewFile = file) },
      { label: 'Download', action: () => handleDownload(file) },
      { label: file.pinned_at ? 'Unpin' : 'Pin', action: () => handleTogglePin(file) },
      { label: file.visibility === 'public' ? 'Make Private' : 'Share', action: () => (shareFile = file) },
      { label: 'Rename', action: () => handleRename(file) },
      { label: 'Move', action: () => { moveDialogFile = file; isMoveDialogOpen = true; } },
      { label: 'Delete', action: () => handleDelete(file), danger: true },
    ];
  }

  // File operations
  async function handleUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files) {
      const formData = new FormData();
      for (const file of input.files) {
        formData.append('file', file);
      }
      try {
        await fetch('/api/v1/upload', { method: 'POST', body: formData });
        success('Files uploaded successfully');
        files = await DriveService.loadFiles();
      } catch (err) {
        error(`Upload failed: ${err instanceof Error ? err.message : 'Unknown error'}`);
      }
      input.value = '';
    }
  }

  async function handleDownload(file: FileRecord) {
    try {
      await DriveService.downloadFile(file.id, file.name);
    } catch {
      // Error already shown by service
    }
  }

  async function handleDelete(file: FileRecord) {
    if (!confirm(`Delete "${file.name}"?`)) return;
    try {
      // Soft delete - move to trash
      files = files.filter(f => f.id !== file.id);
      selectedFile = null;
      success('File deleted');
    } catch {
      // Error already shown by service
    }
  }

  async function handleDeletePermanent(file: FileRecord) {
    if (!confirm(`Permanently delete "${file.name}"? This cannot be undone.`)) return;
    try {
      await DriveService.deleteFilePermanently(file.id);
      trash = trash.filter(f => f.id !== file.id);
      success('File permanently deleted');
    } catch {
      // Error already shown by service
    }
  }

  async function handleRename(file: FileRecord) {
    const newName = prompt('New name:', file.name);
    if (!newName || newName === file.name) return;
    try {
      await DriveService.renameFile(file.id, newName);
      file.name = newName;
      success('File renamed');
    } catch {
      // Error already shown by service
    }
  }

  async function handleTogglePin(file: FileRecord) {
    try {
      if (file.pinned_at) {
        await DriveService.unpinFile(file.id);
        file.pinned_at = undefined;
        success('File unpinned');
      } else {
        await DriveService.pinFile(file.id);
        file.pinned_at = Math.floor(Date.now() / 1000);
        success('File pinned');
      }
    } catch {
      // Error already shown by service
    }
  }

  async function handleToggleVisibility(isPublic: boolean) {
    if (!shareFile) return;
    try {
      const result = await DriveService.toggleFileVisibility(shareFile.id, isPublic);
      if (result) {
        shareFile = result;
        const idx = files.findIndex(f => f.id === shareFile!.id);
        if (idx >= 0) files[idx] = result;
        success(isPublic ? 'File is now public' : 'File is now private');
      }
    } catch {
      // Error already shown by service
    }
  }

  async function handleRestoreFile(file: FileRecord) {
    try {
      await DriveService.restoreFile(file.id);
      trash = trash.filter(f => f.id !== file.id);
      files = await DriveService.loadFiles();
      success('File restored');
    } catch {
      // Error already shown by service
    }
  }

  async function handleEmptyTrash() {
    if (!confirm('Permanently delete all files in trash? This cannot be undone.')) return;
    try {
      await DriveService.emptyTrash();
      trash = [];
      success('Trash emptied');
    } catch {
      // Error already shown by service
    }
  }

  async function switchToTrash() {
    currentTab = 'trash';
    if (trash.length === 0) {
      trashLoading = true;
      try {
        trash = await DriveService.loadTrash();
      } finally {
        trashLoading = false;
      }
    }
  }

  async function handleMoveFile(fileId: string, folderPath: string) {
    try {
      isMoving = true;
      const result = await DriveService.moveFile(fileId, folderPath);
      if (result) {
        const idx = files.findIndex(f => f.id === fileId);
        if (idx >= 0) {
          files[idx] = result;
          selectedFile = result;
        }
        success('File moved successfully');
      }
    } catch {
      // Error already shown by service
    } finally {
      isMoving = false;
    }
  }
</script>

<div class="drive-container">
  <DriveFolderRail {folders} onSelectFolder={setFolder} />

  <div class="drive-main">
    <div class="drive-header">
      <div>
        <h2>{currentTab === 'files' ? 'Cloud Drive' : 'Trash'}</h2>
        <p class="subtitle">
          {currentTab === 'files'
            ? 'Organize, share, and access your files'
            : 'Recently deleted files'}
        </p>
      </div>
      {#if currentTab === 'files'}
        <button type="button" class="upload-btn" onclick={() => fileInput?.click()}>
          <Icons.Upload size={16} />
          Upload
        </button>
        <input
          bind:this={fileInput}
          type="file"
          multiple
          onchange={handleUpload}
          style="display: none"
        />
      {:else if trash.length > 0}
        <button type="button" class="empty-trash-btn" onclick={handleEmptyTrash}>
          <Icons.Trash2 size={16} />
          Empty Trash
        </button>
      {/if}
    </div>

    <div class="drive-tabs">
      <button
        class="tab-btn"
        class:active={currentTab === 'files'}
        onclick={() => (currentTab = 'files')}
      >
        <Icons.HardDrive size={16} />
        My Files
      </button>
      <button
        class="tab-btn"
        class:active={currentTab === 'trash'}
        onclick={switchToTrash}
      >
        <Icons.Trash2 size={16} />
        Trash
      </button>
    </div>

    {#if currentTab === 'files'}
      <DriveToolbar
        {filterQuery}
        {isSearching}
        {viewMode}
        onSearch={handleSearch}
        onViewModeChange={setViewMode}
      />
      <DriveFileView
        files={displayFiles}
        {viewMode}
        isLoading={isLoading && currentTab === 'files'}
        isEmpty={isEmpty && currentTab === 'files'}
        onSelectFile={(file) => {
          selectedFile = file;
        }}
        onContextMenu={showContextMenu}
      />
    {:else}
      <TrashView
        files={trash}
        isLoading={trashLoading}
        onRestore={handleRestoreFile}
        onDelete={handleDeletePermanent}
        onEmptyTrash={handleEmptyTrash}
        onContextMenu={showContextMenu}
      />
    {/if}
  </div>

  <DriveDetailsPanel
    file={selectedFile}
    onToggleVisibility={async (isPublic) => {
      if (selectedFile) {
        await handleToggleVisibility(isPublic);
      }
    }}
    onShare={() => (shareFile = selectedFile)}
    onMove={() => {
      if (selectedFile) {
        moveDialogFile = selectedFile;
        isMoveDialogOpen = true;
      }
    }}
  />
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
/>

<SharingModal
  file={shareFile}
  isOpen={shareFile !== null}
  onClose={() => (shareFile = null)}
  onToggleVisibility={handleToggleVisibility}
/>

<MoveFileDialog
  file={moveDialogFile}
  {folders}
  isOpen={isMoveDialogOpen}
  isMoving={isMoving}
  onMove={async (folderPath) => {
    if (moveDialogFile) {
      await handleMoveFile(moveDialogFile.id, folderPath);
      isMoveDialogOpen = false;
    }
  }}
  onClose={() => (isMoveDialogOpen = false)}
/>

<UploadQueue />

<style>
  .drive-container {
    display: flex;
    height: 100%;
    background: var(--bg);
  }

  .drive-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .drive-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-6);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .drive-header h2 {
    margin: 0 0 var(--s-1) 0;
    font-size: var(--fs-24);
    font-weight: 700;
    color: var(--text);
  }

  .subtitle {
    margin: 0;
    font-size: var(--fs-13);
    color: var(--muted);
  }

  .upload-btn,
  .empty-trash-btn {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-4);
    border: none;
    border-radius: var(--r-2);
    background: var(--blue);
    color: white;
    font-size: var(--fs-13);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .upload-btn:hover {
    background: var(--blue-dark, var(--blue));
    transform: translateY(-1px);
  }

  .empty-trash-btn {
    background: var(--danger);
  }

  .empty-trash-btn:hover {
    background: var(--danger-dark, var(--danger));
  }

  .drive-tabs {
    display: flex;
    gap: var(--s-2);
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border: none;
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-13);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .tab-btn:hover {
    background: var(--surface-2);
  }

  .tab-btn.active {
    background: var(--blue);
    color: white;
  }
</style>
