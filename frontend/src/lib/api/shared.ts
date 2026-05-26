export interface ApiResponse<T> {
  data?: T;
  error?: string;
}

export const BASE = '/api/v1';

export async function request<T>(path: string, init?: RequestInit): Promise<T> {
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

export async function authRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch('/api/v1/auth' + path, {
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

export async function devicesRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch('/api/v1/auth/devices' + path, {
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

export async function shareRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch('/api/v1/files' + path + '/share', {
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

export async function rawRequest(path: string, init?: RequestInit): Promise<Response> {
  return fetch(path, {
    credentials: 'same-origin',
    ...init,
  });
}
