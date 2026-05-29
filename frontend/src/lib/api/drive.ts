import { request, rawRequest, shareRequest, BASE } from './shared';

export interface FileRecord {
  schema_version?: number;
  id: string;
  name: string;
  size_bytes: number;
  content_hash?: string;
  mime_type: string;
  uploaded_at: number;
  updated_at?: number;
  folder_path?: string;
  tags?: string[];
  pinned?: boolean;
  pinned_at?: number;
  public?: boolean;
  visibility?: 'public' | 'private' | string;
  public_token?: string;
}

export interface FolderEntry {
  path: string;
  file_count: number;
}

export interface VisibilityResponse {
  schema_version: number;
  file: FileRecord;
  public_url?: string;
}

export interface FileShareResponse {
  schema_version: number;
  public_url: string;
  qr_terminal: string;
}

function normalizeFileRecord(file: FileRecord): FileRecord {
  const isPublic = file.public ?? file.visibility === 'public';
  const isPinned = file.pinned ?? file.pinned_at !== undefined;

  return {
    ...file,
    public: isPublic,
    pinned: isPinned,
    pinned_at: file.pinned_at ?? (isPinned ? 1 : undefined),
    updated_at: file.updated_at ?? file.uploaded_at,
    tags: file.tags ?? [],
    folder_path: file.folder_path ?? '',
  };
}

export function fileContentUrl(id: string): string {
  return `/api/v1/files/${encodeURIComponent(id)}/content?disposition=inline`;
}

export function fileDownloadUrl(id: string): string {
  return `/api/v1/files/${encodeURIComponent(id)}/content`;
}

export const driveApi = {
  // Files
  listFiles: async (limit?: number, cursor?: string) => {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());
    if (cursor) params.append('page', cursor);
    const query = params.toString();
    const data = await request<{ files: FileRecord[]; next_cursor?: string }>(
      `/files${query ? `?${query}` : ''}`,
    );
    return {
      ...data,
      files: (data.files || []).map(normalizeFileRecord),
      nextCursor: data.next_cursor,
    };
  },
  listFolders: () => request<{ schema_version: number; folders: FolderEntry[] }>('/folders'),
  createFolder: (name: string, parentPath?: string) =>
    request<FolderEntry>('/folders', {
      method: 'POST',
      body: JSON.stringify({ path: parentPath ? `${parentPath}/${name}` : name }),
    }),
  moveFolder: (from: string, to: string) =>
    request<{ schema_version: number; files_updated: number }>('/folders/move', {
      method: 'POST',
      body: JSON.stringify({ from, to }),
    }),
  deleteFolder: (path: string) =>
    request<{ schema_version: number; files_updated: number }>('/folders/delete', {
      method: 'POST',
      body: JSON.stringify({ path }),
    }),
  getFile: async (id: string) => normalizeFileRecord(await request<FileRecord>(`/files/${id}`)),
  deleteFile: (id: string) =>
    request(`/files/${encodeURIComponent(id)}`, { method: 'DELETE' }),
  renameFile: (id: string, newName: string) =>
    request(`/files/${encodeURIComponent(id)}`, {
      method: 'PATCH',
      body: JSON.stringify({ name: newName }),
    }),
  updateFileTags: (id: string, tags: string[]) =>
    request(`/files/${encodeURIComponent(id)}/tags`, {
      method: 'POST',
      body: JSON.stringify(tags),
    }),
  pinFile: (id: string) =>
    request(`/files/${encodeURIComponent(id)}/pin`, { method: 'PUT' }),
  unpinFile: (id: string) =>
    request(`/files/${encodeURIComponent(id)}/pin`, { method: 'DELETE' }),
  setFileVisibility: (id: string, isPublic: boolean) =>
    request<VisibilityResponse>(`/files/${encodeURIComponent(id)}/visibility`, {
      method: 'PATCH',
      body: JSON.stringify({ visibility: isPublic ? 'public' : 'private' }),
    }).then((response) => ({
      ...response,
      file: normalizeFileRecord(response.file),
    })),
  bulkSetFileVisibility: (ids: string[], isPublic: boolean) =>
    request<{ schema_version: number; updated: FileRecord[]; count: number }>(`/files/visibility/bulk`, {
      method: 'POST',
      body: JSON.stringify({ ids, visibility: isPublic ? 'public' : 'private' }),
    }).then((response) => ({
      ...response,
      updated: (response.updated || []).map(normalizeFileRecord),
    })),
  moveFile: (id: string, folderPath: string) =>
    request<{ schema_version: number; file: FileRecord }>(`/files/${encodeURIComponent(id)}/folder`, {
      method: 'PATCH',
      body: JSON.stringify({ folder_path: folderPath }),
    }).then((response) => ({
      ...response,
      file: normalizeFileRecord(response.file),
    })),
  getFileShare: (id: string) =>
    request<FileShareResponse>(`/files/${encodeURIComponent(id)}/share`),
  listPublicFiles: async () => {
    const data = await request<{ schema_version: number; files: FileRecord[] }>('/public/files');
    return { ...data, files: (data.files || []).map(normalizeFileRecord) };
  },
  restoreFile: (id: string) =>
    request(`/files/${encodeURIComponent(id)}/restore`, { method: 'POST' }),
  permanentDeleteFile: (id: string) =>
    request(`/files/${encodeURIComponent(id)}/purge`, { method: 'DELETE' }),
  listTrash: async () => {
    const data = await request<{ files: FileRecord[] }>('/trash');
    return { ...data, files: (data.files || []).map(normalizeFileRecord) };
  },
  emptyTrash: () =>
    request('/trash/empty', { method: 'POST' }),

  // File download (returns blob)
  downloadFile: async (id: string, disposition?: 'inline') => {
    const query = disposition ? `?disposition=${disposition}` : '';
    const res = await rawRequest(`/api/v1/files/${encodeURIComponent(id)}/content${query}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.blob();
  },

  // Preview file content (returns text for text files, handles range requests)
  previewFile: async (id: string, rangeHeader?: string) => {
    const headers: Record<string, string> = {};
    if (rangeHeader) headers['Range'] = rangeHeader;
    const res = await rawRequest(`/api/v1/files/${encodeURIComponent(id)}/content?disposition=inline`, {
      headers,
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return {
      text: await res.text(),
      hasRange: res.headers.has('content-range'),
    };
  },

  // Chunked upload
  startUpload: (filename: string, totalSize: number, folder?: string) =>
    request<{ session_id: string; chunk_size: number }>('/files/upload/start', {
      method: 'POST',
      body: JSON.stringify({ filename, total_size: totalSize, folder_path: folder || '' }),
    }),
  uploadChunk: (sessionId: string, index: number, chunk: Blob) =>
    fetch(BASE + `/files/upload/${encodeURIComponent(sessionId)}/chunk/${index}`, {
      credentials: 'same-origin',
      method: 'POST',
      body: chunk,
      headers: {
        'Content-Type': 'application/octet-stream',
      },
    }).then(async (res) => {
      if (!res.ok) {
        const err = await res.json().catch(() => ({}));
        throw new Error(
          err?.error?.message || err?.error || `HTTP ${res.status}`
        );
      }
      return res.json() as Promise<{ session_id: string; chunk_index: number }>;
    }),
  completeUpload: (sessionId: string) =>
    request<{ session_id: string; status: string }>(
      `/files/upload/${encodeURIComponent(sessionId)}`,
      {
        method: 'POST',
      }
    ),
  cancelUpload: (sessionId: string) =>
    request<void>(`/files/upload/${encodeURIComponent(sessionId)}`, {
      method: 'DELETE',
    }),

  // Shared files
  getSharedFile: (shareId: string) =>
    shareRequest<{ schema_version: number; file: FileRecord }>(`/${encodeURIComponent(shareId)}`),
  downloadSharedFile: async (shareId: string) => {
    const res = await rawRequest(`/p/${encodeURIComponent(shareId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.blob();
  },
};
