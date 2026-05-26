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
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .editor-sidebar {
    width: 250px;
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    overflow-y: auto;
  }

  .sidebar-header {
    padding: 12px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .sidebar-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .sidebar-actions {
    display: flex;
    gap: 4px;
  }

  .action-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 4px;
    transition: all 0.2s;
  }

  .action-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .create-form {
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .create-form input {
    padding: 6px 8px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 12px;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .create-form input:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .create-actions {
    display: flex;
    gap: 4px;
  }

  .create-actions button {
    flex: 1;
    padding: 4px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    background: var(--bg-primary);
    color: var(--text-primary);
    transition: all 0.2s;
  }

  .create-actions button:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
  }

  .loading,
  .error-message,
  .empty-state {
    padding: 16px 12px;
    text-align: center;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .error-message {
    color: var(--error-color);
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
  }

  .editor-header {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .file-info {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  .file-name {
    font-weight: 500;
  }

  .dirty-indicator {
    color: var(--accent-color);
    font-size: 12px;
  }

  .empty-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 12px;
    opacity: 0.5;
  }

  .empty-text {
    font-size: 14px;
  }
</style>
