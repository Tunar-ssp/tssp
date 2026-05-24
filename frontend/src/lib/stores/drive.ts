import { writable, derived } from 'svelte/store';
import { api, type FileRecord } from '../api';

export const files = writable<FileRecord[]>([]);
export const selectedIds = writable<Set<string>>(new Set());
export const currentFolder = writable<string>('');
export const isLoading = writable(false);
export const folders = writable<string[]>([]);

export const selectedCount = derived(selectedIds, ($ids) => $ids.size);

export const visibleFiles = derived(
  [files, currentFolder],
  ([$files, $folder]) =>
    $files.filter(f => (f.folder_path || '') === $folder)
      .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
);

export async function loadFiles(folder?: string) {
  isLoading.set(true);
  try {
    const data = await api.listFiles(500);
    files.set(data.files || []);

    // Extract unique folders
    const folderSet = new Set(data.files?.map(f => f.folder_path || '') || []);
    folders.set(Array.from(folderSet).sort());

    if (folder !== undefined) {
      currentFolder.set(folder);
    }
  } finally {
    isLoading.set(false);
  }
}

export function toggleSelect(id: string) {
  selectedIds.update(s => {
    const newSet = new Set(s);
    if (newSet.has(id)) newSet.delete(id);
    else newSet.add(id);
    return newSet;
  });
}

export function selectAll(ids: string[]) {
  selectedIds.set(new Set(ids));
}

export function clearSelection() {
  selectedIds.set(new Set());
}

export function setFolder(path: string) {
  currentFolder.set(path);
  clearSelection();
}
