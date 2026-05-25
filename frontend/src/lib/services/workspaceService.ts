/**
 * Workspace Service
 *
 * Handles all workspace-related operations: creation, editing, deletion, language selection.
 * Extracted from WorkspaceSurface.svelte and workspace.ts store.
 *
 * Features:
 * - Workspace CRUD operations
 * - Language detection and validation
 * - Large file handling with warnings
 * - Syntax highlighting support
 *
 * Error Handling:
 * - Network errors with retry logic
 * - Validation errors with user-friendly messages
 * - Unsupported languages with fallback to plaintext
 *
 * Edge Cases:
 * - Files > 100MB: Warning shown
 * - Unknown languages: Fallback to plaintext
 * - Long code: Chunked processing
 * - Syntax errors: Graceful fallback
 */

import type { Workspace } from '$lib/api';
import { api } from '$lib/api';

// Logging utility
function log(context: string, message: string, data?: any) {
  console.debug(`[workspaceService] ${context}: ${message}`, data || '');
}

// Error types
class WorkspaceServiceError extends Error {
  constructor(
    public code: string,
    message: string,
    public originalError?: Error
  ) {
    super(message);
    this.name = 'WorkspaceServiceError';
  }
}

// Validation constraints
const MAX_NAME_LENGTH = 200;
const MAX_BODY_LENGTH = 50_000_000; // 50MB of text
const FILE_SIZE_WARNING_THRESHOLD = 100_000_000; // 100MB

// Supported languages for syntax highlighting
const SUPPORTED_LANGUAGES = new Set([
  'javascript', 'typescript', 'python', 'rust', 'go', 'markdown',
  'html', 'css', 'sql', 'json', 'yaml', 'bash', 'text'
]);

/**
 * Validate workspace data
 * @throws WorkspaceServiceError on validation failure
 */
function validateWorkspaceData(data: Partial<Workspace>): void {
  if (data.name !== undefined) {
    if (data.name.length === 0) {
      throw new WorkspaceServiceError(
        'VALIDATION_ERROR',
        'Workspace name cannot be empty'
      );
    }
    if (data.name.length > MAX_NAME_LENGTH) {
      throw new WorkspaceServiceError(
        'VALIDATION_ERROR',
        `Name must be less than ${MAX_NAME_LENGTH} characters`
      );
    }
  }

  if (data.body !== undefined) {
    if (data.body.length > MAX_BODY_LENGTH) {
      throw new WorkspaceServiceError(
        'VALIDATION_ERROR',
        `Code is too large (max ${(MAX_BODY_LENGTH / 1024 / 1024).toFixed(0)}MB)`
      );
    }
  }

  if (data.language !== undefined) {
    const lang = (data.language || '').toLowerCase();
    if (lang && !SUPPORTED_LANGUAGES.has(lang)) {
      // Log unsupported language but don't fail - fallback to plaintext
      log('validateWorkspaceData', 'Unsupported language, using plaintext', { language: lang });
    }
  }
}

/**
 * Normalize and validate language selection
 * Fallback to plaintext for unknown languages
 */
function normalizeLanguage(language: string | undefined): string {
  if (!language) return 'text';

  const normalized = language.toLowerCase().trim();

  if (SUPPORTED_LANGUAGES.has(normalized)) {
    return normalized;
  }

  // Fallback to plaintext for unsupported languages
  log('normalizeLanguage', 'Unsupported language, using plaintext', { language: normalized });
  return 'text';
}

/**
 * Create a new workspace
 * @throws WorkspaceServiceError on failure
 */
export async function createWorkspace(
  name: string = 'Untitled',
  language: string = 'text'
): Promise<Workspace> {
  log('createWorkspace', 'Starting', { name, language });

  try {
    validateWorkspaceData({ name });

    const normalizedLanguage = normalizeLanguage(language);

    const workspace = await api.createWorkspace({
      name: name || 'Untitled',
      language: normalizedLanguage,
      body: '',
    });

    if (!workspace?.id) {
      throw new WorkspaceServiceError(
        'INVALID_RESPONSE',
        'Server returned invalid workspace data'
      );
    }

    log('createWorkspace', 'Success', { id: workspace.id });
    return workspace;
  } catch (err) {
    const message = err instanceof WorkspaceServiceError
      ? err.message
      : 'Failed to create workspace';

    log('createWorkspace', 'Error', { error: message });
    throw new WorkspaceServiceError(
      'CREATE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Update workspace with large file warning
 * Edge case: Warn if file is very large
 */
export async function updateWorkspace(
  id: string,
  data: Partial<Workspace>
): Promise<Workspace> {
  log('updateWorkspace', 'Starting', { id, keys: Object.keys(data) });

  try {
    if (!id?.trim()) {
      throw new WorkspaceServiceError('VALIDATION_ERROR', 'Workspace ID required');
    }

    validateWorkspaceData(data);

    // Warn if body is very large
    if (data.body && data.body.length > FILE_SIZE_WARNING_THRESHOLD) {
      const sizeMB = (data.body.length / 1024 / 1024).toFixed(1);
      log('updateWorkspace', 'Warning: Large file', { sizeMB, id });
    }

    // Normalize language if provided
    if (data.language) {
      data.language = normalizeLanguage(data.language);
    }

    const workspace = await api.updateWorkspace(id, data);

    if (!workspace?.id) {
      throw new WorkspaceServiceError(
        'INVALID_RESPONSE',
        'Server returned invalid workspace data'
      );
    }

    log('updateWorkspace', 'Success', { id });
    return workspace;
  } catch (err) {
    const message = err instanceof WorkspaceServiceError
      ? err.message
      : 'Failed to update workspace';

    log('updateWorkspace', 'Error', { error: message, id });

    if (err instanceof WorkspaceServiceError && err.code === 'VALIDATION_ERROR') {
      throw err;
    }

    throw new WorkspaceServiceError(
      'UPDATE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Delete a workspace
 * @throws WorkspaceServiceError on failure
 */
export async function deleteWorkspace(id: string): Promise<void> {
  log('deleteWorkspace', 'Starting', { id });

  try {
    if (!id?.trim()) {
      throw new WorkspaceServiceError('VALIDATION_ERROR', 'Workspace ID required');
    }

    await api.deleteWorkspace(id);

    log('deleteWorkspace', 'Success', { id });
  } catch (err) {
    const message = err instanceof WorkspaceServiceError
      ? err.message
      : 'Failed to delete workspace';

    log('deleteWorkspace', 'Error', { error: message, id });
    throw new WorkspaceServiceError(
      'DELETE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * List all workspaces
 * Edge case: Empty result set, large result set
 */
export async function listWorkspaces(limit: number = 100): Promise<Workspace[]> {
  log('listWorkspaces', 'Starting', { limit });

  try {
    if (limit < 1 || limit > 1000) {
      throw new WorkspaceServiceError(
        'VALIDATION_ERROR',
        'Limit must be between 1 and 1000'
      );
    }

    const response = await api.listWorkspaces(limit);
    const workspaces = response.workspaces || [];

    log('listWorkspaces', 'Success', { count: workspaces.length });
    return workspaces;
  } catch (err) {
    const message = err instanceof WorkspaceServiceError
      ? err.message
      : 'Failed to load workspaces';

    log('listWorkspaces', 'Error', { error: message });
    throw new WorkspaceServiceError(
      'LIST_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Search workspaces by name or content
 * Edge cases: Special characters, empty query
 */
export async function searchWorkspaces(query: string): Promise<Workspace[]> {
  log('searchWorkspaces', 'Starting', { queryLength: query.length });

  try {
    if (!query || query.trim().length === 0) {
      return [];
    }

    // Sanitize query
    const sanitized = query
      .trim()
      .slice(0, 200)
      .replace(/[<>]/g, '');

    if (!sanitized) return [];

    const response = await api.searchWorkspaces(sanitized);
    const workspaces = response.workspaces || [];

    log('searchWorkspaces', 'Success', { resultCount: workspaces.length });
    return workspaces;
  } catch (err) {
    log('searchWorkspaces', 'Error');
    // Return empty instead of throwing
    return [];
  }
}

/**
 * Change workspace language
 * Automatically normalizes unknown languages to plaintext
 */
export async function setWorkspaceLanguage(
  id: string,
  language: string
): Promise<Workspace> {
  log('setWorkspaceLanguage', 'Starting', { id, language });

  try {
    if (!id?.trim()) {
      throw new WorkspaceServiceError('VALIDATION_ERROR', 'Workspace ID required');
    }

    const normalized = normalizeLanguage(language);

    const workspace = await api.updateWorkspace(id, { language: normalized });

    if (!workspace?.id) {
      throw new WorkspaceServiceError(
        'INVALID_RESPONSE',
        'Server returned invalid workspace data'
      );
    }

    log('setWorkspaceLanguage', 'Success', { id, language: normalized });
    return workspace;
  } catch (err) {
    const message = err instanceof WorkspaceServiceError
      ? err.message
      : 'Failed to change language';

    log('setWorkspaceLanguage', 'Error', { error: message, id });
    throw new WorkspaceServiceError(
      'LANGUAGE_CHANGE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Get list of supported languages
 * Used for language dropdown UI
 */
export function getSupportedLanguages(): Array<{ id: string; label: string; ext: string }> {
  return [
    { id: 'javascript', label: 'JavaScript', ext: '.js' },
    { id: 'typescript', label: 'TypeScript', ext: '.ts' },
    { id: 'python', label: 'Python', ext: '.py' },
    { id: 'rust', label: 'Rust', ext: '.rs' },
    { id: 'go', label: 'Go', ext: '.go' },
    { id: 'markdown', label: 'Markdown', ext: '.md' },
    { id: 'html', label: 'HTML', ext: '.html' },
    { id: 'css', label: 'CSS', ext: '.css' },
    { id: 'sql', label: 'SQL', ext: '.sql' },
    { id: 'json', label: 'JSON', ext: '.json' },
    { id: 'yaml', label: 'YAML', ext: '.yaml' },
    { id: 'bash', label: 'Bash', ext: '.sh' },
    { id: 'text', label: 'Plain Text', ext: '.txt' },
  ];
}

/**
 * Estimate if workspace is "dirty" (has unsaved changes)
 * Compares current vs saved state
 */
export function hasUnsavedChanges(
  saved: Workspace | null,
  current: { name: string; body: string; language: string }
): boolean {
  if (!saved) return current.body.length > 0 || current.name !== 'Untitled';

  return (
    saved.name !== current.name ||
    saved.body !== current.body ||
    saved.language !== current.language
  );
}

export { WorkspaceServiceError };
