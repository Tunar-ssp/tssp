import { request } from './shared';

export interface Workspace {
  id: string;
  name: string;
  language: string;
  body: string;
  created_at: number;
  updated_at: number;
}

export interface WorkspaceDocumentSummary {
  id: string;
  path: string;
}

export interface WorkspaceCapabilities {
  schema_version: number;
  terminal: {
    status: 'available' | 'disabled' | 'forbidden' | 'unavailable_sandbox' | 'unavailable';
    message?: string;
  };
  lsp: {
    status: 'available' | 'disabled' | 'unavailable' | 'not_implemented';
    available_languages?: string[];
    message?: string;
  };
}

export const workspaceApi = {
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
  getWorkspaceCapabilities: (id: string) =>
    request<WorkspaceCapabilities>(`/workspaces/${encodeURIComponent(id)}/capabilities`),
  getWorkspaceTerminalStatus: (id: string) =>
    request<{ status: 'available' | 'disabled' | 'forbidden' | 'unavailable_sandbox' | 'unavailable' }>(`/workspaces/${encodeURIComponent(id)}/terminal`),
  getWorkspaceLspStatus: (id: string) =>
    request<{ status: 'available' | 'disabled' | 'unavailable' | 'not_implemented'; languages?: string[] }>(`/workspaces/${encodeURIComponent(id)}/lsp`),
};
