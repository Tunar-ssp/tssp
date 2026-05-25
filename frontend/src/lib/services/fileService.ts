import { api } from '../api';
import { success, error } from '../stores/notifications';
import { loadFiles } from '../stores/drive';

export async function deleteFile(id: string, name: string) {
  try {
    await api.deleteFile(id);
    await loadFiles();
    success('Deleted', `"${name}" moved to trash`);
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
    await api.setFileVisibility(id, isPublic);
    await loadFiles();
    success('Visibility Updated', isPublic ? 'File shared' : 'File made private');
    return true;
  } catch (err: any) {
    error('Update Failed', err.message || 'Could not update visibility');
    return false;
  }
}

export async function uploadFiles(files: FileList, folder: string = '') {
  try {
    const formData = new FormData();
    let count = 0;

    for (const file of files) {
      formData.append('files', file);
      count++;
    }

    if (folder) {
      formData.append('folder', folder);
    }

    const res = await fetch('/api/v1/files/batch', {
      method: 'POST',
      body: formData,
      credentials: 'same-origin',
    });

    if (!res.ok) {
      throw new Error(`Upload failed: ${res.statusText}`);
    }

    await loadFiles();
    success('Upload Complete', `${count} file(s) uploaded successfully`);
    return true;
  } catch (err: any) {
    error('Upload Failed', err.message || 'Could not upload files');
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
