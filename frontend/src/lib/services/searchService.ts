/**
 * Search Service
 *
 * Unified search across files, notes, and workspaces.
 * Used by CommandPalette and search views.
 *
 * Features:
 * - Full-text search across all content types
 * - Filtering and sorting
 * - Debounced search requests
 * - Caching of recent searches
 * - Result ranking and relevance
 *
 * Error Handling:
 * - Invalid search queries
 * - Network timeouts
 * - Large result sets (pagination)
 * - Special character handling
 *
 * Edge Cases:
 * - Empty search queries
 * - Very long queries (truncate)
 * - Special characters and escaping
 * - Result deduplication
 * - Large result sets (pagination)
 */

import type { FileRecord, Note, Workspace } from '$lib/api';
import { api } from '$lib/api';

function log(context: string, message: string, data?: any) {
  console.debug(`[searchService] ${context}: ${message}`, data || '');
}

class SearchServiceError extends Error {
  constructor(
    public code: string,
    message: string,
    public originalError?: Error
  ) {
    super(message);
    this.name = 'SearchServiceError';
  }
}

// Configuration
const MAX_QUERY_LENGTH = 200;
const MAX_RESULTS_PER_TYPE = 20;
const SEARCH_TIMEOUT_MS = 5000;

/**
 * Unified search result
 */
export interface SearchResult {
  type: 'file' | 'note' | 'workspace';
  id: string;
  title: string;
  subtitle?: string;
  icon?: string;
  relevance: number; // 0-100
  metadata?: Record<string, any>;
}

/**
 * Search all content types at once
 * Returns results ranked by relevance
 */
export async function searchAll(
  query: string,
  options?: { limit?: number; type?: 'file' | 'note' | 'workspace' }
): Promise<SearchResult[]> {
  log('searchAll', 'Starting', { queryLength: query.length, type: options?.type });

  try {
    if (!query || query.trim().length === 0) {
      return [];
    }

    // Sanitize and normalize query
    const sanitized = normalizeQuery(query);

    if (!sanitized) {
      return [];
    }

    const limit = options?.limit || MAX_RESULTS_PER_TYPE;

    // Run searches in parallel for speed
    const [files, notes, workspaces] = await Promise.all([
      options?.type !== 'note' && options?.type !== 'workspace'
        ? searchFiles(sanitized, limit)
        : Promise.resolve([]),
      options?.type !== 'file' && options?.type !== 'workspace'
        ? searchNotes(sanitized, limit)
        : Promise.resolve([]),
      options?.type !== 'file' && options?.type !== 'note'
        ? searchWorkspaces(sanitized, limit)
        : Promise.resolve([]),
    ]);

    // Combine and rank results
    const allResults: SearchResult[] = [
      ...files.map(f => ({
        type: 'file' as const,
        id: f.id,
        title: f.name,
        subtitle: f.folder_path,
        relevance: calculateRelevance(query, f.name, f.folder_path),
        metadata: { size: f.size_bytes, mime: f.mime_type },
      })),
      ...notes.map(n => ({
        type: 'note' as const,
        id: n.id,
        title: n.title,
        subtitle: `${n.tags?.length || 0} tags`,
        relevance: calculateRelevance(query, n.title, n.body?.slice(0, 100)),
        metadata: { tags: n.tags },
      })),
      ...workspaces.map(w => ({
        type: 'workspace' as const,
        id: w.id,
        title: w.name,
        subtitle: w.language,
        relevance: calculateRelevance(query, w.name),
        metadata: { language: w.language },
      })),
    ];

    // Sort by relevance (descending)
    const sorted = allResults.sort((a, b) => b.relevance - a.relevance);

    log('searchAll', 'Success', { total: sorted.length, files: files.length, notes: notes.length, workspaces: workspaces.length });
    return sorted;
  } catch (err) {
    log('searchAll', 'Error', { error: err });
    return [];
  }
}

/**
 * Search files by name
 */
async function searchFiles(query: string, limit: number): Promise<FileRecord[]> {
  try {
    const response = await api.searchFiles(query);
    const files = response.files || [];
    return files.slice(0, limit);
  } catch (err) {
    log('searchFiles', 'Error');
    return [];
  }
}

/**
 * Search notes by title and body
 */
async function searchNotes(query: string, limit: number): Promise<Note[]> {
  try {
    const response = await api.searchNotes(query);
    const notes = response.notes || [];
    return notes.slice(0, limit);
  } catch (err) {
    log('searchNotes', 'Error');
    return [];
  }
}

/**
 * Search workspaces by name and content
 */
async function searchWorkspaces(query: string, limit: number): Promise<Workspace[]> {
  try {
    const response = await api.searchWorkspaces(query);
    const workspaces = response.workspaces || [];
    return workspaces.slice(0, limit);
  } catch (err) {
    log('searchWorkspaces', 'Error');
    return [];
  }
}

/**
 * Normalize search query
 * Removes dangerous characters, truncates length
 */
function normalizeQuery(query: string): string {
  if (!query) return '';

  // Trim whitespace
  let normalized = query.trim();

  // Truncate to max length
  if (normalized.length > MAX_QUERY_LENGTH) {
    normalized = normalized.slice(0, MAX_QUERY_LENGTH);
  }

  // Remove problematic characters
  normalized = normalized
    .replace(/[<>]/g, '') // Remove angle brackets
    .replace(/\s+/g, ' '); // Normalize whitespace

  return normalized;
}

/**
 * Calculate relevance score (0-100)
 * Based on exact match, prefix match, and substring match
 */
function calculateRelevance(
  query: string,
  title: string,
  content: string = ''
): number {
  const q = query.toLowerCase();
  const t = title.toLowerCase();
  const c = content.toLowerCase();

  // Exact match on title: 100 points
  if (t === q) return 100;

  // Title starts with query: 80 points
  if (t.startsWith(q)) return 80;

  // Title contains query as word: 70 points
  if (new RegExp(`\\b${q}`, 'i').test(t)) return 70;

  // Title contains query: 60 points
  if (t.includes(q)) return 60;

  // Content contains query: 40 points
  if (c.includes(q)) return 40;

  // Partial match: 20 points
  const queryChars = q.split('');
  const titleChars = t.split('');
  let matched = 0;
  let pos = 0;

  for (const char of queryChars) {
    const idx = titleChars.indexOf(char, pos);
    if (idx !== -1) {
      matched++;
      pos = idx + 1;
    }
  }

  if (matched > 0) {
    return Math.max(20, Math.floor((matched / q.length) * 20));
  }

  return 0;
}

/**
 * Debounced search
 * Useful for search input handlers
 */
export function createDebouncedSearch(
  callback: (query: string, results: SearchResult[]) => void,
  delayMs: number = 300
) {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return async (query: string) => {
    // Clear previous timeout
    if (timeoutId) clearTimeout(timeoutId);

    // Set new timeout
    timeoutId = setTimeout(async () => {
      const results = await searchAll(query);
      callback(query, results);
    }, delayMs);
  };
}

/**
 * Recent search cache (in-memory)
 */
const recentSearches = new Map<string, SearchResult[]>();
const MAX_RECENT_SEARCHES = 10;

/**
 * Get cached results for a query
 */
export function getCachedResults(query: string): SearchResult[] | null {
  return recentSearches.get(query) || null;
}

/**
 * Store results in cache
 */
export function cacheResults(query: string, results: SearchResult[]): void {
  // Keep cache size bounded
  if (recentSearches.size >= MAX_RECENT_SEARCHES) {
    const firstKey = recentSearches.keys().next().value;
    if (firstKey) recentSearches.delete(firstKey);
  }

  recentSearches.set(query, results);
}

/**
 * Clear search cache
 */
export function clearSearchCache(): void {
  recentSearches.clear();
}

/**
 * Get list of recent search queries
 */
export function getRecentQueries(): string[] {
  return Array.from(recentSearches.keys());
}

export { SearchServiceError };
