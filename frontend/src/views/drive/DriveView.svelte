<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { visibleFiles, folders, isLoading, loadFiles, setFolder, selectedIds } from '$lib/stores/drive';
  import { success, error } from '$lib/stores/notifications';
  import FolderTree from '$lib/components/FolderTree.svelte';
  import FileGrid from '$lib/components/FileGrid.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import FilePreviewModal from '$lib/components/FilePreviewModal.svelte';
  import UploadQueue from '$lib/components/UploadQueue.svelte';
  import SharingModal from '$lib/components/SharingModal.svelte';
  import Breadcrumb from '$lib/components/Breadcrumb.svelte';
  import * as FileService from '$lib/services/fileService';
  import { onMount } from 'svelte';

  let contextMenu = { visible: false, x: 0, y: 0, file: null as any };
  let previewFile: any = null;
  let shareFile: any = null;
  let fileInput: HTMLInputElement;
  let filterQuery = '';
  let viewMode: 'list' | 'grid' = (typeof localStorage !== 'undefined' ? (localStorage.getItem('driveViewMode') as any) : 'grid') || 'grid';
  let uploads: any[] = $state([]);

  function toggleViewMode(mode: 'list' | 'grid') {
    viewMode = mode;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('driveViewMode', mode);
    }
  }

  onMount(async () => {
    await loadFiles();
  });

  async function handleUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files) {
      await FileService.uploadFiles(input.files);
      input.value = '';
    }
  }

  function showContextMenu(event: MouseEvent, file: any) {
    event.preventDefault();
    contextMenu = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      file,
    };
  }

  async function handleDownload(file: any) {
    await FileService.downloadFile(file.id, file.name);
  }

  async function handleDelete(file: any) {
    if (confirm(`Delete "${file.name}"?`)) {
      await FileService.deleteFile(file.id, file.name);
    }
  }

  async function handleRename(file: any) {
    const newName = prompt('New name:', file.name);
    if (newName && newName !== file.name) {
      await FileService.renameFile(file.id, newName);
    }
  }

  async function handlePin(file: any) {
    await FileService.togglePin(file.id);
  }

  async function handleShare(file: any) {
    shareFile = file;
  }

  async function handlePreview(file: any) {
    previewFile = file;
  }

  async function handleShareToggle(fileId: string, isPublic: boolean) {
    await FileService.togglePublic(fileId);
    success(isPublic ? 'File shared' : 'File made private');
  }

  let filteredFiles = $derived(filterQuery
    ? $visibleFiles.filter(f => f.name.toLowerCase().includes(filterQuery.toLowerCase()))
    : $visibleFiles);

  function getContextItems(file: any) {
    return [
      { label: 'Preview', action: () => handlePreview(file) },
      { label: 'Download', action: () => handleDownload(file) },
      { label: file.pinned_at ? 'Unpin' : 'Pin', action: () => handlePin(file) },
      { label: file.public ? 'Make Private' : 'Share', action: () => handleShare(file) },
      { label: 'Rename', action: () => handleRename(file) },
      { label: 'Delete', action: () => handleDelete(file), danger: true },
    ];
  }
</script>

<div class="drive-view">
  <div class="sidebar">
    <FolderTree {folders} onSelectFolder={setFolder} />
  </div>

  <div class="main-content">
    <div class="header">
      <div>
        <h2>Cloud Drive</h2>
        <p class="subtitle">Organize, share, and access your files</p>
      </div>
      <button class="upload-btn" on:click={() => fileInput?.click()}>
        <Icons.Upload size={16} />
        Upload
      </button>
      <input
        bind:this={fileInput}
        type="file"
        multiple
        on:change={handleUpload}
        style="display: none"
      />
    </div>

    <div class="search-bar">
      <Icons.Search size={16} />
      <input type="text" placeholder="Search files..." bind:value={filterQuery} />
      <div style="flex: 1" />
      <div class="view-toggle">
        <button
          class="toggle-btn"
          class:active={viewMode === 'grid'}
          on:click={() => toggleViewMode('grid')}
          title="Grid view"
        >
          <Icons.Grid2x2 size={16} />
        </button>
        <button
          class="toggle-btn"
          class:active={viewMode === 'list'}
          on:click={() => toggleViewMode('list')}
          title="List view"
        >
          <Icons.List size={16} />
        </button>
      </div>
    </div>

    <div class="files-container">
      {#if $isLoading}
        <div class="loading">
          <div class="spinner" />
          Loading files...
        </div>
      {:else if filteredFiles.length === 0}
        <div class="empty">
          <Icons.Inbox size={48} />
          <h3>No files</h3>
          <p>Drop files here or click upload to get started</p>
        </div>
      {:else if viewMode === 'list'}
        <div class="files-list">
          {#each filteredFiles as file (file.id)}
            <div
              class="file-row"
              on:click={() => handlePreview(file)}
              on:contextmenu={(e) => showContextMenu(e, file)}
              role="button"
              tabindex="0"
            >
              <div class="file-details">
                <div class="file-icon">
                  <FileIcon mimeType={file.mime_type} name={file.name} size={20} />
                </div>
                <div class="file-info">
                  <div class="file-name">{file.name}</div>
                  <div class="file-meta">
                    {(file.size_bytes / 1024 / 1024).toFixed(1)} MB • {new Date((file.updated_at || file.uploaded_at) * 1000).toLocaleDateString()}
                  </div>
                </div>
              </div>

              <div class="file-actions">
                {#if file.pinned_at}
                  <Icons.Pin size={14} style="color: var(--orange)" title="Pinned" />
                {/if}
                {#if file.public}
                  <Icons.Share2 size={14} style="color: var(--green)" title="Public" />
                {/if}
                <button class="action-btn" on:click={() => handleDownload(file)} title="Download">
                  <Icons.Download size={14} />
                </button>
                <button class="action-btn" on:click={(e) => showContextMenu(e, file)} title="More">
                  <Icons.MoreVertical size={14} />
                </button>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="files-grid">
          {#each filteredFiles as file (file.id)}
            <div
              class="file-card"
              on:click={() => handlePreview(file)}
              on:contextmenu={(e) => showContextMenu(e, file)}
              role="button"
              tabindex="0"
            >
              <div class="file-card-icon">
                <FileIcon mimeType={file.mime_type} name={file.name} size={32} />
              </div>
              <div class="file-card-name">{file.name}</div>
              <div class="file-card-meta">
                {(file.size_bytes / 1024 / 1024).toFixed(1)} MB
              </div>
              <div class="file-card-actions">
                {#if file.pinned_at}
                  <Icons.Pin size={12} style="color: var(--orange)" title="Pinned" />
                {/if}
                {#if file.public}
                  <Icons.Share2 size={12} style="color: var(--green)" title="Public" />
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
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
    const f = $visibleFiles.find(x => x.id === fileId);
    if (f) handleDownload(f);
    previewFile = null;
  }}
/>

<SharingModal
  file={shareFile}
  isOpen={shareFile !== null}
  onClose={() => (shareFile = null)}
  onShare={(fileId, isPublic) => handleShareToggle(fileId, isPublic)}
/>

<UploadQueue {uploads} />

<style>
  .drive-view {
    flex: 1;
    display: flex;
    overflow: hidden;
    background: var(--bg);
  }

  .sidebar {
    flex-shrink: 0;
    width: 220px;
    height: 100%;
    border-right: 1px solid var(--border);
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header {
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--surface);
  }

  .header h2 {
    margin: 0;
    font-size: var(--fs-24);
    color: var(--text);
  }

  .subtitle {
    margin: 4px 0 0;
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .upload-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border-radius: var(--r-2);
    border: 1px solid var(--border);
    background: var(--blue);
    color: #0a1228;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .upload-btn:hover {
    opacity: 0.9;
  }

  .search-bar {
    padding: 12px 24px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--surface);
    color: var(--muted);
  }

  .search-bar input {
    flex: 0 1 300px;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-13);
    outline: none;
  }

  .search-bar input::placeholder {
    color: var(--muted);
  }

  .view-toggle {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
  }

  .toggle-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    transition: all 0.15s;
  }

  .toggle-btn:hover {
    color: var(--text);
  }

  .toggle-btn.active {
    background: var(--surface-3);
    color: var(--blue);
  }

  .files-container {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .empty h3 {
    margin: 0;
    color: var(--text-2);
  }

  .empty p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .files-list {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .file-row {
    padding: 12px 24px;
    border-bottom: 1px solid var(--hairline);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    transition: background 0.15s;
    cursor: context-menu;
  }

  .file-row:hover {
    background: var(--surface);
  }

  .file-details {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .file-icon {
    flex-shrink: 0;
    color: var(--muted);
  }

  .file-info {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-meta {
    font-size: 11px;
    color: var(--muted);
    margin-top: 2px;
  }

  .file-actions {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .pinned {
    color: var(--orange);
  }

  .shared {
    color: var(--green);
  }

  .action-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .action-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .files-grid {
    flex: 1;
    overflow: auto;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 16px;
    padding: 20px;
  }

  .file-card {
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: all 0.15s;
    text-align: center;
    position: relative;
  }

  .file-card:hover,
  .file-card:focus-visible {
    background: var(--surface-2);
    border-color: var(--blue);
    outline: none;
    transform: translateY(-2px);
  }

  .file-card-icon {
    flex-shrink: 0;
    color: var(--muted);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 60px;
    height: 60px;
  }

  .file-card-name {
    font-size: var(--fs-12);
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    word-break: break-word;
    width: 100%;
  }

  .file-card-meta {
    font-size: 11px;
    color: var(--muted);
  }

  .file-card-actions {
    display: flex;
    gap: 4px;
    position: absolute;
    top: 8px;
    right: 8px;
  }
</style>
