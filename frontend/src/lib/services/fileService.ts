import { api } from '../api';
import { success, error } from '../stores/notifications';
import { loadFiles } from '../stores/drive';
import { uploadQueue } from '../stores/uploadQueue';

export async function deleteFile(id: string, name: string) {
  try {
    await api.deleteFile(id);
    await loadFiles();
    success('Permanently Deleted', `"${name}" has been permanently deleted and cannot be recovered`);
    return true;
  } catch (err: any) {
    error('Delete Failed', err.message || 'Could not delete file');
    return false;
  }
}

export async function renameFile(id: string, newName: string) {
  try {
    await api.renameFile(id, newName);
    await loadFiles();
    success('Renamed', `File renamed to "${newName}"`);
    return true;
  } catch (err: any) {
    error('Rename Failed', err.message || 'Could not rename file');
    return false;
  }
}

export async function updateFileTags(id: string, tags: string[]) {
  try {
    await api.updateFileTags(id, tags);
    await loadFiles();
    success('Tagged', `Applied ${tags.length} tag(s)`);
    return true;
  } catch (err: any) {
    error('Tag Failed', err.message || 'Could not update tags');
    return false;
  }
}

export async function togglePin(id: string, currentlyPinned: boolean) {
  try {
    if (currentlyPinned) {
      await api.unpinFile(id);
      success('Unpinned', 'File unpinned from top');
    } else {
      await api.pinFile(id);
      success('Pinned', 'File pinned to top');
    }
    await loadFiles();
    return true;
  } catch (err: any) {
    error('Pin Failed', err.message || 'Could not change pin state');
    return false;
  }
}

export async function togglePublic(id: string, isPublic: boolean) {
  try {
    const result = await api.setFileVisibility(id, isPublic);
    await loadFiles();
    success('Visibility Updated', isPublic ? 'File shared' : 'File made private');
    return result;
  } catch (err: any) {
    error('Update Failed', err.message || 'Could not update visibility');
    return null;
  }
}

export async function uploadFiles(files: FileList, folder: string = '') {
  try {
    const count = files.length;

    // Queue files for chunked, resumable upload with persistence
    await uploadQueue.addFiles(files, folder);

    success('Uploads Queued', `${count} file(s) queued for resumable upload`);

    // Files will be uploaded in background with progress tracking
    // Queue persists across page refreshes for recovery
    return true;
  } catch (err: any) {
    error('Upload Failed', err.message || 'Could not queue files for upload');
    return false;
  }
}

export async function downloadFile(id: string, name: string) {
  try {
    const res = await fetch(`/api/v1/files/${id}/content`, {
      credentials: 'same-origin',
    });

    if (!res.ok) throw new Error('Download failed');

    const blob = await res.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = name;
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
    document.body.removeChild(a);

    success('Downloaded', `"${name}" downloaded`);
    return true;
  } catch (err: any) {
    error('Download Failed', err.message || 'Could not download file');
    return false;
  }
}
