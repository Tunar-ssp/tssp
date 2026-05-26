/**
 * Workspace Editor Manager
 * Handles all editor-related state and operations
 */

import type { Workspace } from '$lib/api';

export interface EditorState {
  bodyDraft: string;
  nameDraft: string;
  selectedLanguage: string;
  isModified: boolean;
  cursorLine: number;
  cursorColumn: number;
  isDirty: boolean;
}

export class WorkspaceEditorManager {
  private state: EditorState = {
    bodyDraft: '',
    nameDraft: '',
    selectedLanguage: 'text',
    isModified: false,
    cursorLine: 1,
    cursorColumn: 1,
    isDirty: false,
  };

  private updateCallbacks: ((state: EditorState) => void)[] = [];
  private saveTimer: NodeJS.Timeout | null = null;
  private autoSaveDelay = 900;

  on(callback: (state: EditorState) => void) {
    this.updateCallbacks.push(callback);
    return () => {
      this.updateCallbacks = this.updateCallbacks.filter((cb) => cb !== callback);
    };
  }

  private notifyUpdate() {
    this.updateCallbacks.forEach((cb) => cb(this.state));
  }

  loadWorkspace(workspace: Workspace) {
    this.state = {
      bodyDraft: workspace.body,
      nameDraft: workspace.name,
      selectedLanguage: workspace.language || 'text',
      isModified: false,
      cursorLine: 1,
      cursorColumn: 1,
      isDirty: false,
    };
    this.notifyUpdate();
  }

  setContent(content: string) {
    this.state.bodyDraft = content;
    this.state.isModified = true;
    this.notifyUpdate();
  }

  setName(name: string) {
    this.state.nameDraft = name;
    this.state.isModified = true;
    this.notifyUpdate();
  }

  setLanguage(language: string) {
    this.state.selectedLanguage = language;
    this.state.isModified = true;
    this.notifyUpdate();
  }

  setCursor(line: number, column: number) {
    this.state.cursorLine = line;
    this.state.cursorColumn = column;
    this.notifyUpdate();
  }

  markSaved() {
    this.state.isModified = false;
    this.state.isDirty = false;
    this.notifyUpdate();
  }

  getState(): EditorState {
    return { ...this.state };
  }

  reset() {
    this.state = {
      bodyDraft: '',
      nameDraft: '',
      selectedLanguage: 'text',
      isModified: false,
      cursorLine: 1,
      cursorColumn: 1,
      isDirty: false,
    };
    this.notifyUpdate();
  }

  cleanup() {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
    }
  }
}
