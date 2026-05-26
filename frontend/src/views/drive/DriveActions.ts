/**
 * Drive Actions - Encapsulates all file operations and business logic
 * Extracted from monolithic DriveView to improve testability and reusability
 */

import { api, type FileRecord } from '$lib/api';

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
      await api.permanentlyDeleteFile(file.id);
      this.notifySuccess(`${file.name} permanently deleted`);
      return true;
    } catch (err) {
      this.notifyError(`Failed to permanently delete file`);
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

  formatBytes(bytes: number): string {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    const value = bytes / 1024 ** index;
    return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
  }

  formatRelative(timestamp?: number): string {
    if (!timestamp) return 'just now';
    const delta = Math.max(0, Math.floor(Date.now() / 1000) - timestamp);
    if (delta < 60) return 'just now';
    if (delta < 3600) return `${Math.floor(delta / 60)}m`;
    if (delta < 86400) return `${Math.floor(delta / 3600)}h`;
    if (delta < 604800) return `${Math.floor(delta / 86400)}d`;
    return `${Math.floor(delta / 604800)}w`;
  }
}
