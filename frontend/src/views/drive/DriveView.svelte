<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { visibleFiles, folders, isLoading, loadFiles, setFolder, selectedIds } from '$lib/stores/drive';
  import FolderTree from '$lib/components/FolderTree.svelte';
  import FileGrid from '$lib/components/FileGrid.svelte';
  import UploadArea from '$lib/components/UploadArea.svelte';
  import { onMount } from 'svelte';

  let showUploadArea = false;
  let selectedFile: any = null;

  onMount(async () => {
    await loadFiles();
  });

  async function handleUpload(event: CustomEvent) {
    const files = event.detail.files;
    const folder = event.detail.folder;

    const formData = new FormData();
    for (const file of files) {
      formData.append('files', file);
      formData.append('folder', folder);
    }

    try {
      const response = await fetch('/api/v1/files/upload', {
        method: 'POST',
        body: formData,
        credentials: 'same-origin',
      });

      if (!response.ok) throw new Error('Upload failed');
      await loadFiles();
      showUploadArea = false;
    } catch (err) {
      console.error('Upload error:', err);
    }
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
        <p class="subtitle">Store, browse, preview, and share files</p>
      </div>
      <button class="upload-btn">
        <Icons.Upload size={16} />
        Upload
      </button>
    </div>

    <div class="content">
      {#if showUploadArea}
        <UploadArea
          onUpload={(files, folder) => {
            handleUpload(new CustomEvent('upload', { detail: { files, folder } }));
          }}
        />
      {:else}
        <FileGrid files={$visibleFiles} loading={$isLoading} />
      {/if}
    </div>
  </div>

  {#if $selectedIds.size === 1}
    <div class="details-panel">
      <div class="panel-header">
        <h3>Details</h3>
        <button class="close-btn">
          <Icons.X size={16} />
        </button>
      </div>
      {#each $visibleFiles as file (file.id)}
        {#if $selectedIds.has(file.id)}
          <div class="detail-item">
            <span class="label">Name</span>
            <span class="value">{file.name}</span>
          </div>
          <div class="detail-item">
            <span class="label">Size</span>
            <span class="value">
              {(file.size_bytes / 1024 / 1024).toFixed(2)} MB
            </span>
          </div>
          <div class="detail-item">
            <span class="label">Type</span>
            <span class="value">{file.mime_type}</span>
          </div>
          <div class="detail-item">
            <span class="label">Modified</span>
            <span class="value">
              {new Date(file.updated_at).toLocaleDateString()}
            </span>
          </div>
          {#if file.tags}
            <div class="detail-item">
              <span class="label">Tags</span>
              <div class="tags">
                {#each file.tags as tag}
                  <span class="tag">{tag}</span>
                {/each}
              </div>
            </div>
          {/if}
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .drive-view {
    flex: 1;
    display: flex;
    overflow: hidden;
    background: var(--bg);
  }

  .sidebar {
    flex-shrink: 0;
    height: 100%;
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
    font-size: var(--fs-13);
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
    font-size: var(--fs-13);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .upload-btn:hover {
    background: var(--blue);
    opacity: 0.9;
  }

  .content {
    flex: 1;
    overflow: hidden;
    display: flex;
  }

  .details-panel {
    width: 280px;
    border-left: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .panel-header {
    padding: 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .panel-header h3 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }

  .close-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-2);
    transition: all 0.15s;
  }

  .close-btn:hover {
    background: var(--surface-2);
  }

  .detail-item {
    padding: 12px 16px;
    border-bottom: 1px solid var(--hairline);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .label {
    font-size: 11px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .value {
    font-size: var(--fs-12);
    color: var(--text);
    word-break: break-word;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 4px;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: var(--r-full);
    background: var(--blue-soft);
    color: var(--blue);
    font-size: 11px;
    font-weight: 500;
  }
</style>
