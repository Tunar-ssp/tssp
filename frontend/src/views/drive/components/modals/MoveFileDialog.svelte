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
  let newFolderName = $state('');
  let showNewFolderInput = $state(false);

  function autofocusNode(node: HTMLInputElement) {
    queueMicrotask(() => node.focus());
  }

  function resetForm() {
    selectedFolder = '';
    newFolderName = '';
    showNewFolderInput = false;
  }

  async function handleMove() {
    if (!file) return;

    let targetPath = selectedFolder;
    if (showNewFolderInput && newFolderName.trim()) {
      targetPath = selectedFolder ? `${selectedFolder}/${newFolderName.trim()}` : newFolderName.trim();
    }

    if (!targetPath) return;

    try {
      await onMove(file.id, targetPath);
      resetForm();
      onClose();
    } catch {
      // Error already handled by service
    }
  }

  let availableFolders = $derived(
    folders.filter(f => !f.includes(file?.folder_path || '')).sort()
  );

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
        <h3>Move: <strong>{file.name}</strong></h3>
        <button type="button" class="close-btn" onclick={onClose}>
          <Icons.X size={18} />
        </button>
      </div>

      <div class="dialog-content">
        <div class="folder-tree">
          <button
            type="button"
            class="folder-item"
            class:selected={selectedFolder === ''}
            onclick={() => { selectedFolder = ''; showNewFolderInput = false; }}
          >
            <Icons.HardDrive size={16} />
            <span>Root (Bucket)</span>
          </button>

          {#each availableFolders as folder}
            <button
              type="button"
              class="folder-item"
              class:selected={selectedFolder === folder}
              onclick={() => { selectedFolder = folder; showNewFolderInput = false; }}
            >
              <Icons.Folder size={16} />
              <span>{folder}</span>
            </button>
          {/each}
        </div>

        <div class="divider">OR</div>

        <div class="new-folder-section">
          <button
            type="button"
            class="new-folder-btn"
            class:active={showNewFolderInput}
            onclick={() => showNewFolderInput = !showNewFolderInput}
          >
            <Icons.FolderPlus size={16} />
            Create New Folder
          </button>

          {#if showNewFolderInput}
            <div class="new-folder-input-group">
              {#if selectedFolder}
                <div class="folder-path">{selectedFolder} /</div>
              {/if}
              <input
                type="text"
                bind:value={newFolderName}
                placeholder="folder name"
                class="new-folder-input"
                use:autofocusNode
              />
            </div>
          {/if}
        </div>

        <div class="current-location">
          <span class="label">Current Location:</span>
          <span class="value">{file.folder_path || 'Root'}</span>
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
          disabled={!selectedFolder && !(showNewFolderInput && newFolderName.trim()) || isMoving}
        >
          {#if isMoving}
            <div class="spinner"></div>
            Moving...
          {:else}
            <Icons.ArrowRight size={14} />
            Move Here
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
    max-width: 500px;
    display: flex;
    flex-direction: column;
    max-height: 80vh;
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

  .dialog-header strong {
    color: var(--blue);
  }

  .close-btn {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    border-radius: 8px;
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .dialog-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-4);
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
  }

  .folder-tree {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .folder-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-2);
    text-align: left;
    cursor: pointer;
    transition: all 0.2s;
  }

  .folder-item:hover {
    background: var(--surface-2);
    color: var(--text);
    border-color: var(--border);
  }

  .folder-item.selected {
    background: rgba(110, 168, 255, 0.15);
    border-color: rgba(110, 168, 255, 0.3);
    color: var(--text);
  }

  .folder-item span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .divider {
    text-align: center;
    color: var(--text-3);
    font-size: var(--fs-12);
    padding: var(--s-2) 0;
    opacity: 0.6;
  }

  .new-folder-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .new-folder-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    transition: all 0.2s;
  }

  .new-folder-btn:hover,
  .new-folder-btn.active {
    background: var(--surface-2);
    color: var(--text);
    border-color: rgba(110, 168, 255, 0.3);
  }

  .new-folder-input-group {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--surface-2);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .folder-path {
    color: var(--text-2);
    font-family: var(--ff-mono);
    font-size: var(--fs-12);
    white-space: nowrap;
  }

  .new-folder-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-family: var(--ff-mono);
    font-size: var(--fs-13);
    outline: none;
  }

  .new-folder-input::placeholder {
    color: var(--text-3);
  }

  .current-location {
    padding: var(--s-3);
    border-radius: 8px;
    background: rgba(110, 168, 255, 0.05);
    border: 1px solid rgba(110, 168, 255, 0.15);
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: var(--fs-13);
  }

  .current-location .label {
    color: var(--text-2);
    font-weight: 600;
  }

  .current-location .value {
    color: var(--text);
    font-family: var(--ff-mono);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dialog-actions {
    display: flex;
    gap: 10px;
    padding: var(--s-4);
    border-top: 1px solid var(--border);
    justify-content: flex-end;
  }

  .btn-secondary,
  .btn-primary {
    height: 40px;
    padding: 0 20px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    transition: all 0.2s;
  }

  .btn-secondary:hover {
    background: var(--surface);
    color: var(--text);
  }

  .btn-primary {
    background: rgba(110, 168, 255, 0.15);
    border-color: rgba(110, 168, 255, 0.3);
    color: var(--text);
  }

  .btn-primary:hover:not(:disabled) {
    background: rgba(110, 168, 255, 0.25);
  }

  .btn-primary:disabled,
  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
