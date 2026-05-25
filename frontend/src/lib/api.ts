export interface ApiResponse<T> {
  data?: T;
  error?: string;
}

export interface User {
  id: string;
  name: string;
  role: 'admin' | 'user';
}

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

export interface Note {
  id: string;
  title: string;
  body: string;
  tags: string[];
  pinned_at?: number;
  created_at: number;
  updated_at: number;
}

export interface Workspace {
  id: string;
  name: string;
  language: string;
  body: string;
  created_at: number;
  updated_at: number;
}

const BASE = '/api/v1';

async function authRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch('/api/auth' + path, {
    credentials: 'same-origin',
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
    ...init,
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(
      err?.error?.message || err?.error || `HTTP ${res.status}`
    );
  }

  return res.json();
}

async function devicesRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch('/api/devices' + path, {
    credentials: 'same-origin',
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
    ...init,
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(
      err?.error?.message || err?.error || `HTTP ${res.status}`
    );
  }

  return res.json();
}

async function shareRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch('/api/share' + path, {
    credentials: 'same-origin',
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
    ...init,
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(
      err?.error?.message || err?.error || `HTTP ${res.status}`
    );
  }

  return res.json();
}

async function rawRequest(path: string, init?: RequestInit): Promise<Response> {
  return fetch(path, {
    credentials: 'same-origin',
    ...init,
  });
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

export interface SearchResult {
  type: 'file' | 'note' | 'workspace';
  id: string;
  name?: string;
  title?: string;
  snippet?: string;
  tags?: string[];
  visibility?: string;
  folder_path?: string;
  updated_at?: number;
}

export interface AdminUser {
  id: string;
  name: string;
  role: 'admin' | 'user';
  created_at: number;
  disabled: boolean;
}

export interface AdminSession {
  token: string;
  token_preview: string;
  kind: string;
  user_id?: string;
  user_name?: string;
  role?: string;
  created_at: number;
  expires_at: number;
  current: boolean;
}

export interface AdminActivityItem {
  kind: string;
  id: string;
  title: string;
  detail: string;
  occurred_at: number;
  visibility?: string;
  size_bytes?: number;
  language?: string;
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

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(BASE + path, {
    credentials: 'same-origin',
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
    ...init,
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(
      err?.error?.message || err?.error || `HTTP ${res.status}`
    );
  }

  return res.json();
}

export const api = {
  // Auth
  getMe: () => request<User>('/auth/me'),
  
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
    request(`/files/${encodeURIComponent(id)}/permanent`, { method: 'DELETE' }),
  listTrash: async () => {
    const data = await request<{ files: FileRecord[] }>('/trash');
    return { ...data, files: (data.files || []).map(normalizeFileRecord) };
  },
  emptyTrash: () =>
    request('/trash/empty', { method: 'POST' }),

  // Notes
  listNotes: async (limit?: number, cursor?: string) => {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());
    if (cursor) params.append('page', cursor);
    const query = params.toString();
    const data = await request<{ notes: Note[]; next_cursor?: string }>(
      `/notes${query ? `?${query}` : ''}`,
    );
    return {
      ...data,
      nextCursor: data.next_cursor,
    };
  },
  getNote: (id: string) => request<Note>(`/notes/${encodeURIComponent(id)}`),
  createNote: (note: Partial<Note>) =>
    request<Note>('/notes', {
      method: 'POST',
      body: JSON.stringify(note),
    }),
  updateNote: (id: string, updates: Partial<Note>) =>
    request<Note>(`/notes/${encodeURIComponent(id)}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    }),
  deleteNote: (id: string) =>
    request(`/notes/${encodeURIComponent(id)}`, { method: 'DELETE' }),
  duplicateNote: (id: string) =>
    request<Note>(`/notes/${encodeURIComponent(id)}/duplicate`, { method: 'POST' }),
  replaceNoteTags: (id: string, tags: string[]) =>
    request(`/notes/${encodeURIComponent(id)}/tags`, {
      method: 'PUT',
      body: JSON.stringify(tags),
    }),
  pinNote: (id: string) =>
    request(`/notes/${encodeURIComponent(id)}/pin`, { method: 'PUT' }),
  unpinNote: (id: string) =>
    request(`/notes/${encodeURIComponent(id)}/pin`, { method: 'DELETE' }),

  // Workspaces
  listWorkspaces: async (limit?: number, cursor?: string) => {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());
    if (cursor) params.append('page', cursor);
    const query = params.toString();
    const data = await request<{ workspaces: Workspace[]; next_cursor?: string }>(
      `/workspaces${query ? `?${query}` : ''}`,
    );
    return {
      ...data,
      nextCursor: data.next_cursor,
    };
  },
  getWorkspace: (id: string) =>
    request<Workspace>(`/workspaces/${encodeURIComponent(id)}`),
  createWorkspace: (ws: Partial<Workspace>) =>
    request<Workspace>('/workspaces', {
      method: 'POST',
      body: JSON.stringify(ws),
    }),
  updateWorkspace: (id: string, updates: Partial<Workspace>) =>
    request<Workspace>(`/workspaces/${encodeURIComponent(id)}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    }),
  deleteWorkspace: (id: string) =>
    request(`/workspaces/${encodeURIComponent(id)}`, { method: 'DELETE' }),

  // Status
  getStatus: () =>
    request<{
      schema_version: number;
      version: string;
      status: string;
      uptime_seconds: number;
      file_count: number;
      note_count: number;
      tag_count: number;
      pinned_count: number;
      recent_upload_count_24h: number;
      storage_bytes_used: number;
      corrupt_file_count: number;
      public_url?: string;
    }>('/status'),
  search: (query: string, limit = 25) =>
    request<{ schema_version: number; results: SearchResult[] }>(
      `/search?q=${encodeURIComponent(query)}&limit=${limit}`,
    ),

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

  // Auth
  login: (credentials: { name?: string; code?: string; password?: string }) =>
    request<{ schema_version: number; token: string; name: string; role: string }>(
      '/auth/login',
      {
        method: 'POST',
        body: JSON.stringify(credentials),
      }
    ),
  logout: () =>
    request<void>('/auth/logout', { method: 'POST' }),

  // Devices
  listDevices: () =>
    devicesRequest<{ devices: Array<{ id: string; name: string; trusted_at?: number }> }>(''),
  removeDevice: (deviceId: string) =>
    devicesRequest<void>(`/${encodeURIComponent(deviceId)}`, { method: 'DELETE' }),

  // Admin
  getAdminOverview: () =>
    request<{
      schema_version: number;
      repository: {
        file_count: number;
        storage_bytes_used: number;
        note_count: number;
      };
      system: {
        uptime_seconds: number;
        cpu_percent?: number;
        memory_percent?: number;
        disk_percent?: number;
      };
    }>('/admin/overview'),
  getAdminStatus: () =>
    request<{
      schema_version: number;
      status: string;
      version: string;
      uptime_seconds: number;
      uptime_hours: number;
      last_restart: string;
      disk_used: number;
      disk_total: number;
      memory_used: number;
      memory_total: number;
      cpu_percent: number;
      load_average: number;
      total_files: number;
      total_size: number;
      db_size: number;
      db_status: string;
    }>('/admin/status'),
  listAdminConsoleCommands: () =>
    request<{ schema_version: number; commands: Array<{ id: string; name: string; description?: string }> }>(
      '/admin/console/commands'
    ),
  runAdminConsoleCommand: (command: string) =>
    request<{ schema_version: number; output: string; success: boolean }>('/admin/console/run', {
      method: 'POST',
      body: JSON.stringify({ command }),
    }),
  listAdminDevices: () =>
    request<{ schema_version: number; devices: Array<{ id: string; token: string; trusted_at?: number }> }>(
      '/admin/devices'
    ),
  removeAdminDevice: (token: string) =>
    request<void>(`/admin/devices/${encodeURIComponent(token)}`, { method: 'DELETE' }),
  listAdminUsers: () =>
    request<{ schema_version: number; users: AdminUser[] }>(
      '/admin/users'
    ),
  listAdminSessions: (limit?: number) =>
    request<{ schema_version: number; sessions: AdminSession[] }>(
      `/admin/sessions${limit ? `?limit=${limit}` : ''}`
    ),
  listAdminActivity: (limit?: number) =>
    request<{ schema_version: number; items: AdminActivityItem[] }>(
      `/admin/activity${limit ? `?limit=${limit}` : ''}`
    ),

  // Chunked upload
  startUpload: (folder?: string) =>
    request<{ session_id: string; chunk_size: number }>('/files/upload/start', {
      method: 'POST',
      body: JSON.stringify({ folder: folder || '' }),
    }),
  uploadChunk: (sessionId: string, index: number, chunk: Blob) => {
    const formData = new FormData();
    formData.append('chunk', chunk);
    return request<{ session_id: string; chunk_index: number }>(
      `/files/upload/${encodeURIComponent(sessionId)}/chunk/${index}`,
      {
        method: 'PUT',
        body: formData,
        headers: {}, // Remove Content-Type to let browser set it with boundary
      }
    );
  },
  completeUpload: (sessionId: string, files: Array<{ name: string; mime_type: string }>) =>
    request<{ schema_version: number; files: FileRecord[] }>(
      `/files/upload/${encodeURIComponent(sessionId)}`,
      {
        method: 'POST',
        body: JSON.stringify({ files }),
      }
    ),

  // Share
  getSharedFile: (shareId: string) =>
    shareRequest<{ schema_version: number; file: FileRecord }>(`/${encodeURIComponent(shareId)}`),
  downloadSharedFile: async (shareId: string) => {
    const res = await rawRequest(`/p/${encodeURIComponent(shareId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.blob();
  },
};

export const listPublicFiles = api.listPublicFiles;
export const runSearch = api.search;
