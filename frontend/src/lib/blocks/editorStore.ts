/**
 * Svelte store for managing block editor state.
 * Handles blocks, selection, autosave, and IndexedDB persistence.
 */

import { writable, derived, get } from 'svelte/store';
import type { Block, EditorState } from './types';
import { serializeBlocks, deserializeBlocks, blocksToMarkdown } from './utils';
import { editorHistory } from './history';

// IndexedDB setup
const DB_NAME = 'tssp_notes';
const STORE_NAME = 'drafts';

async function initDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, 1);

    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);

    request.onupgradeneeded = (e) => {
      const db = (e.target as IDBOpenDBRequest).result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: 'noteId' });
      }
    };
  });
}

async function saveDraftToIndexedDB(noteId: string, blocks: Block[]): Promise<void> {
  try {
    const db = await initDB();
    const tx = db.transaction(STORE_NAME, 'readwrite');
    const store = tx.objectStore(STORE_NAME);

    await new Promise<void>((resolve, reject) => {
      const request = store.put({
        noteId,
        blocks: serializeBlocks(blocks),
        savedAt: Date.now(),
      });
      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve();
    });
  } catch (error) {
    console.error('Failed to save draft to IndexedDB:', error);
  }
}

async function loadDraftFromIndexedDB(noteId: string): Promise<Block[] | null> {
  try {
    const db = await initDB();
    const tx = db.transaction(STORE_NAME, 'readonly');
    const store = tx.objectStore(STORE_NAME);

    return new Promise((resolve, reject) => {
      const request = store.get(noteId);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const result = request.result;
        if (result) {
          resolve(deserializeBlocks(result.blocks));
        } else {
          resolve(null);
        }
      };
    });
  } catch (error) {
    console.error('Failed to load draft from IndexedDB:', error);
    return null;
  }
}

async function clearDraftFromIndexedDB(noteId: string): Promise<void> {
  try {
    const db = await initDB();
    const tx = db.transaction(STORE_NAME, 'readwrite');
    const store = tx.objectStore(STORE_NAME);

    await new Promise<void>((resolve, reject) => {
      const request = store.delete(noteId);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve();
    });
  } catch (error) {
    console.error('Failed to clear draft from IndexedDB:', error);
  }
}

/**
 * Editor blocks store
 */
export const editorBlocks = writable<Block[]>([]);

/**
 * Current selection (block ID and cursor offset)
 */
export const editorSelection = writable<{ blockId: string; offset: number } | null>(null);

/**
 * Whether the editor has unsaved changes
 */
export const editorIsDirty = writable(false);

/**
 * Whether autosave is in progress
 */
export const editorIsSaving = writable(false);

/**
 * Active note ID for IndexedDB draft saving
 */
export const editorActiveNoteId = writable<string | null>(null);

/**
 * Whether undo is available
 */
export const canUndo = derived(
  editorBlocks,
  () => editorHistory.canUndo()
);

/**
 * Whether redo is available
 */
export const canRedo = derived(
  editorBlocks,
  () => editorHistory.canRedo()
);

/**
 * Get current editor state
 */
export const editorState = derived(
  [editorBlocks, editorSelection, editorIsDirty],
  ([$blocks, $selection, $isDirty]) => ({
    blocks: $blocks,
    selection: $selection,
    isDirty: $isDirty,
  } as EditorState)
);

/**
 * Convert blocks to markdown for saving
 */
export const editorMarkdown = derived(editorBlocks, ($blocks) => blocksToMarkdown($blocks));

/**
 * Initialize editor with blocks
 */
export function initializeEditor(blocks: Block[], noteId: string) {
  editorBlocks.set(blocks);
  editorSelection.set(null);
  editorIsDirty.set(false);
  editorActiveNoteId.set(noteId);
  editorHistory.clear();
  editorHistory.save(blocks);
}

/**
 * Load draft from IndexedDB if available
 */
export async function loadEditorDraft(noteId: string): Promise<Block[] | null> {
  const draft = await loadDraftFromIndexedDB(noteId);
  return draft;
}

/**
 * Update blocks and mark as dirty
 */
export function updateBlocks(blocks: Block[]) {
  editorBlocks.set(blocks);
  editorIsDirty.set(true);
}

/**
 * Update a single block
 */
export function updateBlock(blockId: string, updates: Partial<Block>) {
  const currentBlocks = get(editorBlocks);
  editorHistory.save(currentBlocks);

  editorBlocks.update((blocks) => {
    const updateRecursive = (blocks: Block[]): Block[] => {
      return blocks.map((block) => {
        if (block.id === blockId) {
          return { ...block, ...updates };
        }
        if (block.children) {
          return {
            ...block,
            children: updateRecursive(block.children),
          };
        }
        return block;
      });
    };

    return updateRecursive(blocks);
  });

  editorIsDirty.set(true);
}

/**
 * Insert a block at a specific position
 */
export function insertBlock(newBlock: Block, afterBlockId?: string) {
  const currentBlocks = get(editorBlocks);
  editorHistory.save(currentBlocks);

  editorBlocks.update((blocks) => {
    if (!afterBlockId) {
      return [...blocks, newBlock];
    }

    const insertRecursive = (blocks: Block[]): Block[] => {
      const result: Block[] = [];
      for (const block of blocks) {
        result.push(block);
        if (block.id === afterBlockId) {
          result.push(newBlock);
        } else if (block.children) {
          block.children = insertRecursive(block.children);
        }
      }
      return result;
    };

    return insertRecursive(blocks);
  });

  editorSelection.set({ blockId: newBlock.id, offset: 0 });
  editorIsDirty.set(true);
}

/**
 * Delete a block
 */
export function deleteBlock(blockId: string) {
  const currentBlocks = get(editorBlocks);
  editorHistory.save(currentBlocks);

  editorBlocks.update((blocks) => {
    const deleteRecursive = (blocks: Block[]): Block[] => {
      return blocks
        .filter((block) => block.id !== blockId)
        .map((block) => ({
          ...block,
          children: block.children ? deleteRecursive(block.children) : undefined,
        }));
    };

    return deleteRecursive(blocks);
  });

  editorSelection.set(null);
  editorIsDirty.set(true);
}

/**
 * Set cursor selection
 */
export function setSelection(blockId: string, offset: number) {
  editorSelection.set({ blockId, offset });
}

/**
 * Autosave handler with debounce (800ms)
 */
let autosaveTimeout: ReturnType<typeof setTimeout>;

export function scheduleAutosave() {
  clearTimeout(autosaveTimeout);

  autosaveTimeout = setTimeout(async () => {
    const noteId = get(editorActiveNoteId);
    const blocks = get(editorBlocks);
    const isDirty = get(editorIsDirty);

    if (!isDirty || !noteId) return;

    editorIsSaving.set(true);
    try {
      await saveDraftToIndexedDB(noteId, blocks);
    } finally {
      editorIsSaving.set(false);
    }
  }, 800);
}

/**
 * Clear draft after successful save to backend
 */
export async function clearEditorDraft() {
  const noteId = get(editorActiveNoteId);
  if (noteId) {
    await clearDraftFromIndexedDB(noteId);
  }
  editorIsDirty.set(false);
}

/**
 * Undo last change
 */
export function undo() {
  const previousBlocks = editorHistory.undo();
  if (previousBlocks) {
    editorBlocks.set(previousBlocks);
    editorSelection.set(null);
    editorIsDirty.set(true);
  }
}

/**
 * Redo last undone change
 */
export function redo() {
  const nextBlocks = editorHistory.redo();
  if (nextBlocks) {
    editorBlocks.set(nextBlocks);
    editorSelection.set(null);
    editorIsDirty.set(true);
  }
}

/**
 * Reset editor state
 */
export function resetEditor() {
  editorBlocks.set([]);
  editorSelection.set(null);
  editorIsDirty.set(false);
  editorIsSaving.set(false);
  editorActiveNoteId.set(null);
  editorHistory.clear();
  clearTimeout(autosaveTimeout);
}
