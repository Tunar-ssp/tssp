/**
 * Drive Actions - Encapsulates all file operations and business logic
 * Extracted from monolithic DriveView to improve testability and reusability
 */

import { api, type FileRecord, type VisibilityResponse } from '$lib/api';

export interface DriveActionHandlers {
  onSuccess?: (message: string) => void;
  onError?: (message: string) => void;
}

export class DriveActions {
  constructor(private handlers: DriveActionHandlers = {}) {}

  private notifySuccess(message: string) {
    this.handlers.onSuccess?.(message);
  }

  private notifyError(message: string) {
    this.handlers.onError?.(message);
  }

  async pinFile(file: FileRecord): Promise<boolean> {
    try {
      if (file.pinned_at) {
        await api.unpinFile(file.id);
        this.notifySuccess(`${file.name} unpinned`);
      } else {
        await api.pinFile(file.id);
        this.notifySuccess(`${file.name} pinned`);
      }
      return true;
    } catch (err) {
      this.notifyError(`Failed to pin/unpin ${file.name}`);
      return false;
    }
  }

  async deleteFile(file: FileRecord): Promise<boolean> {
    try {
      await api.deleteFile(file.id);
      this.notifySuccess(`${file.name} moved to trash`);
      return true;
    } catch (err) {
      this.notifyError(`Failed to delete ${file.name}`);
      return false;
    }
  }

  async renameFile(file: FileRecord, newName: string): Promise<boolean> {
    if (!newName || newName === file.name) return false;

    try {
      await api.renameFile(file.id, newName);
      this.notifySuccess(`Renamed to ${newName}`);
      return true;
    } catch (err) {
      this.notifyError(`Failed to rename file`);
      return false;
    }
  }

  async restoreFile(file: FileRecord): Promise<boolean> {
    try {
      await api.restoreFile(file.id);
      this.notifySuccess(`${file.name} restored`);
      return true;
    } catch (err) {
      this.notifyError(`Failed to restore file`);
      return false;
    }
  }

  async permanentlyDeleteFile(file: FileRecord): Promise<boolean> {
    try {
      await api.permanentDeleteFile(file.id);
      this.notifySuccess(`${file.name} permanently deleted`);
      return true;
    } catch (err) {
      this.notifyError(`Failed to permanently delete file`);
      return false;
    }
  }

  async moveFile(fileId: string, folderPath: string): Promise<boolean> {
    try {
      await api.moveFile(fileId, folderPath);
      this.notifySuccess('File moved successfully');
      return true;
    } catch (err) {
      this.notifyError('Failed to move file');
      return false;
    }
  }

  async setFileVisibility(fileId: string, isPublic: boolean): Promise<VisibilityResponse | null> {
    try {
      const response = await api.setFileVisibility(fileId, isPublic);
      this.notifySuccess(isPublic ? 'File made public' : 'File made private');
      return response;
    } catch (err) {
      this.notifyError('Failed to change visibility');
      return null;
    }
  }

  async emptyTrash(): Promise<boolean> {
    try {
      await api.emptyTrash();
      this.notifySuccess('Trash emptied');
      return true;
    } catch (err) {
      this.notifyError('Failed to empty trash');
      return false;
    }
  }

  downloadFile(file: FileRecord): void {
    try {
      window.location.assign(`/api/v1/files/${encodeURIComponent(file.id)}/content`);
    } catch (err) {
      this.notifyError(`Failed to download ${file.name}`);
    }
  }
}
