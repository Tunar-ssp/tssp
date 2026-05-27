<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Workspace } from '$lib/api';
  import { isSaving, updateActiveWorkspace, deleteWorkspace } from '$lib/stores/workspace';

  export let workspace: Workspace | null;

  let name = '';
  let language = 'txt';
  let body = '';

  let saveTimeout: number;

  $: if (workspace) {
    name = workspace.name;
    language = workspace.language;
    body = workspace.body;
  }

  function handleNameChange(e: Event) {
    const target = e.target as HTMLInputElement;
    name = target.value;
    scheduleAutoSave();
  }

  function handleBodyChange(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    body = target.value;
    scheduleAutoSave();
  }

  function scheduleAutoSave() {
    clearTimeout(saveTimeout);
    saveTimeout = window.setTimeout(() => {
      if (workspace) {
        updateActiveWorkspace({
          name,
          body,
        }).catch(console.error);
      }
    }, 1500);
  }

  function handleDelete() {
    if (workspace && confirm(`Delete "${workspace.name}"?`)) {
      deleteWorkspace(workspace.id).catch(console.error);
    }
  }

  const languages = [
    'txt', 'js', 'ts', 'py', 'rs', 'go', 'java', 'c', 'cpp',
    'html', 'css', 'json', 'yaml', 'toml', 'xml', 'md', 'sh', 'sql'
  ];
</script>

{#if workspace}
  <div class="editor-container">
    <div class="editor-header">
      <input
        type="text"
        class="name-input"
        value={name}
        on:change={handleNameChange}
        placeholder="Workspace name"
      />

      <div class="language-select">
        <select bind:value={language} on:change={() => scheduleAutoSave()}>
          {#each languages as lang}
            <option value={lang}>{lang}</option>
          {/each}
        </select>
      </div>

      <div class="header-right">
        {#if $isSaving}
          <div class="saving">
            <div class="spinner"></div>
            Saving
          </div>
        {:else}
          <span class="saved">Saved</span>
        {/if}

        <button class="delete-btn" on:click={handleDelete}>
          <Icons.Trash2 size={16} />
        </button>
      </div>
    </div>

    <textarea
      class="code-input"
      value={body}
      on:input={handleBodyChange}
      spellcheck="false"
    ></textarea>
  </div>
{:else}
  <div class="empty-state">
    <Icons.Code2 size={40} />
    <p>No workspace selected</p>
    <p class="secondary">Create a new workspace or select one from the list</p>
  </div>
{/if}

<style>
  .editor-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg);
  }

  .editor-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--surface);
  }

  .name-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: var(--fs-14);
    font-weight: 500;
    color: var(--text);
    outline: none;
  }

  .name-input::placeholder {
    color: var(--muted);
  }

  .language-select {
    flex-shrink: 0;
  }

  .language-select select {
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
    color: var(--text-2);
    font-size: var(--fs-12);
    cursor: pointer;
    outline: none;
  }

  .language-select select:focus {
    border-color: var(--blue);
    color: var(--text);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .saving {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 1px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .saved {
    font-size: var(--fs-12);
    color: var(--green);
  }

  .delete-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .delete-btn:hover {
    background: transparent;
    border-color: var(--danger);
    color: var(--danger);
  }

  .code-input {
    flex: 1;
    padding: 16px;
    border: none;
    background: transparent;
    color: var(--text);
    font-family: var(--ff-mono);
    font-size: var(--fs-14);
    line-height: 1.6;
    outline: none;
    resize: none;
    tab-size: 2;
  }

  .code-input::placeholder {
    color: var(--muted);
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .empty-state p {
    margin: 0;
  }

  .empty-state .secondary {
    font-size: var(--fs-12);
    color: var(--dim);
  }
</style>
