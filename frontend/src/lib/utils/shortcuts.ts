/**
 * Unified keyboard shortcuts handler
 * Eliminates redundant keyboard event handlers across components
 */

export type KeyboardHandler = (event: KeyboardEvent) => void;

export interface Shortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  meta?: boolean;
  handler: KeyboardHandler;
  description?: string;
}

/**
 * Create a keyboard event handler from a list of shortcuts
 */
export function createShortcutHandler(shortcuts: Shortcut[]): KeyboardHandler {
  return (event: KeyboardEvent) => {
    for (const shortcut of shortcuts) {
      const keyMatch = event.key.toLowerCase() === shortcut.key.toLowerCase();
      const ctrlMatch = (shortcut.ctrl ?? false) === (event.ctrlKey || event.metaKey);
      const shiftMatch = (shortcut.shift ?? false) === event.shiftKey;
      const altMatch = (shortcut.alt ?? false) === event.altKey;

      if (keyMatch && ctrlMatch && shiftMatch && altMatch) {
        event.preventDefault();
        shortcut.handler(event);
        return;
      }
    }
  };
}

/**
 * Register keyboard shortcuts that persist for the component lifecycle
 * Returns cleanup function to remove listeners
 */
export function registerKeyboardShortcuts(
  shortcuts: Shortcut[],
  target: EventTarget = document
): () => void {
  const handler = createShortcutHandler(shortcuts);
  const eventListener = handler as EventListener;
  target.addEventListener('keydown', eventListener);

  return () => {
    target.removeEventListener('keydown', eventListener);
  };
}

/**
 * Common keyboard shortcuts for file operations
 */
export const COMMON_SHORTCUTS = {
  selectAll: { key: 'a', ctrl: true, description: 'Select all files' },
  escape: { key: 'Escape', description: 'Deselect/Cancel' },
  delete: { key: 'Delete', description: 'Delete selected' },
  copy: { key: 'c', ctrl: true, description: 'Copy' },
  paste: { key: 'v', ctrl: true, description: 'Paste' },
  cut: { key: 'x', ctrl: true, description: 'Cut' },
  sidebarToggle: { key: 'b', ctrl: true, description: 'Toggle sidebar' },
  findToggle: { key: 'f', ctrl: true, description: 'Toggle find' },
  save: { key: 's', ctrl: true, description: 'Save' },
  undo: { key: 'z', ctrl: true, description: 'Undo' },
  redo: { key: 'z', ctrl: true, shift: true, description: 'Redo' },
} as const;
