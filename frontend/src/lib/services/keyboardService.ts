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
  const top = activeOverlays.peek();

  // Cmd/Ctrl+K - Toggle command palette.
  // The palette has its own capture-phase handler that owns this key while it is
  // the top-most overlay (it stops propagation), so reaching here means the
  // palette is not on top. Opening it should not leave a conflicting transient
  // overlay (settings/shortcuts) stuck behind it.
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    if (top === 'settings-tray') closeSettingsTray();
    if (top === 'shortcuts') closeShortcutsOverlay();
    toggleCommandPalette();
    return;
  }

  // Escape - close the top-most shell-owned overlay only.
  // We peek instead of pop: component-owned overlays (modal/preview/context-menu,
  // and the command palette) handle their own Escape and manage the stack
  // themselves, so destructively popping here would corrupt nested overlay state.
  if (e.key === 'Escape') {
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
    // 'modal' / 'preview' / 'context-menu' are owned by their components.
    return;
  }

  // The remaining shell shortcuts must not fire while an overlay owns the screen.
  if (top) return;

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
