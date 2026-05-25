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
  id: string;
  name: string;
  size_bytes: number;
  mime_type: string;
  uploaded_at: number;
  updated_at?: number;
  folder_path?: string;
  tags?: string[];
  pinned_at?: number;
  public: boolean;
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
  listFiles: (limit?: number) =>
    request<{ files: FileRecord[] }>(`/files${limit ? `?limit=${limit}` : ''}`),
  listFolders: () => request<{ folders: string[] }>('/folders'),
  getFile: (id: string) => request<FileRecord>(`/files/${id}`),
  deleteFile: (id: string) =>
    request(`/files/${encodeURIComponent(id)}`, { method: 'DELETE' }),
  renameFile: (id: string, newName: string) =>
    request(`/files/${encodeURIComponent(id)}`, {
      method: 'PATCH',
      body: JSON.stringify({ name: newName }),
    }),
  updateFileTags: (id: string, tags: string[]) =>
    request(`/files/${encodeURIComponent(id)}/tags`, {
      method: 'PUT',
      body: JSON.stringify({ tags }),
    }),
  toggleFilePin: (id: string) =>
    request(`/files/${encodeURIComponent(id)}/pin`, { method: 'POST' }),
  toggleFilePublic: (id: string) =>
    request(`/files/${encodeURIComponent(id)}/visibility`, { method: 'POST' }),

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
      status: string;
      file_count: number;
      storage_bytes_used: number;
      storage_total_bytes: number;
    }>('/status'),
};
