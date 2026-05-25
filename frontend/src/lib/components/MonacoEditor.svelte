<script lang="ts">
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';

  interface Props {
    value: string;
    language?: string;
    onChange?: (value: string) => void;
    height?: string;
  }

  let { value = '', language = 'javascript', onChange, height = '100%' }: Props = $props();

  let container: HTMLDivElement;
  let editor: any;
  let monaco: any;
  let isInitialized = $state(false);

  const languages = [
    'javascript',
    'typescript',
    'python',
    'rust',
    'go',
    'java',
    'csharp',
    'cpp',
    'css',
    'html',
    'json',
    'yaml',
    'markdown',
    'sql',
    'bash',
    'php',
    'ruby',
  ];

  onMount(async () => {
    // Load Monaco dynamically
    const monacoModule = await import('monaco-editor');
    monaco = monacoModule.default;

    // Create editor instance
    editor = monaco.editor.create(container, {
      value,
      language,
      theme: 'vs-dark',
      automaticLayout: true,
      minimap: { enabled: true, side: 'right' },
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, Courier New, monospace',
      lineNumbers: 'on',
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      formatOnPaste: true,
      formatOnType: true,
      tabSize: 2,
      insertSpaces: true,
      useTabStops: true,
    });

    // Set theme based on system preference
    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    monaco.editor.setTheme(isDark ? 'vs-dark' : 'vs-light');

    // Listen for changes
    editor.onDidChangeModelContent(() => {
      onChange?.(editor.getValue());
    });

    isInitialized = true;

    return () => {
      editor?.dispose();
    };
  });

  function handleLanguageChange(e: any) {
    if (editor && monaco) {
      const newLanguage = e.target.value;
      const currentModel = editor.getModel();
      if (currentModel) {
        monaco.editor.setModelLanguage(currentModel, newLanguage);
      }
    }
  }

  function handleFormat() {
    if (editor) {
      editor.getAction('editor.action.formatDocument').run();
    }
  }

  function handleUndo() {
    if (editor) {
      editor.trigger('keyboard', 'undo', undefined);
    }
  }

  function handleRedo() {
    if (editor) {
      editor.trigger('keyboard', 'redo', undefined);
    }
  }
</script>

<div class="monaco-wrapper" style="height: {height}">
  <div class="editor-toolbar">
    <div class="toolbar-left">
      <select class="language-select" value={language} on:change={handleLanguageChange}>
        {#each languages as lang}
          <option value={lang}>{lang}</option>
        {/each}
      </select>
    </div>

    <div class="toolbar-right">
      <button class="toolbar-btn" title="Undo" on:click={handleUndo}>
        <Icons.Undo2 size={14} />
      </button>
      <button class="toolbar-btn" title="Redo" on:click={handleRedo}>
        <Icons.Redo2 size={14} />
      </button>
      <button class="toolbar-btn" title="Format (Alt+Shift+F)" on:click={handleFormat}>
        <Icons.Wand2 size={14} />
      </button>
    </div>
  </div>

  <div bind:this={container} class="editor-container"></div>
</div>

<style>
  .monaco-wrapper {
    display: flex;
    flex-direction: column;
    background: #1e1e1e;
    border-radius: var(--r-2);
    overflow: hidden;
    border: 1px solid var(--border);
  }

  .editor-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-2) var(--s-3);
    background: #2d2d2d;
    border-bottom: 1px solid var(--border);
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .language-select {
    padding: var(--s-1) var(--s-2);
    background: #3e3e3e;
    color: #fff;
    border: 1px solid #555;
    border-radius: var(--r-1);
    font-size: var(--fs-12);
    cursor: pointer;
  }

  .language-select:hover {
    background: #454545;
  }

  .toolbar-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--r-1);
    background: transparent;
    color: #888;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick);
  }

  .toolbar-btn:hover {
    background: #3e3e3e;
    color: #fff;
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
  }

  :global(.editor-container .monaco-editor) {
    color: #d4d4d4;
  }

  :global(.editor-container .monaco-editor .current-line) {
    background: rgba(255, 255, 255, 0.1);
  }
</style>
