import { writable } from 'svelte/store';

export const currentView = writable<string>('drive');
export const banner = writable<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);
export const commandPaletteOpen = writable(false);

export function showBanner(message: string, type: 'success' | 'error' | 'info' = 'info') {
  banner.set({ message, type });
  if (type !== 'error') {
    setTimeout(() => banner.set(null), 3000);
  }
}
