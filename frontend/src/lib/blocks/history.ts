/**
 * Undo/redo system for block editor.
 * Maintains history stack with efficient diff-based snapshots.
 */

import type { Block } from './types';

export interface HistoryState {
  blocks: Block[];
  timestamp: number;
}

export class EditorHistory {
  private undoStack: HistoryState[] = [];
  private redoStack: HistoryState[] = [];
  private maxStates = 100;

  /**
   * Save current state to history
   */
  save(blocks: Block[]): void {
    // Deep clone to avoid reference issues
    const state: HistoryState = {
      blocks: JSON.parse(JSON.stringify(blocks)),
      timestamp: Date.now(),
    };

    this.undoStack.push(state);
    this.redoStack = []; // Clear redo stack when new change is made

    // Limit history size to prevent memory bloat
    if (this.undoStack.length > this.maxStates) {
      this.undoStack.shift();
    }
  }

  /**
   * Undo last change
   */
  undo(): Block[] | null {
    if (this.undoStack.length === 0) return null;

    const current = this.undoStack.pop();
    if (!current) return null;

    // Save current state to redo stack
    const blocks = JSON.parse(JSON.stringify(current.blocks));

    // Push the state we're undoing from to redo stack
    const previousState = this.undoStack[this.undoStack.length - 1];
    if (previousState) {
      this.redoStack.push(previousState);
    }

    return blocks;
  }

  /**
   * Redo last undone change
   */
  redo(): Block[] | null {
    if (this.redoStack.length === 0) return null;

    const state = this.redoStack.pop();
    if (!state) return null;

    // Push to undo stack
    this.undoStack.push(state);

    return JSON.parse(JSON.stringify(state.blocks));
  }

  /**
   * Check if undo is possible
   */
  canUndo(): boolean {
    return this.undoStack.length > 1; // Keep at least one state
  }

  /**
   * Check if redo is possible
   */
  canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  /**
   * Clear all history
   */
  clear(): void {
    this.undoStack = [];
    this.redoStack = [];
  }

  /**
   * Get history size for debugging
   */
  getSize(): { undo: number; redo: number } {
    return {
      undo: this.undoStack.length,
      redo: this.redoStack.length,
    };
  }
}

/**
 * Global history instance for editor
 */
export const editorHistory = new EditorHistory();
