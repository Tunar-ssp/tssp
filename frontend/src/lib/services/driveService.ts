import { api, type FileRecord } from '$lib/api';
import { error } from '$lib/stores/notifications';

export interface DriveState {
  files: FileRecord[];
  trash: FileRecord[];
  folders: string[];
  selectedFile: FileRecord | null;
  isLoading: boolean;
  trashLoading: boolean;
  currentTab: 'files' | 'trash';
  viewMode: 'list' | 'grid';
  filterQuery: string;
  searchResults: FileRecord[];
  isSearching: boolean;
}

export async function loadFiles(): Promise<FileRecord[]> {
  try {
    const data = await api.listFiles();
    return data.files || [];
  } catch (err) {
    error(`Failed to load files: ${err instanceof Error ? err.message : 'Unknown error'}`);
    return [];
  }
}

export async function loadFolders(): Promise<string[]> {
  try {
    const data = await api.listFolders();
    return data.folders?.map(f => f.path) || [];
  } catch (err) {
    error(`Failed to load folders: ${err instanceof Error ? err.message : 'Unknown error'}`);
    return [];
  }
}

export async function loadTrash(): Promise<FileRecord[]> {
  try {
    const data = await api.listTrash();
    return data.files || [];
  } catch (err) {
    error(`Failed to load trash: ${err instanceof Error ? err.message : 'Unknown error'}`);
    return [];
  }
}

export async function searchFiles(query: string): Promise<FileRecord[]> {
  try {
    const data = await api.search(query);
    return (data.results || []).filter((r: any) => r.type === 'file') as FileRecord[];
  } catch (err) {
    error(`Search failed: ${err instanceof Error ? err.message : 'Unknown error'}`);
    return [];
  }
}

export async function restoreFile(fileId: string): Promise<void> {
  try {
    await api.restoreFile(fileId);
  } catch (err) {
    error(`Failed to restore file: ${err instanceof Error ? err.message : 'Unknown error'}`);
    throw err;
  }
}

export async function deleteFilePermanently(fileId: string): Promise<void> {
  try {
    await api.permanentDeleteFile(fileId);
  } catch (err) {
    error(`Failed to delete file: ${err instanceof Error ? err.message : 'Unknown error'}`);
    throw err;
  }
}

export async function emptyTrash(): Promise<void> {
  try {
    await api.emptyTrash();
  } catch (err) {
    error(`Failed to empty trash: ${err instanceof Error ? err.message : 'Unknown error'}`);
    throw err;
  }
}

export async function toggleFileVisibility(fileId: string, isPublic: boolean): Promise<FileRecord | null> {
  try {
    const result = await api.setFileVisibility(fileId, isPublic);
    return result?.file || null;
  } catch (err) {
    error(`Failed to toggle visibility: ${err instanceof Error ? err.message : 'Unknown error'}`);
    throw err;
  }
}

export async function renameFile(fileId: string, newName: string): Promise<void> {
  try {
    await api.renameFile(fileId, newName);
  } catch (err) {
    error(`Failed to rename file: ${err instanceof Error ? err.message : 'Unknown error'}`);
    throw err;
  }
}

export async function pinFile(fileId: string): Promise<void> {
  try {
    await api.pinFile(fileId);
  } catch (err) {
    error(`Failed to pin file: ${err instanceof Error ? err.message : 'Unknown error'}`);
    throw err;
  }
}

export async function unpinFile(fileId: string): Promise<void> {
  try {
    await api.unpinFile(fileId);
  } catch (err) {
    error(`Failed to unpin file: ${err instanceof Error ? err.message : 'Unknown error'}`);
    throw err;
  }
}

export async function moveFile(fileId: string, folderPath: string): Promise<FileRecord | null> {
  try {
    const result = await api.moveFile(fileId, folderPath);
    return result?.file || null;
  } catch (err) {
    error(`Failed to move file: ${err instanceof Error ? err.message : 'Unknown error'}`);
    throw err;
  }
}

export async function downloadFile(fileId: string, fileName: string): Promise<void> {
  try {
    window.location.href = `/api/v1/files/${encodeURIComponent(fileId)}/download`;
  } catch (err) {
    error(`Failed to download file: ${err instanceof Error ? err.message : 'Unknown error'}`);
    throw err;
  }
}
