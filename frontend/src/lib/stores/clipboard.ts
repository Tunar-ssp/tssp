/**
 * Clipboard store for file operations
 * Manages copy/cut/paste operations across the application
 */

import { writable } from 'svelte/store';

export interface ClipboardItem {
  id: string;
  name: string;
  type: 'file' | 'folder';
  operation: 'copy' | 'cut';
}

export interface ClipboardState {
  items: ClipboardItem[];
  operation: 'copy' | 'cut' | null;
}

function createClipboardStore() {
  const { subscribe, set, update } = writable<ClipboardState>({
    items: [],
    operation: null,
  });

  return {
    subscribe,

    copy(items: Omit<ClipboardItem, 'operation'>[]): void {
      update(state => ({
        items: items.map(item => ({ ...item, operation: 'copy' })),
        operation: 'copy',
      }));
    },

    cut(items: Omit<ClipboardItem, 'operation'>[]): void {
      update(state => ({
        items: items.map(item => ({ ...item, operation: 'cut' })),
        operation: 'cut',
      }));
    },

    paste(): ClipboardItem[] {
      let items: ClipboardItem[] = [];
      update(state => {
        items = state.items;
        return { items: [], operation: null };
      });
      return items;
    },

    clear(): void {
      set({ items: [], operation: null });
    },

    hasItems(): boolean {
      let has = false;
      subscribe(state => {
        has = state.items.length > 0;
      })();
      return has;
    },

    getItemIds(): string[] {
      let ids: string[] = [];
      subscribe(state => {
        ids = state.items.map(item => item.id);
      })();
      return ids;
    },

    getState() {
      let state: ClipboardState | null = null;
      subscribe(s => {
        state = s;
      })();
      return state;
    },
  };
}

export const clipboard = createClipboardStore();
