<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord } from '$lib/api';
  import FileIcon from './FileIcon.svelte';

  interface $$Props {
    files?: FileRecord[];
    loading?: boolean;
    onSelectFile?: (file: FileRecord) => void;
  }

  let {
    files = [],
    loading = false,
    onSelectFile,
  }: $$Props = $props();

  let viewMode = $state<'grid' | 'list'>('grid');

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function formatDate(dateStr: string | number | undefined): string {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: '2-digit' });
  }

  function handleSelectFile(file: FileRecord) {
    if (onSelectFile) {
      onSelectFile(file);
    }
  }
</script>

<div class="file-grid-container">
  <div class="toolbar">
    <span class="file-count">{files.length} file{files.length !== 1 ? 's' : ''}</span>
    <div class="view-toggle">
      <button
        type="button"
        class="toggle-btn"
        class:active={viewMode === 'grid'}
        onclick={() => (viewMode = 'grid')}
        title="Grid view"
      >
        <Icons.Grid2x2 size={14} />
      </button>
      <button
        type="button"
        class="toggle-btn"
        class:active={viewMode === 'list'}
        onclick={() => (viewMode = 'list')}
        title="List view"
      >
        <Icons.List size={14} />
      </button>
    </div>
  </div>

  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <span>Loading files...</span>
    </div>
  {:else if files.length === 0}
    <div class="empty">
      <Icons.Inbox size={40} />
      <h3>No files yet</h3>
      <p>Upload files to get started</p>
    </div>
  {:else if viewMode === 'grid'}
    <div class="grid">
      {#each files as file (file.id)}
        <div
          class="file-card"
          role="button"
          tabindex="0"
          onclick={() => handleSelectFile(file)}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              handleSelectFile(file);
            }
          }}
        >
          <div class="file-icon">
            <FileIcon name={file.name} mimeType={file.mime_type} size={40} />
          </div>
          <div class="file-info">
            <div class="file-name" title={file.name}>{file.name}</div>
            <div class="file-size">{formatSize(file.size_bytes)}</div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="list">
      {#each files as file (file.id)}
        <div
          class="list-item"
          role="button"
          tabindex="0"
          onclick={() => handleSelectFile(file)}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              handleSelectFile(file);
            }
          }}
        >
          <div class="item-icon">
            <FileIcon name={file.name} mimeType={file.mime_type} size={24} />
          </div>
          <div class="item-name" title={file.name}>{file.name}</div>
          <span class="item-type">{file.mime_type}</span>
          <span class="item-size">{formatSize(file.size_bytes)}</span>
          <span class="item-date">{formatDate(file.updated_at)}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .file-grid-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    gap: 16px;
  }

  .file-count {
    font-size: var(--fs-12);
    color: var(--text-2);
    font-weight: 500;
  }

  .view-toggle {
    display: flex;
    gap: 2px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    padding: 2px;
    background: var(--surface-2);
  }

  .toggle-btn {
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
    border-radius: 3px;
    transition: all 0.15s;
  }

  .toggle-btn:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .toggle-btn.active {
    background: var(--surface);
    color: var(--text);
  }

  .loading,
  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
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

  .empty h3 {
    margin: 0;
    font-size: var(--fs-14);
    color: var(--text-2);
  }

  .empty p {
    margin: 0;
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .grid {
    flex: 1;
    overflow: auto;
    padding: 16px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: 12px;
  }

  .file-card {
    padding: 12px;
    border-radius: var(--r-3);
    border: 1px solid var(--border);
    background: var(--surface);
    cursor: pointer;
    transition: all 0.15s;
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: center;
    text-align: center;
  }

  .file-card:hover {
    background: var(--surface-2);
    border-color: var(--blue-subtle);
  }

  .file-card:focus-visible {
    outline: 2px solid var(--blue);
    outline-offset: 2px;
  }

  .file-icon {
    display: flex;
    justify-content: center;
    align-items: center;
    color: var(--muted);
  }

  .file-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1;
  }

  .file-name {
    font-size: var(--fs-12);
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-size {
    font-size: 11px;
    color: var(--muted);
  }

  .list {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
  }

  .list-item {
    padding: 12px 16px;
    border-bottom: 1px solid var(--hairline);
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .list-item:hover {
    background: var(--surface-2);
  }

  .list-item:focus-visible {
    outline: 2px solid var(--blue);
    outline-offset: -2px;
  }

  .item-icon {
    display: flex;
    color: var(--muted);
    flex-shrink: 0;
  }

  .item-name {
    font-size: var(--fs-13);
    color: var(--text);
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-type {
    font-size: var(--fs-11);
    color: var(--text-2);
    min-width: 120px;
    text-align: right;
  }

  .item-size {
    font-size: var(--fs-12);
    color: var(--text-2);
    min-width: 60px;
    text-align: right;
  }

  .item-date {
    font-size: var(--fs-12);
    color: var(--muted);
    min-width: 80px;
    text-align: right;
    flex-shrink: 0;
  }
</style>
