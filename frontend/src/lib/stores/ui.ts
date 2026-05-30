import { derived, get, writable } from 'svelte/store';
import { dismissDialog } from './dialog';

export type AppView = 'home' | 'drive' | 'notes' | 'workspace' | 'admin';
export type BannerType = 'success' | 'error' | 'info';
export type DockMode = 'always' | 'autohide' | 'compact' | 'hidden';
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
  tone?: BannerType;
  title?: string;
  detail?: string;
}

const STORAGE_KEY = 'tssp.shell.preferences.v2';
const defaultPreferences: ShellPreferences = {
  theme: 'dark',
  accent: 'green',
  dockMode: 'autohide',
  dockOrder: ['drive', 'notes', 'workspace', 'admin'],
  density: 'comfortable',
  defaultDriveView: 'grid',
  landingApp: 'home',
};

function isBrowser() {
  return typeof window !== 'undefined';
}

function parseViewFromHash(): AppView | null {
  if (!isBrowser()) return null;

  const raw = window.location.hash.replace(/^#/, '').trim().toLowerCase();
  switch (raw) {
    case 'home':
    case 'drive':
    case 'notes':
    case 'workspace':
    case 'admin':
      return raw;
    default:
      return null;
  }
}

function readInitialView(): AppView {
  return parseViewFromHash() || readPreferences().landingApp;
}

function syncHash(view: AppView) {
  if (!isBrowser()) return;
  const nextHash = `#${view}`;
  if (window.location.hash !== nextHash) {
    window.location.hash = nextHash;
  }
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
        ? Array.from(
            new Set(
              parsed.dockOrder.filter((item: string) =>
                defaultPreferences.dockOrder.includes(item as ShellPreferences['dockOrder'][number])
              )
            )
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
export const currentView = writable<AppView>(readInitialView());
export const banner = writable<BannerMessage | null>(null);
export const bannerState = banner;
export const commandPaletteOpen = writable(false);
export const commandQuery = writable('');
export const settingsTrayOpen = writable(false);
export const shortcutsOverlayOpen = writable(false);
export const selectionIntent = writable<SelectionIntent | null>(null);

export type OverlayId = 'command-palette' | 'settings-tray' | 'shortcuts' | 'modal' | 'preview' | 'context-menu';
const overlayStack = writable<OverlayId[]>([]);

export const activeOverlays = {
  subscribe: overlayStack.subscribe,
  push: (id: OverlayId) => overlayStack.update(stack => [...stack.filter(i => i !== id), id]),
  pop: () => {
    let popped: OverlayId | undefined;
    overlayStack.update(stack => {
      popped = stack[stack.length - 1];
      return stack.slice(0, -1);
    });
    return popped;
  },
  remove: (id: OverlayId) => overlayStack.update(stack => stack.filter(i => i !== id)),
  peek: (): OverlayId | undefined => {
    const stack = get(overlayStack);
    return stack[stack.length - 1];
  },
  isTop: (id: OverlayId) => {
    const stack = get(overlayStack);
    return stack[stack.length - 1] === id;
  },
  hasAny: derived(overlayStack, stack => stack.length > 0)
};

preferences.subscribe((value) => {
  applyDocumentPreferences(value);
  if (isBrowser()) {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
  }
});

applyDocumentPreferences(readPreferences());

if (isBrowser()) {
  window.addEventListener('hashchange', () => {
    const view = parseViewFromHash();
    if (view) {
      currentView.set(view);
    }
  });
}

export const dockMode = derived(preferences, ($preferences) => $preferences.dockMode);
export const dockOrder = derived(preferences, ($preferences) => $preferences.dockOrder);

/**
 * Close every global/transient overlay that the shell owns. View-scoped overlays
 * (modals, previews, context menus rendered inside a view) clean themselves up via
 * their own lifecycle when the view unmounts on navigation.
 */
export function closeTransientOverlays() {
  if (get(commandPaletteOpen)) closeCommandPalette();
  if (get(settingsTrayOpen)) closeSettingsTray();
  if (get(shortcutsOverlayOpen)) closeShortcutsOverlay();
  dismissDialog();
}

export function navigateTo(view: AppView, intent?: SelectionIntent | null) {
  // Navigation must never leave a conflicting overlay stuck on top of the new view.
  closeTransientOverlays();
  currentView.set(view);
  syncHash(view);
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
  activeOverlays.push('command-palette');
}

export function closeCommandPalette() {
  commandPaletteOpen.set(false);
  activeOverlays.remove('command-palette');
}

export function toggleCommandPalette() {
  const isOpen = get(commandPaletteOpen);
  if (isOpen) closeCommandPalette();
  else openCommandPalette();
}

export function openSettingsTray() {
  settingsTrayOpen.set(true);
  activeOverlays.push('settings-tray');
}

export function closeSettingsTray() {
  settingsTrayOpen.set(false);
  activeOverlays.remove('settings-tray');
}

export function toggleSettingsTray() {
  const isOpen = get(settingsTrayOpen);
  if (isOpen) closeSettingsTray();
  else openSettingsTray();
}

export function openShortcutsOverlay() {
  shortcutsOverlayOpen.set(true);
  activeOverlays.push('shortcuts');
}

export function closeShortcutsOverlay() {
  shortcutsOverlayOpen.set(false);
  activeOverlays.remove('shortcuts');
}

export function toggleShortcutsOverlay() {
  const isOpen = get(shortcutsOverlayOpen);
  if (isOpen) closeShortcutsOverlay();
  else openShortcutsOverlay();
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
