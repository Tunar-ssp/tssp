import { writable, derived } from 'svelte/store';
import { api, type FileRecord } from '../api';

export const files = writable<FileRecord[]>([]);
export const selectedIds = writable<Set<string>>(new Set());
export const currentFolder = writable<string>('');
export const isLoading = writable(false);
export const folders = writable<string[]>([]);
export const hasMore = writable(false);
export const nextCursor = writable<string | undefined>(undefined);

export const selectedCount = derived(selectedIds, ($ids) => $ids.size);

export const visibleFiles = derived(
  [files, currentFolder],
  ([$files, $folder]) =>
    $files.filter(f => (f.folder_path || '') === $folder)
      .sort((a, b) => (b.updated_at ?? b.uploaded_at) - (a.updated_at ?? a.uploaded_at))
);

export async function loadFiles(folder?: string) {
  isLoading.set(true);
  try {
    const data = await api.listFiles(100);
    files.set(data.files || []);
    hasMore.set(!!data.nextCursor);
    nextCursor.set(data.nextCursor);

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

export async function loadMoreFiles() {
  try {
    const cursor = (await new Promise(resolve => {
      nextCursor.subscribe(resolve)();
    })) as string | undefined;
    if (!cursor) return;

    isLoading.set(true);
    const data = await api.listFiles(100, cursor);
    files.update(existing => [...existing, ...(data.files || [])]);
    hasMore.set(!!data.nextCursor);
    nextCursor.set(data.nextCursor);
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
