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
  listFiles: async (limit?: number) => {
    const data = await request<{ files: FileRecord[] }>(
      `/files${limit ? `?limit=${limit}` : ''}`,
    );
    return { ...data, files: (data.files || []).map(normalizeFileRecord) };
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
  getFileShare: (id: string) =>
    request<FileShareResponse>(`/files/${encodeURIComponent(id)}/share`),
  listPublicFiles: async () => {
    const data = await request<{ schema_version: number; files: FileRecord[] }>('/public/files');
    return { ...data, files: (data.files || []).map(normalizeFileRecord) };
  },

  // Notes
  listNotes: () => request<{ notes: Note[] }>('/notes'),
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
  listWorkspaces: () => request<{ workspaces: Workspace[] }>('/workspaces'),
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
};

export const listPublicFiles = api.listPublicFiles;
export const runSearch = api.search;
