/**
 * Router: Hash-based view routing
 * 
 * Maps currentView store to app views.
 * Listens to hashchange and updates store accordingly.
 * Public routes: home, drive, notes, workspace, admin, search, public, share, auth
 */

import { currentView, type AppView } from '$lib/stores/ui';
import { get } from 'svelte/store';

export const VALID_VIEWS: AppView[] = [
  'home',
  'drive',
  'notes',
  'workspace',
  'admin',
  'search',
  'public',
  'share',
  'auth',
];

/**
 * Initialize router:
 * - Set initial view from hash
 * - Listen to hashchange
 * Returns cleanup function
 */
export function initRouter() {
  const handleHashChange = () => {
    const hash = window.location.hash.slice(1) || 'home';
    const view = hash as AppView;
    
    if (VALID_VIEWS.includes(view)) {
      currentView.set(view);
    } else {
      // Invalid view, redirect to home
      window.location.hash = '#home';
    }
  };

  // Set initial view
  handleHashChange();

  // Listen for changes
  window.addEventListener('hashchange', handleHashChange);

  return () => {
    window.removeEventListener('hashchange', handleHashChange);
  };
}

/**
 * Navigate to a view by updating hash
 */
export function navigateToView(view: AppView) {
  window.location.hash = `#${view}`;
}

/**
 * Get current view
 */
export function getCurrentView(): AppView {
  return get(currentView);
}
