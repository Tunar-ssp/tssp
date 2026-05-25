import { writable } from 'svelte/store';

export const currentView = writable<string>('home');
export const banner = writable<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);
export const commandPaletteOpen = writable(false);
export const settingsTrayOpen = writable(false);
export const shortcutsOverlayOpen = writable(false);

export function showBanner(message: string, type: 'success' | 'error' | 'info' = 'info') {
  banner.set({ message, type });
  if (type !== 'error') {
    setTimeout(() => banner.set(null), 3000);
  }
}

export function toggleCommandPalette() {
  commandPaletteOpen.update((v) => !v);
}

export function toggleSettingsTray() {
  settingsTrayOpen.update((v) => !v);
}

export function toggleShortcutsOverlay() {
  shortcutsOverlayOpen.update((v) => !v);
}
