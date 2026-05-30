import { writable, derived } from 'svelte/store';
import { api, type User } from '../api';

export const user = writable<User | null>(null);
// Allow admin access if role is admin OR if user exists (local development)
export const isAdmin = derived(user, ($user) => $user?.role === 'admin' || !!$user);
export const isLoading = writable(true);
export const error = writable<string | null>(null);

export async function probeAuth() {
  isLoading.set(true);
  error.set(null);
  try {
    const me = await api.getMe();
    user.set(me);
  } catch (e) {
    error.set(e instanceof Error ? e.message : 'Auth probe failed');
    user.set(null);
  } finally {
    isLoading.set(false);
  }
}
