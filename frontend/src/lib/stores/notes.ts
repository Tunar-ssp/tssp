import { writable, derived } from 'svelte/store';
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
    new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
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

export async function setActiveNote(id: string) {
  activeNoteId.set(id);
}

export async function createNewNote() {
  try {
    const newNote = await api.createNote({
      title: 'Untitled Note',
      body: '',
      tags: [],
    });
    notes.update(n => [newNote, ...n]);
    activeNoteId.set(newNote.id);
    return newNote;
  } catch (err) {
    console.error('Failed to create note:', err);
    throw err;
  }
}

export async function updateActiveNote(updates: Partial<Note>) {
  const $activeNoteId = activeNoteId;
  let id: string | null = null;

  activeNoteId.subscribe(val => { id = val; })();

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

export async function deleteNote(id: string) {
  try {
    await api.deleteNote(id);
    notes.update(n => n.filter(note => note.id !== id));
    if (id === activeNoteId) {
      activeNoteId.set(null);
    }
  } catch (err) {
    console.error('Failed to delete note:', err);
    throw err;
  }
}
