<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord } from '$lib/api';

  interface Props {
    file: FileRecord | null;
    isOpen: boolean;
    isRenaming?: boolean;
    onRename: (file: FileRecord, newName: string) => Promise<void>;
    onClose: () => void;
  }

  let { file, isOpen, isRenaming = false, onRename, onClose }: Props = $props();
  let nextName = $state('');

  $effect(() => {
    if (isOpen && file) {
      nextName = file.name;
    }
  });

  // Focus and pre-select the basename (everything before the last extension)
  // so the user types over the name but keeps the extension by default.
  function focusName(node: HTMLInputElement) {
    queueMicrotask(() => {
      node.focus();
      const dot = node.value.lastIndexOf('.');
      if (dot > 0) {
        node.setSelectionRange(0, dot);
      } else {
        node.select();
      }
    });
  }

  async function handleRename() {
    if (!file || !nextName.trim() || nextName === file.name) return;
    try {
      await onRename(file, nextName.trim());
      onClose();
    } catch {
      // Error handled by service
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleRename();
    } else if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

{#if isOpen && file}
  <div class="dialog-overlay" role="presentation" onclick={(e) => e.target === e.currentTarget && onClose()}>
    <div class="dialog">
      <div class="dialog-header">
        <h3>Rename: <strong>{file.name}</strong></h3>
        <button type="button" class="close-btn" onclick={onClose}>
          <Icons.X size={18} />
        </button>
      </div>

      <div class="dialog-content">
        <div class="form-group">
          <label for="rename-input">New Name</label>
          <input
            id="rename-input"
            type="text"
            bind:value={nextName}
            onkeydown={handleKeydown}
            class="name-input"
            use:focusName
          />
        </div>
      </div>

      <div class="dialog-actions">
        <button type="button" class="btn-secondary" onclick={onClose} disabled={isRenaming}>
          Cancel
        </button>
        <button
          type="button"
          class="btn-primary"
          onclick={handleRename}
          disabled={!nextName.trim() || nextName === file.name || isRenaming}
        >
          {#if isRenaming}
            <div class="spinner"></div>
            Renaming...
          {:else}
            Rename
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
    background: rgba(0, 0, 0, 0.6);
    z-index: 2000;
  }

  .dialog {
    background: var(--surface);
    border-radius: var(--r-3);
    box-shadow: var(--shadow-modal);
    width: 90%;
    max-width: 400px;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-4) var(--s-6);
    border-bottom: 1px solid var(--border);
  }

  .dialog-header h3 {
    margin: 0;
    font-size: var(--fs-16);
    font-weight: 600;
  }

  .dialog-header strong {
    color: var(--blue);
    font-weight: 700;
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
    border-radius: var(--r-2);
  }

  .close-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .dialog-content {
    padding: var(--s-6);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .form-group label {
    font-size: var(--fs-12);
    font-weight: 600;
    color: var(--text-2);
    text-transform: uppercase;
  }

  .name-input {
    width: 100%;
    padding: var(--s-3) var(--s-4);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
    color: var(--text);
    font-size: var(--fs-14);
    outline: none;
  }

  .name-input:focus {
    border-color: var(--blue);
    background: var(--bg);
  }

  .dialog-actions {
    display: flex;
    gap: var(--s-3);
    padding: var(--s-4) var(--s-6);
    border-top: 1px solid var(--border);
    background: var(--surface-2);
    border-bottom-left-radius: var(--r-3);
    border-bottom-right-radius: var(--r-3);
  }

  .btn-primary, .btn-secondary {
    flex: 1;
    padding: var(--s-2) var(--s-4);
    border-radius: var(--r-2);
    font-size: var(--fs-14);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    height: 40px;
  }

  .btn-primary {
    background: var(--blue);
    color: white;
    border: none;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-secondary {
    background: transparent;
    color: var(--text-2);
    border: 1px solid var(--border);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--surface-3);
    color: var(--text);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin { to { transform: rotate(360deg); } }
</style>
