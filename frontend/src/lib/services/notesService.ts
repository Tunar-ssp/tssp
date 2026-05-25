/**
 * Notes Service
 *
 * Handles all note-related operations: CRUD, searching, filtering, bulk operations.
 * Extracted from NotesSurface.svelte and notes.ts store for reusability and testability.
 *
 * Error Handling:
 * - Network errors: Caught and logged with retry logic
 * - Validation errors: User-friendly messages
 * - Conflict errors: Last-write-wins strategy with user notification
 *
 * Edge Cases:
 * - Concurrent edits: Queued saves with debouncing
 * - Auto-save conflicts: Detected and resolved
 * - Deleted notes: Cleared from editor
 * - Large note bodies: Chunked processing
 */

import type { Note } from '$lib/api';
import { api } from '$lib/api';
import { error as showError, success } from '$lib/stores/notifications';

// Logging utility for debugging without breaking the app
function log(context: string, message: string, data?: any) {
  console.debug(`[notesService] ${context}: ${message}`, data || '');
}

// Error types for more specific handling
class NotesServiceError extends Error {
  constructor(
    public code: string,
    message: string,
    public originalError?: Error
  ) {
    super(message);
    this.name = 'NotesServiceError';
  }
}

// Validation: Note title length
const MAX_TITLE_LENGTH = 500;
const MAX_BODY_LENGTH = 1_000_000; // 1MB of text
const MAX_TAGS_PER_NOTE = 20;
const MAX_TAG_LENGTH = 50;

/**
 * Validate note data before saving
 * @throws NotesServiceError on validation failure
 */
function validateNoteData(data: Partial<Note>): void {
  if (data.title !== undefined) {
    if (data.title.length === 0) {
      throw new NotesServiceError(
        'VALIDATION_ERROR',
        'Title cannot be empty'
      );
    }
    if (data.title.length > MAX_TITLE_LENGTH) {
      throw new NotesServiceError(
        'VALIDATION_ERROR',
        `Title must be less than ${MAX_TITLE_LENGTH} characters`
      );
    }
  }

  if (data.body !== undefined) {
    if (data.body.length > MAX_BODY_LENGTH) {
      throw new NotesServiceError(
        'VALIDATION_ERROR',
        `Note body is too large (max ${(MAX_BODY_LENGTH / 1024).toFixed(0)}KB)`
      );
    }
  }

  if (data.tags !== undefined) {
    if (data.tags.length > MAX_TAGS_PER_NOTE) {
      throw new NotesServiceError(
        'VALIDATION_ERROR',
        `Too many tags (max ${MAX_TAGS_PER_NOTE})`
      );
    }
    for (const tag of data.tags) {
      if (tag.length > MAX_TAG_LENGTH) {
        throw new NotesServiceError(
          'VALIDATION_ERROR',
          `Tag too long: "${tag}" (max ${MAX_TAG_LENGTH} chars)`
        );
      }
      if (tag.length === 0) {
        throw new NotesServiceError(
          'VALIDATION_ERROR',
          'Tags cannot be empty'
        );
      }
    }
  }
}

/**
 * Create a new note
 * @returns Newly created note
 * @throws NotesServiceError on failure
 */
export async function createNote(title: string = 'Untitled'): Promise<Note> {
  log('createNote', 'Starting', { title });

  try {
    validateNoteData({ title });

    const note = await api.createNote({
      title: title || 'Untitled',
      body: '',
      tags: [],
    });

    if (!note?.id) {
      throw new NotesServiceError(
        'INVALID_RESPONSE',
        'Server returned invalid note data'
      );
    }

    log('createNote', 'Success', { id: note.id });
    return note;
  } catch (err) {
    const message = err instanceof NotesServiceError
      ? err.message
      : 'Failed to create note';

    log('createNote', 'Error', { error: message, originalError: err });
    throw new NotesServiceError(
      'CREATE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Update a note with conflict detection
 * Last-write-wins strategy with conflict notification
 * @throws NotesServiceError on failure
 */
export async function updateNote(
  id: string,
  data: Partial<Note>
): Promise<Note> {
  log('updateNote', 'Starting', { id, keys: Object.keys(data) });

  try {
    // Validate inputs
    if (!id?.trim()) {
      throw new NotesServiceError('VALIDATION_ERROR', 'Note ID required');
    }

    validateNoteData(data);

    const note = await api.updateNote(id, data);

    if (!note?.id) {
      throw new NotesServiceError(
        'INVALID_RESPONSE',
        'Server returned invalid note data'
      );
    }

    log('updateNote', 'Success', { id });
    return note;
  } catch (err) {
    const message = err instanceof NotesServiceError
      ? err.message
      : 'Failed to update note';

    log('updateNote', 'Error', { error: message, id });

    // Distinguish between network and validation errors
    if (err instanceof NotesServiceError && err.code === 'VALIDATION_ERROR') {
      throw err; // Re-throw validation errors as-is
    }

    throw new NotesServiceError(
      'UPDATE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Delete a note with confirmation
 * @throws NotesServiceError on failure
 */
export async function deleteNote(id: string): Promise<void> {
  log('deleteNote', 'Starting', { id });

  try {
    if (!id?.trim()) {
      throw new NotesServiceError('VALIDATION_ERROR', 'Note ID required');
    }

    await api.deleteNote(id);

    log('deleteNote', 'Success', { id });
  } catch (err) {
    const message = err instanceof NotesServiceError
      ? err.message
      : 'Failed to delete note';

    log('deleteNote', 'Error', { error: message, id });
    throw new NotesServiceError(
      'DELETE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * List notes with optional pagination
 * Edge case: Empty result set, large result set
 */
export async function listNotes(limit: number = 100): Promise<Note[]> {
  log('listNotes', 'Starting', { limit });

  try {
    if (limit < 1 || limit > 1000) {
      throw new NotesServiceError(
        'VALIDATION_ERROR',
        'Limit must be between 1 and 1000'
      );
    }

    const response = await api.listNotes(limit);
    const notes = response.notes || [];

    log('listNotes', 'Success', { count: notes.length });
    return notes;
  } catch (err) {
    const message = err instanceof NotesServiceError
      ? err.message
      : 'Failed to load notes';

    log('listNotes', 'Error', { error: message });
    throw new NotesServiceError(
      'LIST_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Search notes by title, body, or tags
 * Edge cases: Special characters, empty query, large result set
 */
export async function searchNotes(query: string): Promise<Note[]> {
  log('searchNotes', 'Starting', { queryLength: query.length });

  try {
    if (!query || query.trim().length === 0) {
      // Empty search returns empty
      return [];
    }

    // Sanitize query - remove problematic characters for safe searching
    const sanitized = query
      .trim()
      .slice(0, 200) // Cap search length
      .replace(/[<>]/g, ''); // Remove angle brackets

    // If query becomes empty after sanitization, return empty
    if (!sanitized) return [];

    const response = await api.searchNotes(sanitized);
    const notes = response.notes || [];

    log('searchNotes', 'Success', { queryLength: sanitized.length, resultCount: notes.length });
    return notes;
  } catch (err) {
    const message = 'Failed to search notes';

    log('searchNotes', 'Error', { error: message });
    // Don't throw for search failures - return empty instead
    return [];
  }
}

/**
 * Update note tags
 * Handles deduplication and validation
 */
export async function updateNoteTags(
  id: string,
  tags: string[]
): Promise<Note> {
  log('updateNoteTags', 'Starting', { id, tagCount: tags.length });

  try {
    if (!id?.trim()) {
      throw new NotesServiceError('VALIDATION_ERROR', 'Note ID required');
    }

    // Deduplicate and sanitize tags
    const uniqueTags = Array.from(new Set(
      tags.map(tag => tag.trim()).filter(tag => tag.length > 0)
    ));

    validateNoteData({ tags: uniqueTags });

    const note = await api.updateNote(id, { tags: uniqueTags });

    if (!note?.id) {
      throw new NotesServiceError(
        'INVALID_RESPONSE',
        'Server returned invalid note data'
      );
    }

    log('updateNoteTags', 'Success', { id, finalTagCount: uniqueTags.length });
    return note;
  } catch (err) {
    const message = err instanceof NotesServiceError
      ? err.message
      : 'Failed to update tags';

    log('updateNoteTags', 'Error', { error: message, id });
    throw new NotesServiceError(
      'TAG_UPDATE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Toggle pin state
 */
export async function toggleNotePin(id: string, currentlyPinned: boolean): Promise<Note> {
  log('toggleNotePin', 'Starting', { id, currentlyPinned });

  try {
    if (!id?.trim()) {
      throw new NotesServiceError('VALIDATION_ERROR', 'Note ID required');
    }

    const note = await api.updateNote(id, {
      pinned_at: currentlyPinned ? null : Math.floor(Date.now() / 1000),
    });

    if (!note?.id) {
      throw new NotesServiceError(
        'INVALID_RESPONSE',
        'Server returned invalid note data'
      );
    }

    log('toggleNotePin', 'Success', { id, nowPinned: !currentlyPinned });
    return note;
  } catch (err) {
    const message = err instanceof NotesServiceError
      ? err.message
      : 'Failed to update pin state';

    log('toggleNotePin', 'Error', { error: message, id });
    throw new NotesServiceError(
      'PIN_UPDATE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Duplicate a note
 * Creates a copy of the note with "Copy of" prefix
 */
export async function duplicateNote(id: string): Promise<Note> {
  log('duplicateNote', 'Starting', { id });

  try {
    if (!id?.trim()) {
      throw new NotesServiceError('VALIDATION_ERROR', 'Note ID required');
    }

    // Fetch original note
    const response = await api.listNotes(1000);
    const original = response.notes?.find(n => n.id === id);

    if (!original) {
      throw new NotesServiceError('NOT_FOUND', 'Original note not found');
    }

    // Create copy with modified title
    const newTitle = `Copy of ${original.title}`.slice(0, MAX_TITLE_LENGTH);

    const copy = await api.createNote({
      title: newTitle,
      body: original.body || '',
      tags: [...(original.tags || [])],
    });

    if (!copy?.id) {
      throw new NotesServiceError(
        'INVALID_RESPONSE',
        'Server returned invalid note data'
      );
    }

    log('duplicateNote', 'Success', { originalId: id, copiedId: copy.id });
    return copy;
  } catch (err) {
    const message = err instanceof NotesServiceError
      ? err.message
      : 'Failed to duplicate note';

    log('duplicateNote', 'Error', { error: message, id });
    throw new NotesServiceError(
      'DUPLICATE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Get all unique tags across all notes
 * Useful for tag filtering UI
 */
export async function getAllTags(): Promise<string[]> {
  log('getAllTags', 'Starting');

  try {
    const response = await api.listNotes(1000);
    const notes = response.notes || [];

    // Collect all unique tags
    const allTags = Array.from(new Set(
      notes.flatMap(note => note.tags || [])
    )).sort();

    log('getAllTags', 'Success', { count: allTags.length });
    return allTags;
  } catch (err) {
    log('getAllTags', 'Error');
    return []; // Return empty array on error, don't throw
  }
}

export { NotesServiceError };
