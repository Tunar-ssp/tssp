/**
 * Keyboard Priority System
 * 
 * Enforces keyboard event hierarchy:
 * 1. Focused text-input/editor owns plain keys
 * 2. Top overlay (activeOverlays.peek()) owns Esc/Enter/arrows
 * 3. Global shortcuts (Ctrl+K palette, Ctrl+, settings) only when no overlay is top
 * 
 * One capture-phase dispatcher to prevent conflicts.
 */

import { activeOverlays } from '$lib/stores/ui';
import { get } from 'svelte/store';

export interface KeyboardPriority {
  level: 'input' | 'overlay' | 'global';
  description: string;
}

/**
 * Keyboard priority table (for reference)
 */
export const KEYBOARD_PRIORITY_TABLE = `
Priority Order:
1. FOCUSED INPUT/EDITOR
   - Text input (<input>, <textarea>)
   - Content-editable elements
   - Monaco/Tiptap editors
   - These consume all keyboard events (plain keys, navigation, etc.)

2. TOP OVERLAY
   - When overlay is displayed and active
   - Esc: close overlay (or exit to previous)
   - Enter: confirm action
   - Arrow keys: navigate options
   - Tab: focus management
   - Shift+Tab: reverse focus

3. GLOBAL SHORTCUTS
   - Only when NO overlay is top
   - Ctrl+K: open command palette
   - Ctrl+,: open settings
   - Ctrl+?: show shortcuts
   - Alt+1-9: app switching
   - Other documented shortcuts

Global overlay stack prevents key leak:
- Only peek() overlay gets events, not lower layers
- Route changes close transient overlays
- Modal focus trap on Esc (don't leak to background)
`;

/**
 * Determine current keyboard priority level
 */
export function getKeyboardPriority(): KeyboardPriority {
  // Check if focused element is an input
  const focused = document.activeElement;
  const isInput = 
    focused instanceof HTMLInputElement ||
    focused instanceof HTMLTextAreaElement ||
    (focused instanceof HTMLElement && focused.contentEditable === 'true') ||
    focused?.className.includes('monaco-editor') ||
    focused?.className.includes('ProseMirror');

  if (isInput) {
    return {
      level: 'input',
      description: 'Text input or editor has focus; consuming all keys',
    };
  }

  // Check if there's an active overlay
  const overlays = get(activeOverlays);
  if (overlays.length > 0) {
    return {
      level: 'overlay',
      description: `Top overlay "${overlays[overlays.length - 1]}" is active; owns Esc/Enter/arrows`,
    };
  }

  return {
    level: 'global',
    description: 'No input or overlay; global shortcuts available',
  };
}

/**
 * Check if a key event should be handled at global level
 * Returns false if event should be consumed by input/overlay
 */
export function shouldHandleGlobally(event: KeyboardEvent): boolean {
  const priority = getKeyboardPriority();

  if (priority.level === 'input') {
    // Let input handle plain keys, but allow Escape to bubble
    if (event.key === 'Escape') return true;
    return false;
  }

  if (priority.level === 'overlay') {
    // Only allow overlay-specific keys (overlay handles them, not global)
    return false;
  }

  // At global level, all keys can be handled
  return true;
}

/**
 * Setup global keyboard dispatcher
 * Calls capture-phase listener on document to intercept before normal handlers
 * Returns cleanup function
 */
export function setupGlobalKeyboardDispatcher(
  onGlobalKeyDown: (event: KeyboardEvent) => void
) {
  const handleKeyDown = (event: KeyboardEvent) => {
    if (shouldHandleGlobally(event)) {
      onGlobalKeyDown(event);
    }
  };

  // Use capture phase to intercept before other handlers
  document.addEventListener('keydown', handleKeyDown, true);

  return () => {
    document.removeEventListener('keydown', handleKeyDown, true);
  };
}
