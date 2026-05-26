/**
 * Application configuration and metadata
 * Centralized configuration for app views, navigation, and branding
 */

import * as Icons from 'lucide-svelte';
import type { AppView } from '$lib/stores/ui';
import HomeView from '$views/home/HomeLauncher.svelte';
import DriveView from '$views/drive/DriveView.svelte';
import NotesView from '$views/notes/NotesSurface.svelte';
import WorkspaceView from '$views/workspace/WorkspaceSurface.svelte';
import OperationsView from '$views/operations/OperationsView.svelte';

/**
 * App view metadata and configuration
 */
export interface AppViewConfig {
  title: string;
  icon: any;
  accent: string;
  crumbs: string[];
  requiresAdmin?: boolean;
}

/**
 * View component registry
 */
export const viewRegistry = {
  home: HomeView,
  drive: DriveView,
  notes: NotesView,
  workspace: WorkspaceView,
  admin: OperationsView,
} as const;

/**
 * View metadata
 */
export const appMeta: Record<AppView, AppViewConfig> = {
  home: {
    title: 'Launcher',
    icon: Icons.Home,
    accent: '#7c8190',
    crumbs: ['Launcher'],
  },
  drive: {
    title: 'Cloud Drive',
    icon: Icons.Cloud,
    accent: '#6ea8ff',
    crumbs: ['Drive'],
  },
  notes: {
    title: 'Notes',
    icon: Icons.BookText,
    accent: '#5be39a',
    crumbs: ['Notes'],
  },
  workspace: {
    title: 'Workspace',
    icon: Icons.Code2,
    accent: '#ff8a3d',
    crumbs: ['Workspace'],
  },
  admin: {
    title: 'Admin',
    icon: Icons.Shield,
    accent: '#a394ff',
    crumbs: ['Admin'],
    requiresAdmin: true,
  },
};

/**
 * Global keyboard commands available in command palette
 */
export const globalCommands = [
  {
    id: 'command-settings',
    label: 'Open settings',
    description: 'Adjust dock, theme, density, and defaults',
    icon: Icons.Settings2,
    shortcut: '⌘,',
  },
  {
    id: 'command-home',
    label: 'Open launcher',
    description: 'Return to the product home screen',
    icon: Icons.Home,
    shortcut: '⌘1',
  },
  {
    id: 'command-drive',
    label: 'Open drive',
    description: 'Access your cloud storage',
    icon: Icons.Cloud,
    shortcut: '⌘2',
  },
  {
    id: 'command-notes',
    label: 'Open notes',
    description: 'View and manage your notes',
    icon: Icons.BookText,
    shortcut: '⌘3',
  },
  {
    id: 'command-workspace',
    label: 'Open workspace',
    description: 'Edit and run code snippets',
    icon: Icons.Code2,
    shortcut: '⌘4',
  },
];

/**
 * Get view component by key
 */
export function getViewComponent(view: AppView) {
  return viewRegistry[view] || viewRegistry.home;
}

/**
 * Get view metadata
 */
export function getViewMetadata(view: AppView): AppViewConfig {
  return appMeta[view] || appMeta.home;
}

/**
 * Check if view requires admin
 */
export function viewRequiresAdmin(view: AppView): boolean {
  return appMeta[view]?.requiresAdmin || false;
}
