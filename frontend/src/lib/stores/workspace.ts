import { writable, derived, get } from 'svelte/store';
import { api, type Workspace } from '../api';

export const workspaces = writable<Workspace[]>([]);
export const activeWorkspaceId = writable<string | null>(null);
export const isLoading = writable(false);
export const isSaving = writable(false);

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
