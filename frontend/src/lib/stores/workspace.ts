import { writable, derived, get } from 'svelte/store';
import { api, type Workspace, type WorkspaceFileEntry } from '../api';

export const workspaces = writable<Workspace[]>([]);
export const activeWorkspaceId = writable<string | null>(null);
export const isLoading = writable(false);
export const isSaving = writable(false);

// Filesystem mode stores
export const fileTree = writable<WorkspaceFileEntry[]>([]);
export const activeFilePath = writable<string | null>(null);
export const fileContent = writable<string>('');
export const isDirty = writable(false);
export const isFileLoading = writable(false);
export const fileError = writable<string | null>(null);

export const activeWorkspace = derived(
  [workspaces, activeWorkspaceId],
  ([$workspaces, $id]) => $workspaces.find((w) => w.id === $id) || null
);

export async function loadWorkspaces() {
  isLoading.set(true);
  try {
    const data = await api.listWorkspaces();
    workspaces.set(data.workspaces || []);
  } finally {
    isLoading.set(false);
  }
}

export function setActiveWorkspace(id: string | null) {
  activeWorkspaceId.set(id);
}

export async function createNewWorkspace() {
  try {
    const newWorkspace = await api.createWorkspace({
      name: 'untitled',
      language: 'text',
      body: '',
    });
    workspaces.update(w => [newWorkspace, ...w]);
    activeWorkspaceId.set(newWorkspace.id);
    return newWorkspace;
  } catch (err) {
    console.error('Failed to create workspace:', err);
    throw err;
  }
}

export async function updateActiveWorkspace(updates: Partial<Workspace>) {
  const id = get(activeWorkspaceId);

  if (!id) return;

  isSaving.set(true);
  try {
    const updated = await api.updateWorkspace(id, updates);
    workspaces.update(w => w.map(workspace => workspace.id === id ? updated : workspace));
  } catch (err) {
    console.error('Failed to update workspace:', err);
    throw err;
  } finally {
    isSaving.set(false);
  }
}

export async function deleteWorkspace(id: string) {
  try {
    await api.deleteWorkspace(id);
    workspaces.update(w => w.filter(workspace => workspace.id !== id));
    if (id === get(activeWorkspaceId)) {
      activeWorkspaceId.set(null);
    }
  } catch (err) {
    console.error('Failed to delete workspace:', err);
    throw err;
  }
}

// Filesystem operations
export async function loadFileTree(workspaceId: string, path?: string) {
  isFileLoading.set(true);
  fileError.set(null);
  try {
    const entries = await api.listWorkspaceFiles(workspaceId, path);
    fileTree.set(entries.entries || []);
    return entries.entries || [];
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Failed to load file tree';
    fileError.set(message);
    console.error('Failed to load file tree:', err);
    throw err;
  } finally {
    isFileLoading.set(false);
  }
}

export async function openFile(workspaceId: string, path: string) {
  activeFilePath.set(path);
  isFileLoading.set(true);
  fileError.set(null);
  isDirty.set(false);
  try {
    const response = await api.readWorkspaceFile(workspaceId, path);
    fileContent.set(response.content || '');
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Failed to load file';
    fileError.set(message);
    console.error('Failed to open file:', err);
    throw err;
  } finally {
    isFileLoading.set(false);
  }
}

export async function saveFile(workspaceId: string, path: string, content: string) {
  isSaving.set(true);
  fileError.set(null);
  try {
    await api.writeWorkspaceFile(workspaceId, path, content);
    fileContent.set(content);
    isDirty.set(false);
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Failed to save file';
    fileError.set(message);
    console.error('Failed to save file:', err);
    throw err;
  } finally {
    isSaving.set(false);
  }
}

export function markFileDirty() {
  isDirty.set(true);
}

export function clearFileState() {
  activeFilePath.set(null);
  fileContent.set('');
  isDirty.set(false);
  fileError.set(null);
}
