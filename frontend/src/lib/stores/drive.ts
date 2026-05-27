/**
 * Drive state management store
 * Extracted from DriveView to centralize file operations and state
 */

import { writable, derived } from 'svelte/store';
import { api, type FileRecord, type FolderEntry, type VisibilityResponse } from '$lib/api';

export type DriveLens = 'all' | 'images' | 'videos' | 'documents' | 'public' | 'trash';

interface DriveState {
  files: FileRecord[];
  trash: FileRecord[];
  folderEntries: FolderEntry[];
  selectedFileId: string | null;
  isLoading: boolean;
  isLoadingMore: boolean;
  trashLoading: boolean;
  nextCursor?: string;
  hasMore: boolean;
  currentFolder: string;
  filterQuery: string;
  activeLens: DriveLens;
  viewMode: 'grid' | 'list';
}

const initialState: DriveState = {
  files: [],
  trash: [],
  folderEntries: [],
  selectedFileId: null,
  isLoading: true,
  isLoadingMore: false,
  trashLoading: false,
  hasMore: false,
  currentFolder: '',
  filterQuery: '',
  activeLens: 'all',
  viewMode: 'grid',
};

export const driveState = writable<DriveState>(initialState);

/**
 * Get file by ID
 */
export function getFile(fileId: string): FileRecord | null {
  let file: FileRecord | null = null;
  driveState.subscribe(state => {
    file = state.files.find(f => f.id === fileId) ?? null;
  })();
  return file;
}

/**
 * Load files with pagination
 */
export async function loadFiles(reset = false): Promise<void> {
  driveState.update(state => ({ ...state, isLoading: reset }));
  try {
    const response = await api.listFiles(50, reset ? undefined : undefined);
    driveState.update(state => ({
      ...state,
      files: reset ? response.files : [...state.files, ...response.files],
      nextCursor: response.nextCursor,
      hasMore: !!response.nextCursor,
      isLoading: false,
      isLoadingMore: false,
    }));
  } catch (err) {
    driveState.update(state => ({ ...state, isLoading: false, isLoadingMore: false }));
    throw err;
  }
}

/**
 * Load trash
 */
export async function loadTrash(): Promise<void> {
  driveState.update(state => ({ ...state, trashLoading: true }));
  try {
    const response = await api.listTrash();
    driveState.update(state => ({
      ...state,
      trash: response.files,
      trashLoading: false,
    }));
  } catch (err) {
    driveState.update(state => ({ ...state, trashLoading: false }));
    throw err;
  }
}

/**
 * Update file in state
 */
export function updateFileInState(nextFile: FileRecord): void {
  driveState.update(state => ({
    ...state,
    files: state.files.map(f => (f.id === nextFile.id ? nextFile : f)),
  }));
}

/**
 * Delete file
 */
export async function deleteFile(fileId: string): Promise<void> {
  const file = getFile(fileId);
  if (!file) return;

  try {
    await api.deleteFile(fileId);
    driveState.update(state => ({
      ...state,
      files: state.files.filter(f => f.id !== fileId),
      selectedFileId: state.selectedFileId === fileId ? null : state.selectedFileId,
    }));
  } catch (err) {
    throw err;
  }
}

/**
 * Restore file from trash
 */
export async function restoreFile(fileId: string): Promise<void> {
  try {
    await api.restoreFile(fileId);
    driveState.update(state => ({
      ...state,
      trash: state.trash.filter(f => f.id !== fileId),
    }));
  } catch (err) {
    throw err;
  }
}

/**
 * Toggle file visibility
 */
export async function setFileVisibility(fileId: string, isPublic: boolean): Promise<VisibilityResponse | null> {
  try {
    const result = await api.setFileVisibility(fileId, isPublic);
    const file = getFile(fileId);
    if (file) {
      updateFileInState({ ...file, visibility: isPublic ? 'public' : 'private' });
    }
    return result;
  } catch (err) {
    throw err;
  }
}

/**
 * Move file
 */
export async function moveFile(fileId: string, folderPath: string): Promise<void> {
  try {
    await api.moveFile(fileId, folderPath);
    const file = getFile(fileId);
    if (file) {
      updateFileInState({ ...file, folder_path: folderPath });
    }
  } catch (err) {
    throw err;
  }
}

/**
 * Rename file
 */
export async function renameFile(fileId: string, newName: string): Promise<void> {
  try {
    await api.renameFile(fileId, newName);
    const file = getFile(fileId);
    if (file) {
      updateFileInState({ ...file, name: newName });
    }
  } catch (err) {
    throw err;
  }
}

/**
 * Set filter query
 */
export function setFilterQuery(query: string): void {
  driveState.update(state => ({ ...state, filterQuery: query }));
}

/**
 * Set active lens
 */
export function setActiveLens(lens: DriveLens): void {
  driveState.update(state => ({ ...state, activeLens: lens }));
}

/**
 * Set view mode
 */
export function setViewMode(mode: 'grid' | 'list'): void {
  driveState.update(state => ({ ...state, viewMode: mode }));
}

/**
 * Select file
 */
export function selectFile(fileId: string | null): void {
  driveState.update(state => ({ ...state, selectedFileId: fileId }));
}

/**
 * Derived: filtered files based on lens and query
 */
export const filteredFiles = derived(driveState, state => {
  return state.files.filter(file => {
    if (state.currentFolder && (file.folder_path || '') !== state.currentFolder) return false;
    if (state.activeLens === 'images' && !file.mime_type.startsWith('image/')) return false;
    if (state.activeLens === 'videos' && !file.mime_type.startsWith('video/')) return false;
    if (state.activeLens === 'documents' && !isDocument(file.mime_type)) return false;
    if (state.activeLens === 'public' && file.visibility !== 'public') return false;

    const query = state.filterQuery.trim().toLowerCase();
    if (!query) return true;
    return (
      file.name.toLowerCase().includes(query) ||
      (file.folder_path || '').toLowerCase().includes(query) ||
      (file.tags || []).some(tag => tag.toLowerCase().includes(query))
    );
  });
});

/**
 * Check if MIME type is a document
 */
function isDocument(mimeType: string): boolean {
  return (
    mimeType.startsWith('text/') ||
    mimeType.includes('pdf') ||
    mimeType.includes('document') ||
    mimeType.includes('word') ||
    mimeType.includes('sheet')
  ) && !mimeType.startsWith('image/') && !mimeType.startsWith('video/');
}
