<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { api } from '$lib/api';
  import { visibleFiles, folders, isLoading, loadFiles, setFolder, hasMore, loadMoreFiles } from '$lib/stores/drive';
  import { error } from '$lib/stores/notifications';
  import FolderTree from '$lib/components/FolderTree.svelte';
  import FileGrid from '$lib/components/FileGrid.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import FilePreviewModal from '$lib/components/FilePreviewModal.svelte';
  import UploadQueue from '$lib/components/UploadQueue.svelte';
  import SharingModal from '$lib/components/SharingModal.svelte';
  import * as FileService from '$lib/services/fileService';
  import { onMount } from 'svelte';

  let contextMenu = $state({ visible: false, x: 0, y: 0, file: null as any });
  let previewFile: any = $state(null);
  let shareFile: any = $state(null);
  let fileInput: HTMLInputElement;
  let filterQuery = $state('');
  let currentTab = $state<'files' | 'trash'>('files');
  let viewMode: 'list' | 'grid' = $state(
    (typeof localStorage !== 'undefined' ? (localStorage.getItem('driveViewMode') as any) : 'grid') || 'grid',
  );
  let uploads: any[] = $state([]);
  let searchResults: any[] = $state([]);
  let isSearching = $state(false);
  let searchTimeout: NodeJS.Timeout | null = null;
  let trashFiles: any[] = $state([]);
  let trashLoading = $state(false);

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
    event.stopPropagation();
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
    await FileService.togglePin(file.id, !!file.pinned_at);
  }

  async function handleShare(file: any) {
    shareFile = file;
  }

  async function handlePreview(file: any) {
    previewFile = file;
  }

  async function handleShareToggle(fileId: string, isPublic: boolean) {
    const result = await FileService.togglePublic(fileId, isPublic);
    if (result?.file) shareFile = result.file;
    return result;
  }

  async function performSearch(query: string) {
    if (!query.trim()) {
      searchResults = [];
      isSearching = false;
      return;
    }

    isSearching = true;
    try {
      const data = await api.search(query);
      searchResults = (data.results || []).filter((r: any) => r.type === 'file');
      isSearching = false;
    } catch {
      error('Search failed');
      isSearching = false;
    }
  }

  function handleSearchInput(e: any) {
    filterQuery = e.target.value;

    // Clear previous timeout
    if (searchTimeout) clearTimeout(searchTimeout);

    // Debounce search by 300ms
    if (filterQuery.trim()) {
      isSearching = true;
      searchTimeout = setTimeout(() => performSearch(filterQuery), 300);
    } else {
      searchResults = [];
      isSearching = false;
    }
  }

  let filteredFiles = $derived(filterQuery.trim() ? searchResults : $visibleFiles);

  function handleFileKeydown(event: KeyboardEvent, file: any) {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    void handlePreview(file);
  }

  async function loadTrash() {
    trashLoading = true;
    try {
      const data = await api.listTrash();
      trashFiles = data.files || [];
    } catch (err) {
      error(err instanceof Error ? err.message : 'Failed to load trash');
    } finally {
      trashLoading = false;
    }
  }

  async function handleRestoreFile(file: any) {
    try {
      await api.restoreFile(file.id);
      trashFiles = trashFiles.filter(f => f.id !== file.id);
      await loadFiles();
    } catch (err) {
      error(err instanceof Error ? err.message : 'Failed to restore file');
    }
  }

  async function handlePermanentDelete(file: any) {
    if (confirm(`Permanently delete "${file.name}"? This cannot be undone.`)) {
      try {
        await api.permanentDeleteFile(file.id);
        trashFiles = trashFiles.filter(f => f.id !== file.id);
      } catch (err) {
        error(err instanceof Error ? err.message : 'Failed to delete file');
      }
    }
  }

  async function handleEmptyTrash() {
    if (confirm('Permanently delete all files in trash? This cannot be undone.')) {
      try {
        await api.emptyTrash();
        trashFiles = [];
      } catch (err) {
        error(err instanceof Error ? err.message : 'Failed to empty trash');
      }
    }
  }

  function getContextItems(file: any) {
    if (currentTab === 'trash') {
      return [
        { label: 'Restore', action: () => handleRestoreFile(file) },
        { label: 'Delete permanently', action: () => handlePermanentDelete(file), danger: true },
      ];
    }
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
        <h2>{currentTab === 'files' ? 'Cloud Drive' : 'Trash'}</h2>
        <p class="subtitle">{currentTab === 'files' ? 'Organize, share, and access your files' : 'Recently deleted files'}</p>
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
      {:else if trashFiles.length > 0}
        <button type="button" class="upload-btn danger" onclick={handleEmptyTrash}>
          <Icons.Trash2 size={16} />
          Empty Trash
        </button>
      {/if}
    </div>

    <div class="tabs">
      <button
        class="tab-btn"
        class:active={currentTab === 'files'}
        onclick={() => {
          currentTab = 'files';
          filterQuery = '';
          searchResults = [];
        }}
      >
        <Icons.HardDrive size={16} />
        My Files
      </button>
      <button
        class="tab-btn"
        class:active={currentTab === 'trash'}
        onclick={() => {
          currentTab = 'trash';
          if (trashFiles.length === 0) loadTrash();
        }}
      >
        <Icons.Trash2 size={16} />
        Trash
      </button>
    </div>

    {#if currentTab === 'files'}
      <div class="search-bar">
        <Icons.Search size={16} />
        <input
          type="text"
          placeholder="Search files... (server-side)"
          value={filterQuery}
          oninput={handleSearchInput}
        />
        {#if isSearching}
          <div style="color: var(--muted); font-size: 12px;">Searching...</div>
        {/if}
        <div style="flex: 1"></div>
        <div class="view-toggle">
          <button
            type="button"
            class="toggle-btn"
            class:active={viewMode === 'grid'}
            onclick={() => toggleViewMode('grid')}
            title="Grid view"
          >
            <Icons.Grid2x2 size={16} />
          </button>
          <button
            type="button"
            class="toggle-btn"
            class:active={viewMode === 'list'}
            onclick={() => toggleViewMode('list')}
            title="List view"
          >
            <Icons.List size={16} />
          </button>
        </div>
      </div>
    {/if}

    <div class="files-container">
      {#if currentTab === 'trash'}
        {#if trashLoading}
          <div class="loading">
            <div class="spinner"></div>
            Loading trash...
          </div>
        {:else if trashFiles.length === 0}
          <div class="empty">
            <Icons.Trash2 size={48} />
            <h3>Trash is empty</h3>
            <p>Deleted files will appear here</p>
          </div>
        {:else if viewMode === 'list'}
          <div class="files-list">
            {#each trashFiles as file (file.id)}
              <div
                class="file-row"
                oncontextmenu={(e) => showContextMenu(e, file)}
                onkeydown={(e) => handleFileKeydown(e, file)}
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
                  <button
                    type="button"
                    class="action-btn"
                    onclick={(e) => {
                      e.stopPropagation();
                      void handleRestoreFile(file);
                    }}
                    title="Restore"
                  >
                    <Icons.RotateCcw size={14} />
                  </button>
                  <button
                    type="button"
                    class="action-btn danger"
                    onclick={(e) => {
                      e.stopPropagation();
                      void handlePermanentDelete(file);
                    }}
                    title="Delete permanently"
                  >
                    <Icons.Trash size={14} />
                  </button>
                  <button
                    type="button"
                    class="action-btn"
                    onclick={(e) => showContextMenu(e, file)}
                    title="More"
                  >
                    <Icons.MoreVertical size={14} />
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="files-grid">
            {#each trashFiles as file (file.id)}
              <div
                class="file-card"
                oncontextmenu={(e) => showContextMenu(e, file)}
                onkeydown={(e) => handleFileKeydown(e, file)}
                role="button"
                tabindex="0"
              >
                <div class="file-header">
                  <FileIcon mimeType={file.mime_type} name={file.name} size={24} />
                  <button
                    type="button"
                    class="card-action-btn"
                    onclick={(e) => showContextMenu(e, file)}
                  >
                    <Icons.MoreVertical size={16} />
                  </button>
                </div>
                <div class="file-title">{file.name}</div>
                <div class="file-footer">
                  <span class="file-size">{(file.size_bytes / 1024 / 1024).toFixed(1)} MB</span>
                  <div class="file-actions-compact">
                    <button
                      type="button"
                      class="action-btn"
                      onclick={(e) => {
                        e.stopPropagation();
                        void handleRestoreFile(file);
                      }}
                      title="Restore"
                    >
                      <Icons.RotateCcw size={14} />
                    </button>
                    <button
                      type="button"
                      class="action-btn danger"
                      onclick={(e) => {
                        e.stopPropagation();
                        void handlePermanentDelete(file);
                      }}
                      title="Delete permanently"
                    >
                      <Icons.Trash size={14} />
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      {:else if $isLoading}
        <div class="loading">
          <div class="spinner"></div>
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
              onclick={() => handlePreview(file)}
              oncontextmenu={(e) => showContextMenu(e, file)}
              onkeydown={(e) => handleFileKeydown(e, file)}
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
                <button
                  type="button"
                  class="action-btn"
                  onclick={(e) => {
                    e.stopPropagation();
                    void handleDownload(file);
                  }}
                  title="Download"
                >
                  <Icons.Download size={14} />
                </button>
                <button
                  type="button"
                  class="action-btn"
                  onclick={(e) => showContextMenu(e, file)}
                  title="More"
                >
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
              onclick={() => handlePreview(file)}
              oncontextmenu={(e) => showContextMenu(e, file)}
              onkeydown={(e) => handleFileKeydown(e, file)}
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
      {#if currentTab === 'files' && $hasMore}
        <div class="load-more-container">
          <button
            type="button"
            class="load-more-btn"
            onclick={() => loadMoreFiles()}
            disabled={$isLoading}
          >
            {#if $isLoading}
              <div class="spinner-small"></div>
              Loading...
            {:else}
              <Icons.ChevronDown size={16} />
              Load More Files
            {/if}
          </button>
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
  }

  .upload-btn.danger {
    background: var(--red);
    color: white;
  }

  .tabs {
    display: flex;
    gap: 0;
    padding: 0 24px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: all 0.2s;
  }

  .tab-btn:hover {
    color: var(--text);
  }

  .tab-btn.active {
    color: var(--blue);
    border-bottom-color: var(--blue);
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

  .action-btn.danger {
    color: var(--red);
  }

  .action-btn.danger:hover {
    background: rgba(239, 68, 68, 0.1);
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

  .load-more-container {
    display: flex;
    justify-content: center;
    padding: 20px;
    border-top: 1px solid var(--border);
    background: var(--surface);
  }

  .load-more-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
    color: var(--text);
    cursor: pointer;
    transition: all 0.2s;
  }

  .load-more-btn:hover:not(:disabled) {
    background: var(--surface-3);
    border-color: var(--blue);
    color: var(--blue);
  }

  .load-more-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .spinner-small {
    width: 14px;
    height: 14px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
</style>
