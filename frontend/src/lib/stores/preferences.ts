/**
 * User Preferences Store
 *
 * Persisted user settings: theme, density, accent, dock configuration.
 */

import { derived, writable } from 'svelte/store';

export type DockMode = 'always' | 'autohide' | 'compact';
export type DensityMode = 'comfortable' | 'compact';
export type AccentMode = 'green' | 'blue' | 'violet';
export type AppView = 'home' | 'drive' | 'notes' | 'workspace' | 'admin';

export interface ShellPreferences {
  theme: 'dark' | 'light';
  accent: AccentMode;
  dockMode: DockMode;
  dockOrder: Array<'drive' | 'notes' | 'workspace' | 'admin'>;
  density: DensityMode;
  defaultDriveView: 'grid' | 'list';
  landingApp: AppView;
}

const STORAGE_KEY = 'tssp.shell.preferences.v2';
const defaultPreferences: ShellPreferences = {
  theme: 'dark',
  accent: 'green',
  dockMode: 'always',
  dockOrder: ['drive', 'notes', 'workspace', 'admin'],
  density: 'comfortable',
  defaultDriveView: 'grid',
  landingApp: 'home',
};

function isBrowser() {
  return typeof window !== 'undefined';
}

function readPreferences(): ShellPreferences {
  if (!isBrowser()) return defaultPreferences;

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaultPreferences;
    const parsed = JSON.parse(raw);
    return {
      ...defaultPreferences,
      ...parsed,
      dockOrder: Array.isArray(parsed?.dockOrder) && parsed.dockOrder.length
        ? parsed.dockOrder.filter((item: string) =>
            defaultPreferences.dockOrder.includes(item as ShellPreferences['dockOrder'][number])
          )
        : defaultPreferences.dockOrder,
    };
  } catch {
    return defaultPreferences;
  }
}

function applyDocumentPreferences(preferences: ShellPreferences) {
  if (!isBrowser()) return;

  document.documentElement.setAttribute('data-theme', preferences.theme);
  document.documentElement.setAttribute('data-density', preferences.density);
  document.documentElement.setAttribute('data-accent', preferences.accent);
}

export const preferences = writable<ShellPreferences>(readPreferences());
export const dockMode = derived(preferences, ($preferences) => $preferences.dockMode);
export const dockOrder = derived(preferences, ($preferences) => $preferences.dockOrder);

preferences.subscribe((value) => {
  applyDocumentPreferences(value);
  if (isBrowser()) {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
  }
});

applyDocumentPreferences(readPreferences());

export function updatePreferences(next: Partial<ShellPreferences>) {
  preferences.update((current) => {
    const merged = { ...current, ...next };
    if (!merged.dockOrder.length) {
      merged.dockOrder = defaultPreferences.dockOrder;
    }
    return merged;
  });
}

export function setTheme(theme: ShellPreferences['theme']) {
  updatePreferences({ theme });
}

export function setAccent(accent: AccentMode) {
  updatePreferences({ accent });
}

export function setDockMode(mode: DockMode) {
  updatePreferences({ dockMode: mode });
}

export function setDensity(density: DensityMode) {
  updatePreferences({ density });
}

export function setDefaultDriveView(view: ShellPreferences['defaultDriveView']) {
  updatePreferences({ defaultDriveView: view });
}

export function setLandingApp(view: AppView) {
  updatePreferences({ landingApp: view });
}

export function moveDockApp(appId: ShellPreferences['dockOrder'][number], direction: -1 | 1) {
  preferences.update((current) => {
    const index = current.dockOrder.indexOf(appId);
    const nextIndex = index + direction;
    if (index < 0 || nextIndex < 0 || nextIndex >= current.dockOrder.length) {
      return current;
    }

    const nextOrder = [...current.dockOrder];
    const [item] = nextOrder.splice(index, 1);
    nextOrder.splice(nextIndex, 0, item);
    return { ...current, dockOrder: nextOrder };
  });
}
