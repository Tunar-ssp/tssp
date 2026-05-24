<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord } from '$lib/api';
  import FileIcon from './FileIcon.svelte';
  import { selectedIds, toggleSelect, selectAll, clearSelection } from '$lib/stores/drive';

  export let files: FileRecord[] = [];
  export let loading: boolean = false;

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: '2-digit' });
  }

  function handleSelectAll() {
    selectAll(files.map(f => f.id));
  }

  function handleClearSelection() {
    clearSelection();
  }

  let viewMode: 'grid' | 'list' = 'grid';
</script>

<div class="file-grid-container">
  {#if files.length > 0}
    <div class="toolbar">
      <div class="toolbar-left">
        <span class="file-count">{files.length} file{files.length !== 1 ? 's' : ''}</span>
        {#if $selectedIds.size > 0}
          <span class="selected-count">{$selectedIds.size} selected</span>
        {/if}
      </div>

      <div class="toolbar-right">
        {#if $selectedIds.size > 0}
          <button class="toolbar-btn" on:click={handleClearSelection}>
            <Icons.X size={14} />
          </button>
        {/if}

        <button class="toolbar-btn" on:click={handleSelectAll}>
          <Icons.CheckSquare size={14} />
        </button>

        <div class="view-toggle">
          <button
            class="toggle-btn"
            class:active={viewMode === 'grid'}
            on:click={() => (viewMode = 'grid')}
          >
            <Icons.Grid2x2 size={14} />
          </button>
          <button
            class="toggle-btn"
            class:active={viewMode === 'list'}
            on:click={() => (viewMode = 'list')}
          >
            <Icons.List size={14} />
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if loading}
    <div class="loading">
      <div class="spinner" />
      <span>Loading files...</span>
    </div>
  {:else if files.length === 0}
    <div class="empty">
      <Icons.Inbox size={40} />
      <h3>No files yet</h3>
      <p>Drag files here or use the upload button</p>
    </div>
  {:else if viewMode === 'grid'}
    <div class="grid">
      {#each files as file (file.id)}
        <div
          class="file-card"
          class:selected={$selectedIds.has(file.id)}
          on:click={() => toggleSelect(file.id)}
          role="button"
          tabindex="0"
        >
          <div class="file-header">
            <div class="checkbox">
              <input
                type="checkbox"
                checked={$selectedIds.has(file.id)}
                on:change={() => toggleSelect(file.id)}
              />
            </div>
            <button class="menu-btn" on:click|stopPropagation>
              <Icons.MoreVertical size={14} />
            </button>
          </div>

          <div class="file-icon">
            <FileIcon mimeType={file.mime_type} name={file.name} size={32} />
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
          class:selected={$selectedIds.has(file.id)}
          on:click={() => toggleSelect(file.id)}
          role="button"
          tabindex="0"
        >
          <div class="item-left">
            <input
              type="checkbox"
              checked={$selectedIds.has(file.id)}
              on:change={() => toggleSelect(file.id)}
            />
            <div class="item-icon">
              <FileIcon mimeType={file.mime_type} name={file.name} size={20} />
            </div>
            <div class="item-name" title={file.name}>{file.name}</div>
          </div>

          <div class="item-right">
            <span class="item-size">{formatSize(file.size_bytes)}</span>
            <span class="item-date">{formatDate(file.updated_at)}</span>
            <button class="menu-btn" on:click|stopPropagation>
              <Icons.MoreVertical size={14} />
            </button>
          </div>
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

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .file-count {
    font-weight: 500;
  }

  .selected-count {
    color: var(--blue);
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .toolbar-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border-radius: var(--r-2);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .toolbar-btn:hover {
    background: var(--surface-hi);
    color: var(--text);
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
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
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
  }

  .file-card:hover {
    background: var(--surface-2);
    border-color: var(--border-2);
  }

  .file-card.selected {
    background: var(--blue-soft);
    border-color: var(--blue);
  }

  .file-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .file-card:hover .file-header,
  .file-card.selected .file-header {
    opacity: 1;
  }

  .checkbox {
    display: flex;
  }

  .checkbox input {
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .menu-btn {
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

  .menu-btn:hover {
    background: var(--surface-3);
    color: var(--text);
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

  /* List view */
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
    justify-content: space-between;
    gap: 16px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .list-item:hover {
    background: var(--surface-2);
  }

  .list-item.selected {
    background: var(--blue-soft);
  }

  .item-left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 0;
  }

  .item-left input {
    width: 18px;
    height: 18px;
    cursor: pointer;
    flex-shrink: 0;
  }

  .item-icon {
    display: flex;
    color: var(--muted);
    flex-shrink: 0;
  }

  .item-name {
    font-size: var(--fs-13);
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-right {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-shrink: 0;
  }

  .item-size,
  .item-date {
    font-size: var(--fs-12);
    color: var(--text-2);
    min-width: 80px;
    text-align: right;
  }
</style>
