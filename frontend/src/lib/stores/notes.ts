import { writable } from "svelte/store";
import type { NoteRecord } from "../api";

export const notesList = writable<NoteRecord[]>([]);
export const activeNoteId = writable<string | null>(null);
