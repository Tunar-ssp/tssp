<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Workspace } from '$lib/api';
  import MonacoEditor from '$lib/components/MonacoEditor.svelte';

  interface $$Props {
    workspace: Workspace | null;
    nameDraft: string;
    bodyDraft: string;
    selectedLanguage: string;
    cursorLine?: number;
    cursorColumn?: number;
    isModified?: boolean;
    isSaving?: boolean;
    onNameChange?: (name: string) => void;
    onBodyChange?: (body: string) => void;
    onLanguageChange?: (language: string) => void;
  }

  let {
    workspace,
    nameDraft = '',
    bodyDraft = '',
    selectedLanguage = 'text',
    cursorLine = 1,
    cursorColumn = 1,
    isModified = false,
    isSaving = false,
    onNameChange,
    onBodyChange,
    onLanguageChange,
  }: $$Props = $props();
</script>

<div class="workspace-editor">
  {#if workspace}
    <div class="editor-header">
      <input
        type="text"
        value={nameDraft}
        oninput={(e) => onNameChange?.((e.target as HTMLInputElement).value)}
        onchange={(e) => onNameChange?.((e.target as HTMLInputElement).value)}
        class="editor-name"
        placeholder="Workspace name..."
      />
      <div class="editor-status">
        {#if isSaving}
          <span class="status-saving">
            <Icons.Loader2 size={14} class="spin" />
            Saving
          </span>
        {:else if isModified}
          <span class="status-modified">
            <Icons.Circle size={8} />
            Modified
          </span>
        {:else}
          <span class="status-saved">
            <Icons.CheckCircle2 size={14} />
            Saved
          </span>
        {/if}
        <span class="cursor-position">{cursorLine}:{cursorColumn}</span>
      </div>
    </div>

    <div class="editor-body">
      <MonacoEditor
        value={bodyDraft}
        language={selectedLanguage}
        onChange={onBodyChange}
      />
    </div>
  {/if}
</div>

<style>
  .workspace-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: var(--surface);
  }

  .editor-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
  }

  .editor-name {
    flex: 1;
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
    background: transparent;
    border: none;
    outline: none;
    padding: 0;
  }

  .editor-name::placeholder {
    color: var(--muted);
  }

  .editor-status {
    display: inline-flex;
    align-items: center;
    gap: 12px;
    font-size: 12px;
    color: var(--muted);
  }

  .status-saving,
  .status-modified,
  .status-saved {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .status-saving {
    color: var(--warning);
  }

  .status-saved {
    color: var(--green);
  }

  .status-modified {
    color: var(--orange);
  }

  .cursor-position {
    font-family: var(--ff-mono);
    color: var(--muted);
  }

  .editor-body {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
