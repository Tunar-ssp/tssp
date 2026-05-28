/**
 * Centralized keyboard shortcuts and event handling
 * Extracted from App.svelte to improve modularity and testability
 */

import { get } from 'svelte/store';
import {
  toggleCommandPalette,
  commandPaletteOpen,
  toggleSettingsTray,
  settingsTrayOpen,
  toggleShortcutsOverlay,
  shortcutsOverlayOpen,
} from '$lib/stores/ui';

/**
 * Handle global keyboard shortcuts
 */
export function handleGlobalKeydown(e: KeyboardEvent): void {
  // Cmd/Ctrl+K - Toggle command palette
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    toggleCommandPalette();
    return;
  }

  // Escape - Close open overlays in priority order
  if (e.key === 'Escape') {
    if (get(commandPaletteOpen)) {
      commandPaletteOpen.set(false);
      return;
    }
    if (get(settingsTrayOpen)) {
      settingsTrayOpen.set(false);
      return;
    }
    if (get(shortcutsOverlayOpen)) {
      shortcutsOverlayOpen.set(false);
    }
    return;
  }

  // Cmd/Ctrl+, - Toggle settings
  if ((e.ctrlKey || e.metaKey) && e.key === ',') {
    e.preventDefault();
    toggleSettingsTray();
    return;
  }

  // Cmd/Ctrl+? - Toggle shortcuts overlay
  if ((e.ctrlKey || e.metaKey) && e.key === '?') {
    e.preventDefault();
    toggleShortcutsOverlay();
    return;
  }
}

/**
 * Register global keyboard event listeners
 */
export function registerGlobalKeyboardHandlers(): () => void {
  const handleKeydown = (e: KeyboardEvent) => handleGlobalKeydown(e);
  document.addEventListener('keydown', handleKeydown);

  return () => {
    document.removeEventListener('keydown', handleKeydown);
  };
}
