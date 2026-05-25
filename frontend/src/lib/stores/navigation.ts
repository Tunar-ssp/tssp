/**
 * Navigation Store
 *
 * Manages current view and selection intent for cross-view navigation.
 */

import { get, writable } from 'svelte/store';
import type { AppView } from './preferences';

export interface SelectionIntent {
  kind: 'file' | 'note' | 'workspace';
  id: string;
}

export const currentView = writable<AppView>('home');
export const selectionIntent = writable<SelectionIntent | null>(null);

export function navigateTo(view: AppView, intent?: SelectionIntent | null) {
  currentView.set(view);
  if (intent) {
    selectionIntent.set(intent);
  }
}

export function consumeSelectionIntent(): SelectionIntent | null {
  const current = get(selectionIntent);
  selectionIntent.set(null);
  return current;
}
