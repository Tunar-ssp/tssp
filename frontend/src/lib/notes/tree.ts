/**
 * Builds the nested page tree for the Notes sidebar from a flat note list,
 * using each note's `parent_id`. Mirrors Notion's page-inside-page model.
 */
import type { Note } from '$lib/api';

export interface NoteTreeNode {
  note: Note;
  children: NoteTreeNode[];
  depth: number;
}

function siblingSort(a: Note, b: Note): number {
  // Stable ordering: oldest first, so newly created pages append predictably.
  if (a.created_at !== b.created_at) return a.created_at - b.created_at;
  return a.id.localeCompare(b.id);
}

/**
 * Returns the top-level tree nodes. Notes whose `parent_id` points at a missing
 * note are treated as top level so nothing is ever hidden.
 */
export function buildNoteTree(notes: Note[]): NoteTreeNode[] {
  const byId = new Map<string, Note>();
  for (const note of notes) byId.set(note.id, note);

  const childrenOf = new Map<string | null, Note[]>();
  for (const note of notes) {
    const parent = note.parent_id && byId.has(note.parent_id) ? note.parent_id : null;
    const bucket = childrenOf.get(parent) ?? [];
    bucket.push(note);
    childrenOf.set(parent, bucket);
  }

  const build = (parentId: string | null, depth: number): NoteTreeNode[] => {
    const kids = (childrenOf.get(parentId) ?? []).slice().sort(siblingSort);
    return kids.map((note) => ({
      note,
      depth,
      children: build(note.id, depth + 1),
    }));
  };

  return build(null, 0);
}

/** Collects a note id plus all of its descendant ids. */
export function collectSubtreeIds(notes: Note[], rootId: string): Set<string> {
  const childrenOf = new Map<string, string[]>();
  for (const note of notes) {
    if (note.parent_id) {
      const bucket = childrenOf.get(note.parent_id) ?? [];
      bucket.push(note.id);
      childrenOf.set(note.parent_id, bucket);
    }
  }
  const result = new Set<string>();
  const stack = [rootId];
  while (stack.length) {
    const id = stack.pop()!;
    if (result.has(id)) continue;
    result.add(id);
    for (const child of childrenOf.get(id) ?? []) stack.push(child);
  }
  return result;
}

/** Returns the ancestor ids of a note (nearest parent first). */
export function ancestorIds(notes: Note[], id: string): string[] {
  const byId = new Map<string, Note>();
  for (const note of notes) byId.set(note.id, note);
  const chain: string[] = [];
  let current = byId.get(id)?.parent_id ?? null;
  const seen = new Set<string>();
  while (current && byId.has(current) && !seen.has(current)) {
    seen.add(current);
    chain.push(current);
    current = byId.get(current)?.parent_id ?? null;
  }
  return chain;
}
