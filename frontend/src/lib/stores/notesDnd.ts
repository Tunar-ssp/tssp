import { writable } from 'svelte/store';

/** Id of the note currently being dragged in the page tree (null when idle). */
export const draggingNoteId = writable<string | null>(null);
