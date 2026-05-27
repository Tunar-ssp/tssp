/**
 * Drive State Service
 * Orchestrates file operations and maintains reactive state for DriveView
 * Bridges DriveView UI with underlying driveService API operations
 */

import { writable, derived, type Readable, type Writable } from 'svelte/store';
import { api, type FileRecord, type FolderEntry, type VisibilityResponse } from '$lib/api';
import * as DriveService from './driveService';

export interface DriveState {
  files: FileRecord[];
  trash: FileRecord[];
  folders: FolderEntry[];
  selectedFile: FileRecord | null;
  isLoading: boolean;
  error: string | null;
  currentFolder: string;
}

class DriveStateManager {
  private filesStore: Writable<FileRecord[]>;
  private trashStore: Writable<FileRecord[]>;
  private foldersStore: Writable<FolderEntry[]>;
  private selectedFileStore: Writable<FileRecord | null>;
  private isLoadingStore: Writable<boolean>;
  private errorStore: Writable<string | null>;
  private currentFolderStore: Writable<string>;

  public files: Readable<FileRecord[]>;
  public trash: Readable<FileRecord[]>;
  public folders: Readable<FolderEntry[]>;
  public selectedFile: Readable<FileRecord | null>;
  public isLoading: Readable<boolean>;
  public error: Readable<string | null>;
  public currentFolder: Readable<string>;

  constructor() {
    this.filesStore = writable<FileRecord[]>([]);
    this.trashStore = writable<FileRecord[]>([]);
    this.foldersStore = writable<FolderEntry[]>([]);
    this.selectedFileStore = writable<FileRecord | null>(null);
    this.isLoadingStore = writable<boolean>(false);
    this.errorStore = writable<string | null>(null);
    this.currentFolderStore = writable<string>('');

    this.files = { subscribe: this.filesStore.subscribe };
    this.trash = { subscribe: this.trashStore.subscribe };
    this.folders = { subscribe: this.foldersStore.subscribe };
    this.selectedFile = { subscribe: this.selectedFileStore.subscribe };
    this.isLoading = { subscribe: this.isLoadingStore.subscribe };
    this.error = { subscribe: this.errorStore.subscribe };
    this.currentFolder = { subscribe: this.currentFolderStore.subscribe };
  }

  async loadLibrary(reset = false): Promise<void> {
    this.isLoadingStore.set(true);
    this.errorStore.set(null);

    try {
      const [files, folders] = await Promise.all([
        DriveService.loadFiles(),
        DriveService.loadFolders(),
      ]);

      if (reset) {
        this.filesStore.set(files);
      }
      this.foldersStore.set(folders.map((path) => ({
        path,
        label: path.split('/').pop() || 'root',
        depth: path.split('/').length,
        file_count: files.filter((f) => f.folder_path === path).length,
      })));
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to load files');
    } finally {
      this.isLoadingStore.set(false);
    }
  }

  async loadTrash(): Promise<void> {
    this.isLoadingStore.set(true);
    this.errorStore.set(null);

    try {
      const trash = await DriveService.loadTrash();
      this.trashStore.set(trash);
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to load trash');
    } finally {
      this.isLoadingStore.set(false);
    }
  }

  async renameFile(file: FileRecord, newName: string): Promise<boolean> {
    if (!newName || newName === file.name) return false;

    try {
      const updated = await DriveService.renameFile(file.id, newName);
      this.updateFileInStore(updated);
      return true;
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to rename file');
      return false;
    }
  }

  async deleteFile(file: FileRecord): Promise<boolean> {
    try {
      await DriveService.deleteFile(file.id, file.name);
      this.filesStore.update((files) => files.filter((f) => f.id !== file.id));
      return true;
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to delete file');
      return false;
    }
  }

  async restoreFile(file: FileRecord): Promise<boolean> {
    try {
      await DriveService.restoreFile(file.id);
      this.trashStore.update((trash) => trash.filter((f) => f.id !== file.id));
      await this.loadLibrary(false);
      return true;
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to restore file');
      return false;
    }
  }

  async permanentlyDeleteFile(file: FileRecord): Promise<boolean> {
    try {
      await DriveService.deleteFilePermanently(file.id);
      this.trashStore.update((trash) => trash.filter((f) => f.id !== file.id));
      return true;
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to permanently delete');
      return false;
    }
  }

  async togglePin(file: FileRecord): Promise<boolean> {
    try {
      await DriveService.togglePin(file.id, !!file.pinned_at);
      const updated = { ...file, pinned_at: file.pinned_at ? undefined : Date.now() };
      this.updateFileInStore(updated);
      return true;
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to toggle pin');
      return false;
    }
  }

  async moveFile(file: FileRecord, folderPath: string): Promise<boolean> {
    try {
      await DriveService.moveFile(file.id, file.folder_path || '', folderPath);
      await this.loadLibrary(false);
      return true;
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to move file');
      return false;
    }
  }

  async setFileVisibility(fileId: string, isPublic: boolean): Promise<FileRecord | null> {
    try {
      const result = await DriveService.toggleFileVisibility(fileId, isPublic);
      if (result) {
        this.updateFileInStore(result);
      }
      return result;
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to change visibility');
      return null;
    }
  }

  async emptyTrash(): Promise<boolean> {
    try {
      await DriveService.emptyTrash();
      this.trashStore.set([]);
      return true;
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to empty trash');
      return false;
    }
  }

  selectFile(file: FileRecord | null): void {
    this.selectedFileStore.set(file);
  }

  setCurrentFolder(path: string): void {
    this.currentFolderStore.set(path);
  }

  downloadFile(file: FileRecord): void {
    DriveService.downloadFile(file.id, file.name);
  }

  async copyFiles(fileIds: string[], targetFolder: string): Promise<boolean> {
    try {
      await DriveService.copyFiles(fileIds, targetFolder);
      await this.loadLibrary(false);
      return true;
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to copy files');
      return false;
    }
  }

  async moveFiles(fileIds: string[], fromFolder: string, targetFolder: string): Promise<boolean> {
    try {
      await DriveService.moveFiles(fileIds, fromFolder, targetFolder);
      await this.loadLibrary(false);
      return true;
    } catch (err) {
      this.errorStore.set(err instanceof Error ? err.message : 'Failed to move files');
      return false;
    }
  }

  private updateFileInStore(updated: FileRecord): void {
    this.filesStore.update((files) =>
      files.map((f) => (f.id === updated.id ? updated : f))
    );
  }

  clearError(): void {
    this.errorStore.set(null);
  }
}

export const driveStateManager = new DriveStateManager();
