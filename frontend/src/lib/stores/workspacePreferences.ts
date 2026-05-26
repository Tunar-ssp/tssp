import { writable, derived } from 'svelte/store';

export interface EditorPreferences {
  fontSize: number;
  tabSize: number;
  lineNumbers: boolean;
  wordWrap: boolean;
  minimap: boolean;
  theme: 'dark' | 'light';
  fontFamily: string;
}

export interface WorkspacePreferences {
  editor: EditorPreferences;
  showFileTree: boolean;
  showMinimap: boolean;
  autoSave: boolean;
  autoSaveDelay: number;
}

const defaultPreferences: WorkspacePreferences = {
  editor: {
    fontSize: 13,
    tabSize: 2,
    lineNumbers: true,
    wordWrap: true,
    minimap: false,
    theme: 'dark',
    fontFamily: 'var(--ff-mono)',
  },
  showFileTree: false,
  showMinimap: false,
  autoSave: true,
  autoSaveDelay: 900,
};

function createPreferencesStore() {
  const stored = localStorage.getItem('workspace-preferences');
  const initial = stored ? { ...defaultPreferences, ...JSON.parse(stored) } : defaultPreferences;

  const { subscribe, set, update } = writable<WorkspacePreferences>(initial);

  return {
    subscribe,
    updateEditor: (editor: Partial<EditorPreferences>) => {
      update((prefs) => {
        const updated = { ...prefs, editor: { ...prefs.editor, ...editor } };
        localStorage.setItem('workspace-preferences', JSON.stringify(updated));
        return updated;
      });
    },
    updatePreferences: (prefs: Partial<WorkspacePreferences>) => {
      update((current) => {
        const updated = { ...current, ...prefs };
        localStorage.setItem('workspace-preferences', JSON.stringify(updated));
        return updated;
      });
    },
    reset: () => {
      set(defaultPreferences);
      localStorage.removeItem('workspace-preferences');
    },
  };
}

export const workspacePreferences = createPreferencesStore();

export const editorFontSize = derived(workspacePreferences, ($prefs) => $prefs.editor.fontSize);
export const editorTabSize = derived(workspacePreferences, ($prefs) => $prefs.editor.tabSize);
export const editorLineNumbers = derived(workspacePreferences, ($prefs) => $prefs.editor.lineNumbers);
export const editorWordWrap = derived(workspacePreferences, ($prefs) => $prefs.editor.wordWrap);
export const editorMinimap = derived(workspacePreferences, ($prefs) => $prefs.editor.minimap);
