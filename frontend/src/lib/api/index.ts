// Shared types and utilities
export type { ApiResponse } from './shared';
export { BASE, request, authRequest, devicesRequest, shareRequest, rawRequest } from './shared';

// Auth
export { authApi } from './auth';
export type { User, Device } from './auth';

// Drive
export { driveApi, fileContentUrl, fileDownloadUrl } from './drive';
export type { FileRecord, FolderEntry, VisibilityResponse, FileShareResponse } from './drive';

// Notes
export { notesApi } from './notes';
export type { Note } from './notes';

// Workspace
export { workspaceApi } from './workspace';
export type { Workspace, WorkspaceCapabilities, WorkspaceDocumentSummary } from './workspace';

// Admin
export { adminApi } from './admin';
export type { AdminUser, AdminSession, AdminActivityItem } from './admin';

// Search results
export interface SearchResult {
  url?: string;
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

// Import apis and helpers
import { request } from './shared';
import { authApi } from './auth';
import { driveApi } from './drive';
import { notesApi } from './notes';
import { workspaceApi } from './workspace';
import { adminApi } from './admin';

// Status and search endpoints
export const api = {
  // All API methods combined for backward compatibility
  ...authApi,
  ...driveApi,
  ...notesApi,
  ...workspaceApi,
  ...adminApi,

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

  // Search
  search: (query: string, limit?: number) =>
    request<{ schema_version: number; results: SearchResult[] }>(
      `/search?q=${encodeURIComponent(query)}&limit=${limit || 25}`,
    ),

  // Stub declarations (not yet implemented)
  searchNotes: (query: string): Promise<{ notes: any[] }> => Promise.reject('Not implemented'),
  searchWorkspaces: (query: string): Promise<{ workspaces: any[] }> => Promise.reject('Not implemented'),
  searchFiles: (query: string): Promise<{ files: any[] }> => Promise.reject('Not implemented'),
  getFileContent: (id: string): Promise<string> => Promise.reject('Not implemented'),
  getFileDownloadUrl: (id: string): Promise<string> => Promise.reject('Not implemented'),
  generateShareLink: (id: string): Promise<{ url: string }> => Promise.reject('Not implemented'),
  revokeShareLink: (id: string, linkId: string): Promise<void> => Promise.reject('Not implemented'),
  listShareLinks: (id: string): Promise<{ links: { id: string }[] }> => Promise.reject('Not implemented'),
};

// Backward compatibility exports
export const listPublicFiles = api.listPublicFiles;
export const runSearch = api.search;
