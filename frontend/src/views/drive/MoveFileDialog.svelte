<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord } from '$lib/api';

  interface Props {
    file: FileRecord | null;
    folders: string[];
    isOpen: boolean;
    isMoving?: boolean;
    onMove: (fileId: string, folderPath: string) => Promise<void>;
    onClose: () => void;
  }

  let { file, folders, isOpen, isMoving = false, onMove, onClose }: Props = $props();
  let selectedFolder = $state('');
  let customPath = $state('');

  function resetForm() {
    selectedFolder = '';
    customPath = '';
  }

  async function handleMove() {
    if (!file) return;
    const targetPath = selectedFolder || customPath.trim();
    if (!targetPath) return;

    try {
      await onMove(file.id, targetPath);
      resetForm();
      onClose();
    } catch {
      // Error already handled by service
    }
  }

  $effect(() => {
    if (!isOpen) {
      resetForm();
    }
  });
</script>

{#if isOpen && file}
  <div class="dialog-overlay" role="presentation" onclick={(e) => e.target === e.currentTarget && onClose()} onkeydown={(e) => e.key === 'Escape' && onClose()}>
    <div class="dialog">
      <div class="dialog-header">
        <h3>Move File</h3>
        <button type="button" class="close-btn" onclick={onClose}>
          <Icons.X size={18} />
        </button>
      </div>

      <div class="dialog-content">
        <p class="file-name">Moving: <strong>{file.name}</strong></p>

        {#if folders.length > 0}
          <div class="form-group">
            <label for="folder-select">Existing Folders</label>
            <select id="folder-select" bind:value={selectedFolder} class="folder-select">
              <option value="">Select a folder...</option>
              <option value="">Bucket root</option>
              {#each folders as folder}
                <option value={folder}>{folder}</option>
              {/each}
            </select>
          </div>

          <div class="divider">OR</div>
        {/if}

        <div class="form-group">
          <label for="custom-path">Custom Path</label>
          <input
            id="custom-path"
            type="text"
            bind:value={customPath}
            placeholder="e.g. /documents/archive"
            class="path-input"
            disabled={!!selectedFolder}
          />
        </div>

        <div class="current-path">
          <span class="label">Current:</span>
          <span class="value">{file.folder_path || 'Bucket root'}</span>
        </div>
      </div>

      <div class="dialog-actions">
        <button type="button" class="btn-secondary" onclick={onClose} disabled={isMoving}>
          Cancel
        </button>
        <button
          type="button"
          class="btn-primary"
          onclick={handleMove}
          disabled={!selectedFolder && !customPath.trim() || isMoving}
        >
          {#if isMoving}
            <div class="spinner"></div>
            Moving...
          {:else}
            <Icons.FolderOpen size={14} />
            Move File
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    z-index: 1000;
  }

  .dialog {
    background: var(--surface);
    border-radius: var(--r-3);
    box-shadow: 0 20px 25px rgba(0, 0, 0, 0.15);
    width: 90%;
    max-width: 400px;
    display: flex;
    flex-direction: column;
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
  }

  .dialog-header h3 {
    margin: 0;
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text);
  }

  .close-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .close-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .dialog-content {
    padding: var(--s-4);
    flex: 1;
    overflow-y: auto;
  }

  .file-name {
    margin: 0 0 var(--s-4) 0;
    padding: var(--s-3);
    background: var(--surface-2);
    border-radius: var(--r-2);
    font-size: var(--fs-13);
    color: var(--text-2);
  }

  .form-group {
    margin-bottom: var(--s-4);
  }

  .form-group label {
    display: block;
    margin-bottom: var(--s-2);
    font-size: var(--fs-12);
    font-weight: 600;
    text-transform: uppercase;
    color: var(--muted);
  }

  .folder-select,
  .path-input {
    width: 100%;
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
    color: var(--text);
    font-size: var(--fs-13);
    outline: none;
  }

  .folder-select:focus,
  .path-input:focus {
    border-color: var(--blue);
  }

  .folder-select:disabled,
  .path-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .divider {
    text-align: center;
    padding: var(--s-3) 0;
    color: var(--muted);
    font-size: var(--fs-12);
    position: relative;
  }

  .divider::before,
  .divider::after {
    content: '';
    position: absolute;
    top: 50%;
    width: 45%;
    height: 1px;
    background: var(--border);
  }

  .divider::before {
    left: 0;
  }

  .divider::after {
    right: 0;
  }

  .current-path {
    padding: var(--s-3);
    background: var(--surface-2);
    border-radius: var(--r-2);
    font-size: var(--fs-12);
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .current-path .label {
    color: var(--muted);
    font-weight: 500;
  }

  .current-path .value {
    color: var(--text);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dialog-actions {
    display: flex;
    gap: var(--s-2);
    padding: var(--s-4);
    border-top: 1px solid var(--border);
    background: var(--bg);
  }

  .btn-primary,
  .btn-secondary {
    flex: 1;
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    font-size: var(--fs-13);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
  }

  .btn-primary {
    background: var(--blue);
    color: white;
    border-color: var(--blue);
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--blue-dark, var(--blue));
  }

  .btn-secondary {
    background: var(--surface-2);
    color: var(--text);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--surface-3);
  }

  .btn-primary:disabled,
  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
