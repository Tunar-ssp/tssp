import { authRequest, devicesRequest, request } from './shared';

export interface User {
  id: string;
  name: string;
  role: 'admin' | 'user';
}

export interface Device {
  id: string;
  name: string;
  token: string;
  fingerprint: string;
  created_at: number;
  last_used_at: number;
  is_current: boolean;
}

export const authApi = {
  getMe: () => request<User>('/auth/me'),

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
    devicesRequest<{ devices: Array<{ id: string; name: string; trusted_at?: number }>; }>(''),
  removeDevice: (deviceId: string) =>
    devicesRequest<void>(`/${encodeURIComponent(deviceId)}`, { method: 'DELETE' }),
};
