import { writable, derived } from 'svelte/store';
import { api } from '../api';

export interface UserProfile {
  id: string;
  name: string;
  role: 'admin' | 'user';
  email?: string;
  avatar?: string;
  created_at?: string;
}

export const user = writable<UserProfile | null>(null);
export const isLoading = writable(false);

export async function loadUser() {
  isLoading.set(true);
  try {
    const u = await api.getMe();
    user.set(u as any);
  } catch (err) {
    console.error('Failed to load user:', err);
    user.set(null);
  } finally {
    isLoading.set(false);
  }
}

export function logout() {
  user.set(null);
  localStorage.removeItem('auth_token');
}
