export interface ApiResponse<T> {
  data?: T;
  error?: string;
}

export const BASE = '/api/v1';

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    let err: any = {};
    try {
      err = text ? JSON.parse(text) : {};
    } catch {
      err = { error: text || `HTTP ${res.status}` };
    }
    throw new Error(
      err?.error?.message || err?.error || `HTTP ${res.status}`
    );
  }

  if (res.status === 204) {
    return {} as T;
  }

  const text = await res.text();
  if (!text) return {} as T;
  
  try {
    return JSON.parse(text);
  } catch (err) {
    console.error('Failed to parse JSON response:', text);
    return {} as T;
  }
}

export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(BASE + path, {
    credentials: 'same-origin',
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
    ...init,
  });

  return handleResponse<T>(res);
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

  return handleResponse<T>(res);
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

  return handleResponse<T>(res);
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

  return handleResponse<T>(res);
}

export async function rawRequest(path: string, init?: RequestInit): Promise<Response> {
  return fetch(path, {
    credentials: 'same-origin',
    ...init,
  });
}
