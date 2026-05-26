<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import {
    fileTree,
    activeFilePath,
    fileContent,
    isDirty,
    isFileLoading,
    fileError,
    loadFileTree,
    openFile,
    saveFile,
    markFileDirty,
    clearFileState
  } from '$lib/stores/workspace';
  import { success, error } from '$lib/stores/notifications';
  import MonacoEditor from '$lib/components/MonacoEditor.svelte';
  import { createWorkspaceFile, createWorkspaceDirectory, deleteWorkspaceFile } from '$lib/services/workspaceService';
  import FileTreeItem from './FileTreeItem.svelte';

  interface Props {
    workspaceId: string;
  }

  let { workspaceId }: Props = $props();

  let createMode = $state<'file' | 'folder' | null>(null);
  let createPath = $state('');
  let expandedDirs = $state<Record<string, boolean>>({});
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(async () => {
    try {
      await loadFileTree(workspaceId);
    } catch (err) {
      error('Load Failed', err instanceof Error ? err.message : 'Failed to load file tree');
    }
  });

  onDestroy(() => {
    if (saveTimer) clearTimeout(saveTimer);
  });

  function getLanguageFromPath(path: string): string {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    const langMap: Record<string, string> = {
      'js': 'javascript', 'ts': 'typescript', 'py': 'python', 'rs': 'rust',
      'go': 'go', 'md': 'markdown', 'html': 'html', 'css': 'css',
      'sql': 'sql', 'json': 'json', 'yaml': 'yaml', 'sh': 'bash',
      'txt': 'text', 'jsx': 'javascript', 'tsx': 'typescript'
    };
    return langMap[ext] || 'text';
  }

  function getFileName(path: string): string {
    return path.split('/').pop() || path;
  }

  function handleSelectFile(path: string) {
    if ($activeFilePath === path) return;
    void openFile(workspaceId, path);
  }

  function handleEditorInput(newValue: string) {
    fileContent.set(newValue);
    markFileDirty();
    scheduleFileSave();
  }

  function scheduleFileSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      void handleSaveFile(false);
    }, 900);
  }

  async function handleSaveFile(showToast = true) {
    if (!$activeFilePath) {
      error('Save Failed', 'No file selected');
      return;
    }

    try {
      await saveFile(workspaceId, $activeFilePath, $fileContent);
      if (showToast) success('File Saved', `${$activeFilePath} saved`);
    } catch (err) {
      error('Save Failed', err instanceof Error ? err.message : 'Failed to save file');
    }
  }

  function toggleDir(dirPath: string) {
    expandedDirs[dirPath] = !expandedDirs[dirPath];
  }

  async function handleCreateFile() {
    if (!createPath.trim()) {
      error('Invalid Path', 'Path cannot be empty');
      return;
    }

    try {
      await createWorkspaceFile(workspaceId, createPath, '');
      await loadFileTree(workspaceId);
      success('File Created', createPath);
      createPath = '';
      createMode = null;
      handleSelectFile(createPath);
    } catch (err) {
      error('Create Failed', err instanceof Error ? err.message : 'Failed to create file');
    }
  }

  async function handleCreateFolder() {
    if (!createPath.trim()) {
      error('Invalid Path', 'Path cannot be empty');
      return;
    }

    try {
      await createWorkspaceDirectory(workspaceId, createPath);
      await loadFileTree(workspaceId);
      success('Folder Created', createPath);
      createPath = '';
      createMode = null;
    } catch (err) {
      error('Create Failed', err instanceof Error ? err.message : 'Failed to create folder');
    }
  }

  async function handleDeleteFile(path: string) {
    if (!confirm(`Delete "${path}"?`)) return;

    try {
      await deleteWorkspaceFile(workspaceId, path);
      await loadFileTree(workspaceId);
      if ($activeFilePath === path) {
        clearFileState();
      }
      success('File Deleted', path);
    } catch (err) {
      error('Delete Failed', err instanceof Error ? err.message : 'Failed to delete file');
    }
  }
</script>

<div class="workspace-file-editor">
  <div class="editor-sidebar">
    <div class="sidebar-header">
      <h3>Files</h3>
      <div class="sidebar-actions">
        <button
          class="action-btn"
          title="New file"
          onclick={() => (createMode = createMode === 'file' ? null : 'file')}
        >
          <Icons.Plus size={14} />
        </button>
        <button
          class="action-btn"
          title="New folder"
          onclick={() => (createMode = createMode === 'folder' ? null : 'folder')}
        >
          <Icons.FolderPlus size={14} />
        </button>
      </div>
    </div>

    {#if createMode}
      <div class="create-form">
        <input
          type="text"
          placeholder={createMode === 'file' ? 'File path...' : 'Folder path...'}
          bind:value={createPath}
          onkeydown={(e) => {
            if (e.key === 'Enter') {
              if (createMode === 'file') handleCreateFile();
              else handleCreateFolder();
            }
            if (e.key === 'Escape') {
              createMode = null;
              createPath = '';
            }
          }}
        />
        <div class="create-actions">
          <button
            onclick={() => {
              if (createMode === 'file') handleCreateFile();
              else handleCreateFolder();
            }}
          >
            Create
          </button>
          <button onclick={() => (createMode = null)}>Cancel</button>
        </div>
      </div>
    {/if}

    {#if $isFileLoading}
      <div class="loading">Loading files...</div>
    {:else if $fileError}
      <div class="error-message">{$fileError}</div>
    {:else if $fileTree.length === 0}
      <div class="empty-state">No files yet</div>
    {:else}
      <div class="file-tree">
        {#each $fileTree as item (item.path)}
          <FileTreeItem
            {item}
            expanded={expandedDirs[item.path] || false}
            active={$activeFilePath === item.path}
            depth={0}
            onToggle={toggleDir}
            onSelect={handleSelectFile}
            onDelete={handleDeleteFile}
            {expandedDirs}
            activeFilePath={$activeFilePath}
          />
        {/each}
      </div>
    {/if}
  </div>

  <div class="editor-container">
    {#if $activeFilePath}
      <div class="editor-header">
        <div class="file-info">
          <span class="file-name">{$activeFilePath}</span>
          {#if $isDirty}
            <span class="dirty-indicator">●</span>
          {/if}
        </div>
      </div>

      <MonacoEditor
        value={$fileContent}
        language={getLanguageFromPath($activeFilePath)}
        onChange={handleEditorInput}
        height="100%"
        showToolbar={true}
      />
    {:else}
      <div class="empty-editor">
        <div class="empty-icon">📝</div>
        <div class="empty-text">Select a file to edit</div>
      </div>
    {/if}
  </div>
</div>

<style module>
  .workspace-file-editor {
    display: flex;
    height: 100%;
    background: var(--bg);
    color: var(--text);
  }

  .editor-sidebar {
    width: 280px;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    background: var(--surface);
    overflow-y: auto;
  }

  .sidebar-header {
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--surface-2);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .sidebar-header h3 {
    margin: 0;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--muted);
  }

  .sidebar-actions {
    display: flex;
    gap: 6px;
  }

  .action-btn {
    background: none;
    border: none;
    color: var(--text-2);
    cursor: pointer;
    padding: 6px 8px;
    border-radius: 6px;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .create-form {
    padding: 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .create-form input {
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 12px;
    background: var(--bg);
    color: var(--text);
    transition: all 0.15s ease;
  }

  .create-form input:focus {
    outline: none;
    border-color: #0ea5e9;
    box-shadow: 0 0 0 2px rgba(14, 165, 233, 0.1);
  }

  .create-actions {
    display: flex;
    gap: 6px;
  }

  .create-actions button {
    flex: 1;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    background: var(--surface);
    color: var(--text);
    transition: all 0.15s ease;
  }

  .create-actions button:hover {
    background: #0ea5e9;
    border-color: #0ea5e9;
    color: white;
  }

  .loading,
  .error-message,
  .empty-state {
    padding: 24px 16px;
    text-align: center;
    font-size: 12px;
    color: var(--muted);
  }

  .error-message {
    color: #ef4444;
  }

  .file-tree {
    flex: 1;
    padding: 4px 0;
    font-size: 13px;
  }

  .editor-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: var(--bg);
  }

  .editor-header {
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .file-info {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  .file-name {
    font-weight: 500;
    color: var(--text);
  }

  .dirty-indicator {
    color: #f59e0b;
    font-size: 10px;
  }

  .empty-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--muted);
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 12px;
    opacity: 0.4;
  }

  .empty-text {
    font-size: 14px;
    font-weight: 500;
  }
</style>
