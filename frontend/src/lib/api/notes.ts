import { request } from './shared';

export interface Note {
  id: string;
  title: string;
  body: string;
  tags: string[];
  pinned_at?: number;
  created_at: number;
  updated_at: number;
}

export const notesApi = {
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
};
