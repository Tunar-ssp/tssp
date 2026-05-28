/**
 * Workspace module exports
 * Centralizes workspace-related utilities and types
 */

export { WorkspaceEditorManager } from './WorkspaceEditorManager';
export { default as WorkspaceSurface } from './WorkspaceSurface.svelte';
export { default as WorkspaceSidebar } from './WorkspaceSidebar.svelte';
export { default as WorkspaceHomepage } from './WorkspaceHomepage.svelte';
export { default as WorkspaceFileExplorer } from './WorkspaceFileExplorer.svelte';
export { default as WorkspaceSearch } from './WorkspaceSearch.svelte';
export { default as WorkspaceInspector } from './WorkspaceInspector.svelte';
export { default as TerminalClient } from './TerminalClient.svelte';

// Editor + panel pieces (still used internally)
export { default as FileInfoPanel } from './components/panels/FileInfoPanel.svelte';
export { default as OutlinePanel } from './components/panels/OutlinePanel.svelte';
export { default as EditorSettings } from './components/editors/EditorSettings.svelte';
export { default as TaskPanel } from './components/panels/TaskPanel.svelte';
export { default as DebugPanel } from './components/panels/DebugPanel.svelte';
export { default as DependencyGraph } from './components/panels/DependencyGraph.svelte';
export { default as DiffViewer } from './components/editors/DiffViewer.svelte';
export { default as FileHistory } from './components/panels/FileHistory.svelte';
export { default as KeyboardShortcuts } from './components/panels/KeyboardShortcuts.svelte';
export { default as SnippetLibrary } from './components/panels/SnippetLibrary.svelte';
export { default as AutocompleteSuggestions } from './components/editors/AutocompleteSuggestions.svelte';

// Services
export type { SearchOptions } from '$lib/services/workspaceSearchService';
export { findMatches, replaceMatches } from '$lib/services/workspaceSearchService';
