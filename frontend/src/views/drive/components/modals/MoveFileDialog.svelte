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
  let selectedFolder = $state<string | null>(null);

  function resetForm() {
    selectedFolder = null;
  }

  async function handleMove() {
    if (!file || selectedFolder === null) return;
    try {
      await onMove(file.id, selectedFolder);
      resetForm();
      onClose();
    } catch {
      // Error handled by caller
    }
  }

  // Sort folders and build tree structure with depth info
  let sortedFolders = $derived(
    folders.slice().sort((a, b) => a.localeCompare(b))
  );

  function getDepth(path: string): number {
    return path.split('/').length - 1;
  }

  function getFolderName(path: string): string {
    return path.split('/').pop() || path;
  }

  $effect(() => {
    if (!isOpen) {
      resetForm();
    }
  });
</script>

{#if isOpen && file}
  <div
    class="dialog-overlay"
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && onClose()}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  >
    <div class="dialog">
      <div class="dialog-header">
        <h3>Move: <strong>{file.name}</strong></h3>
        <button type="button" class="close-btn" onclick={onClose}>
          <Icons.X size={18} />
        </button>
      </div>

      <div class="dialog-content">
        <div class="current-location">
          <Icons.MapPin size={13} />
          <span>Currently in: <strong>{file.folder_path || 'Root /'}</strong></span>
        </div>

        <div class="folder-tree">
          <!-- Move to root -->
          <button
            type="button"
            class="folder-item"
            class:selected={selectedFolder === ''}
            class:current={file.folder_path === '' || !file.folder_path}
            disabled={file.folder_path === '' || !file.folder_path}
            onclick={() => selectedFolder = ''}
          >
            <Icons.HardDrive size={15} />
            <span class="folder-label">Root /</span>
            {#if file.folder_path === '' || !file.folder_path}
              <span class="current-tag">current</span>
            {/if}
          </button>

          {#each sortedFolders as folder}
            {@const depth = getDepth(folder)}
            {@const name = getFolderName(folder)}
            {@const isCurrent = file.folder_path === folder}
            <button
              type="button"
              class="folder-item"
              class:selected={selectedFolder === folder}
              class:current={isCurrent}
              disabled={isCurrent}
              style="padding-left: {12 + depth * 16}px"
              onclick={() => selectedFolder = folder}
            >
              <Icons.Folder size={15} />
              <span class="folder-label">{name}</span>
              {#if isCurrent}
                <span class="current-tag">current</span>
              {/if}
            </button>
          {/each}
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
          disabled={selectedFolder === null || isMoving}
        >
          {#if isMoving}
            <div class="spinner"></div>
            Moving…
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
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    z-index: 2000;
    backdrop-filter: blur(2px);
  }

  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3, 10px);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
    width: 90%;
    max-width: 420px;
    display: flex;
    flex-direction: column;
    max-height: 80vh;
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .dialog-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dialog-header strong { color: var(--blue); }

  .close-btn {
    flex-shrink: 0;
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    border-radius: 7px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .close-btn:hover { background: var(--surface-2); color: var(--text); }

  .dialog-content {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .current-location {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 10px;
    border-radius: 6px;
    background: rgba(110, 168, 255, 0.06);
    border: 1px solid rgba(110, 168, 255, 0.15);
    font-size: 12px;
    color: var(--text-2);
    flex-shrink: 0;
  }
  .current-location strong { color: var(--text); }

  .folder-tree {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .folder-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 6px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-2);
    text-align: left;
    cursor: pointer;
    transition: all 0.12s;
    font-size: 13px;
    width: 100%;
  }
  .folder-item:hover:not(:disabled) {
    background: var(--surface-2);
    color: var(--text);
    border-color: var(--border);
  }
  .folder-item.selected {
    background: rgba(110, 168, 255, 0.12);
    border-color: rgba(110, 168, 255, 0.3);
    color: var(--text);
  }
  .folder-item.current, .folder-item:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .folder-label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .current-tag {
    font-size: 10px;
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 4px;
    background: var(--surface-3);
    color: var(--muted);
    flex-shrink: 0;
    letter-spacing: 0.3px;
  }

  .dialog-actions {
    display: flex;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    justify-content: flex-end;
    flex-shrink: 0;
  }

  .btn-secondary,
  .btn-primary {
    height: 36px;
    padding: 0 16px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s;
  }
  .btn-secondary:hover { background: var(--surface); color: var(--text); }

  .btn-primary {
    background: rgba(110, 168, 255, 0.15);
    border-color: rgba(110, 168, 255, 0.3);
    color: var(--text);
  }
  .btn-primary:hover:not(:disabled) { background: rgba(110, 168, 255, 0.25); }
  .btn-primary:disabled, .btn-secondary:disabled { opacity: 0.5; cursor: not-allowed; }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
