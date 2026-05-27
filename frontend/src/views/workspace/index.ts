/**
 * Workspace module exports
 * Centralizes workspace-related utilities and types
 */

export { WorkspaceEditorManager } from './WorkspaceEditorManager';
export { default as WorkspaceSurface } from './WorkspaceSurface.svelte';
export { default as WorkspaceView } from './WorkspaceView.svelte';
export { default as WorkspaceSidebar } from './WorkspaceSidebar.svelte';
export { default as WorkspaceHomepage } from './WorkspaceHomepage.svelte';

// Panels
export { default as WorkspaceExplorer } from './WorkspaceExplorer.svelte';
export { default as FileInfoPanel } from './FileInfoPanel.svelte';
export { default as OutlinePanel } from './OutlinePanel.svelte';
export { default as EditorSettings } from './EditorSettings.svelte';
export { default as WorkspaceStats } from './WorkspaceStats.svelte';
export { default as TaskPanel } from './TaskPanel.svelte';
export { default as DebugPanel } from './DebugPanel.svelte';
export { default as VersionControl } from './VersionControl.svelte';
export { default as DependencyGraph } from './DependencyGraph.svelte';
export { default as DiffViewer } from './DiffViewer.svelte';
export { default as FileHistory } from './FileHistory.svelte';
export { default as KeyboardShortcuts } from './KeyboardShortcuts.svelte';
export { default as SnippetLibrary } from './SnippetLibrary.svelte';
export { default as AutocompleteSuggestions } from './AutocompleteSuggestions.svelte';
export { default as BreadcrumbNav } from './BreadcrumbNav.svelte';
export { default as EnhancedWorkspaceSidebar } from './EnhancedWorkspaceSidebar.svelte';

// Services
export type { SearchOptions } from '$lib/services/workspaceSearchService';
export { findMatches, replaceMatches } from '$lib/services/workspaceSearchService';
