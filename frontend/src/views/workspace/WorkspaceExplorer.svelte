<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import VirtualFileTree from './VirtualFileTree.svelte';
  import { success, error } from '$lib/stores/notifications';
  import { createFile, deleteFile, renameFile } from '$lib/services/workspaceFileManager';

  interface $$Props {
    items?: TreeItem[];
    activeFilePath?: string | null;
    onSelectFile?: (path: string) => void;
    onCreateFile?: (path: string, name: string) => void;
    onDeleteFile?: (path: string) => void;
    onRenameFile?: (path: string, newName: string) => void;
  }

  interface TreeItem {
    path: string;
    name: string;
    is_dir: boolean;
    children?: TreeItem[];
    size_bytes?: number;
  }

  let {
    items = [],
    activeFilePath = null,
    onSelectFile = () => {},
    onCreateFile = () => {},
    onDeleteFile = () => {},
    onRenameFile = () => {},
  }: $$Props = $props();

  let showNewFileDialog = $state(false);
  let newFileName = $state('');
  let parentPath = $state('');

  function handleCreateFile() {
    const name = newFileName.trim();
    if (!name) {
      error('Invalid Name', 'File name cannot be empty');
      return;
    }

    try {
      createFile(parentPath ? `${parentPath}/${name}` : name, name);
      onCreateFile(parentPath, name);
      success('File Created', `${name} has been created`);
      showNewFileDialog = false;
      newFileName = '';
      parentPath = '';
    } catch (cause) {
      error('Create Failed', cause instanceof Error ? cause.message : 'Could not create file');
    }
  }

  function handleDeleteFile(path: string) {
    if (!confirm(`Delete "${path}"?`)) return;
    try {
      onDeleteFile(path);
      success('File Deleted', `${path} has been deleted`);
    } catch (cause) {
      error('Delete Failed', cause instanceof Error ? cause.message : 'Could not delete file');
    }
  }

  function handleRenameFile(path: string) {
    const newName = prompt('Rename file', path.split('/').pop())?.trim();
    if (!newName || newName === path.split('/').pop()) return;

    try {
      onRenameFile(path, newName);
      success('File Renamed', `${path} → ${newName}`);
    } catch (cause) {
      error('Rename Failed', cause instanceof Error ? cause.message : 'Could not rename file');
    }
  }

  function openNewFileDialog(path: string) {
    parentPath = path;
    newFileName = '';
    showNewFileDialog = true;
  }
</script>

<div class="workspace-explorer">
  <VirtualFileTree
    {items}
    activeFilePath={activeFilePath}
    onSelectFile={(path) => {
      onSelectFile(path);
      activeFilePath = path;
    }}
    onCreateFile={(path) => openNewFileDialog(path)}
    onDeleteFile={handleDeleteFile}
    onRenameFile={handleRenameFile}
  />

  {#if showNewFileDialog}
    <div class="dialog-overlay" onclick={() => (showNewFileDialog = false)}>
      <div class="dialog" onclick={(e) => e.stopPropagation()}>
        <h3>Create New File</h3>
        <input
          type="text"
          placeholder="File name"
          bind:value={newFileName}
          onkeydown={(e) => {
            if (e.key === 'Enter') handleCreateFile();
            if (e.key === 'Escape') (showNewFileDialog = false);
          }}
          autofocus
        />
        <div class="dialog-actions">
          <button type="button" class="btn btn-primary" onclick={handleCreateFile}>
            Create
          </button>
          <button type="button" class="btn btn-secondary" onclick={() => (showNewFileDialog = false)}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .workspace-explorer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
  }

  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 24px;
    max-width: 400px;
    box-shadow: var(--shadow-card);
  }

  .dialog h3 {
    margin: 0 0 16px;
    font-size: 18px;
    color: var(--text);
  }

  .dialog input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg);
    color: var(--text);
    font-size: 14px;
    margin-bottom: 16px;
  }

  .dialog input:focus {
    outline: none;
    border-color: var(--blue);
  }

  .dialog-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    cursor: pointer;
    font-size: 14px;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .btn-primary {
    background: var(--blue-soft);
    border-color: var(--blue);
    color: var(--blue);
  }

  .btn-primary:hover {
    background: var(--blue);
    color: white;
  }

  .btn-secondary {
    background: var(--surface-2);
  }

  .btn-secondary:hover {
    background: var(--surface-3);
    border-color: var(--border);
  }
</style>
