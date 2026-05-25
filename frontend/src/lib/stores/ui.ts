import { derived, get, writable } from 'svelte/store';

export type AppView = 'home' | 'drive' | 'notes' | 'workspace' | 'admin';
export type BannerType = 'success' | 'error' | 'info';
export type DockMode = 'always' | 'autohide' | 'compact';
export type DensityMode = 'comfortable' | 'compact';
export type AccentMode = 'green' | 'blue' | 'violet';

export interface SelectionIntent {
  kind: 'file' | 'note' | 'workspace';
  id: string;
}

export interface ShellPreferences {
  theme: 'dark' | 'light';
  accent: AccentMode;
  dockMode: DockMode;
  dockOrder: Array<'drive' | 'notes' | 'workspace' | 'admin'>;
  density: DensityMode;
  defaultDriveView: 'grid' | 'list';
  landingApp: AppView;
}

export interface BannerMessage {
  message: string;
  type: BannerType;
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
export const currentView = writable<AppView>(readPreferences().landingApp);
export const banner = writable<BannerMessage | null>(null);
export const bannerState = banner;
export const commandPaletteOpen = writable(false);
export const commandQuery = writable('');
export const settingsTrayOpen = writable(false);
export const shortcutsOverlayOpen = writable(false);
export const selectionIntent = writable<SelectionIntent | null>(null);

preferences.subscribe((value) => {
  applyDocumentPreferences(value);
  if (isBrowser()) {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
  }
});

applyDocumentPreferences(readPreferences());

export const dockMode = derived(preferences, ($preferences) => $preferences.dockMode);
export const dockOrder = derived(preferences, ($preferences) => $preferences.dockOrder);

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

export function showBanner(message: string, type: BannerType = 'info') {
  banner.set({ message, type });
  if (type !== 'error') {
    setTimeout(() => banner.set(null), 3000);
  }
}

export function openCommandPalette(initialQuery = '') {
  commandQuery.set(initialQuery);
  commandPaletteOpen.set(true);
}

export function closeCommandPalette() {
  commandPaletteOpen.set(false);
}

export function toggleCommandPalette() {
  commandPaletteOpen.update((value) => !value);
}

export function toggleSettingsTray() {
  settingsTrayOpen.update((value) => !value);
}

export function toggleShortcutsOverlay() {
  shortcutsOverlayOpen.update((value) => !value);
}

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
