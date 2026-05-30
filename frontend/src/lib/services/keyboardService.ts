/**
 * Centralized keyboard shortcuts and event handling
 * Extracted from App.svelte to improve modularity and testability
 */

import { get } from 'svelte/store';
import {
  toggleCommandPalette,
  closeCommandPalette,
  toggleSettingsTray,
  closeSettingsTray,
  toggleShortcutsOverlay,
  closeShortcutsOverlay,
  activeOverlays,
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
    const top = activeOverlays.pop();
    if (top === 'command-palette') {
      closeCommandPalette();
      return;
    }
    if (top === 'settings-tray') {
      closeSettingsTray();
      return;
    }
    if (top === 'shortcuts') {
      closeShortcutsOverlay();
      return;
    }
    // Modals and other overlays should register themselves with activeOverlays
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
