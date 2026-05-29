import { writable, derived, get } from 'svelte/store';
import { api, type Note } from '../api';

export const notes = writable<Note[]>([]);
export const activeNoteId = writable<string | null>(null);
export const isLoading = writable(false);
export const isSaving = writable(false);
export const searchQuery = writable('');

export const activeNote = derived(
  [notes, activeNoteId],
  ([$notes, $id]) => $notes.find((n) => n.id === $id) || null
);

export const filteredNotes = derived(
  [notes, searchQuery],
  ([$notes, $query]) => {
    if (!$query.trim()) return $notes;
    const q = $query.toLowerCase();
    return $notes.filter(n =>
      n.title.toLowerCase().includes(q) ||
      n.body.toLowerCase().includes(q) ||
      n.tags.some(t => t.toLowerCase().includes(q))
    );
  }
);

export const sortedNotes = derived(filteredNotes, ($notes) =>
  [...$notes].sort((a, b) =>
    (b.pinned_at ? 1_000_000_000_000 + b.pinned_at : b.updated_at) -
    (a.pinned_at ? 1_000_000_000_000 + a.pinned_at : a.updated_at)
  )
);

export async function loadNotes() {
  isLoading.set(true);
  try {
    const data = await api.listNotes();
    notes.set(data.notes || []);
  } finally {
    isLoading.set(false);
  }
}

export function setActiveNote(id: string | null) {
  activeNoteId.set(id);
}

export async function createNewNote(parentId: string | null = null) {
  try {
    const newNote = await api.createNote({
      title: 'Untitled',
      body: '',
      tags: [],
      parent_id: parentId,
    });
    notes.update(n => [newNote, ...n]);
    activeNoteId.set(newNote.id);
    return newNote;
  } catch (err) {
    console.error('Failed to create note:', err);
    throw err;
  }
}

/** Moves a note under a new parent (null = top level). */
export async function moveNoteToParent(id: string, parentId: string | null) {
  const updated = await api.moveNote(id, parentId);
  notes.update(n => n.map(note => note.id === id ? updated : note));
  return updated;
}

/** Sets or clears a note's icon. */
export async function setNoteIcon(id: string, icon: string | null) {
  const updated = await api.setNoteIcon(id, icon);
  notes.update(n => n.map(note => note.id === id ? updated : note));
  return updated;
}

export async function updateActiveNote(updates: Partial<Note>) {
  const id = get(activeNoteId);

  if (!id) return;

  isSaving.set(true);
  try {
    const updated = await api.updateNote(id, updates);
    notes.update(n => n.map(note => note.id === id ? updated : note));
  } catch (err) {
    console.error('Failed to update note:', err);
    throw err;
  } finally {
    isSaving.set(false);
  }
}

/**
 * Persist updates to a specific note by id (independent of which note is
 * currently active). Used to flush a pending autosave when switching notes so
 * the outgoing note's edits are never written onto the incoming note.
 */
export async function saveNote(id: string, updates: Partial<Note>) {
  if (!id) return;
  isSaving.set(true);
  try {
    const updated = await api.updateNote(id, updates);
    notes.update(n => n.map(note => note.id === id ? updated : note));
  } catch (err) {
    console.error('Failed to save note:', err);
    throw err;
  } finally {
    isSaving.set(false);
  }
}

export async function replaceActiveNoteTags(tags: string[]) {
  const id = get(activeNoteId);
  if (!id) return;

  isSaving.set(true);
  try {
    await api.replaceNoteTags(id, tags);
    notes.update(n => n.map(note => note.id === id ? { ...note, tags } : note));
  } catch (err) {
    console.error('Failed to update note tags:', err);
    throw err;
  } finally {
    isSaving.set(false);
  }
}

export async function duplicateNote(id: string) {
  try {
    const duplicated = await api.duplicateNote(id);
    notes.update(n => [duplicated, ...n]);
    activeNoteId.set(duplicated.id);
    return duplicated;
  } catch (err) {
    console.error('Failed to duplicate note:', err);
    throw err;
  }
}

export async function toggleNotePin(id: string, currentlyPinned: boolean) {
  const prevActive = get(activeNoteId);
  try {
    if (currentlyPinned) {
      await api.unpinNote(id);
    } else {
      await api.pinNote(id);
    }
    await loadNotes();
    if (prevActive) activeNoteId.set(prevActive);
  } catch (err) {
    console.error('Failed to update note pin:', err);
    throw err;
  }
}

export async function deleteNote(id: string) {
  try {
    await api.deleteNote(id);
    notes.update(n => n.filter(note => note.id !== id));
    if (id === get(activeNoteId)) {
      activeNoteId.set(null);
    }
  } catch (err) {
    console.error('Failed to delete note:', err);
    throw err;
  }
}
