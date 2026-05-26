import { request } from './shared';

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

export const adminApi = {
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
};
