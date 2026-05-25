<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { workspaces, activeWorkspace, loadWorkspaces, setActiveWorkspace, createNewWorkspace, deleteWorkspace, updateActiveWorkspace, isSaving } from '$lib/stores/workspace';
  import { success, error } from '$lib/stores/notifications';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { onMount } from 'svelte';

  let contextMenu = { visible: false, x: 0, y: 0, workspace: null as any };
  let isLoading = true;
  let searchQuery = '';
  let bodyDraft = '';
  let nameDraft = '';
  let selectedLanguage = '';
  let showLanguageSelect = false;

  const languages = [
    { id: 'javascript', label: 'JavaScript', ext: '.js' },
    { id: 'typescript', label: 'TypeScript', ext: '.ts' },
    { id: 'python', label: 'Python', ext: '.py' },
    { id: 'rust', label: 'Rust', ext: '.rs' },
    { id: 'go', label: 'Go', ext: '.go' },
    { id: 'markdown', label: 'Markdown', ext: '.md' },
    { id: 'html', label: 'HTML', ext: '.html' },
    { id: 'css', label: 'CSS', ext: '.css' },
    { id: 'sql', label: 'SQL', ext: '.sql' },
    { id: 'json', label: 'JSON', ext: '.json' },
    { id: 'yaml', label: 'YAML', ext: '.yaml' },
    { id: 'bash', label: 'Bash', ext: '.sh' },
    { id: 'text', label: 'Plain Text', ext: '.txt' },
  ];

  onMount(async () => {
    await loadWorkspaces();
    isLoading = false;
  });

  $: {
    if ($activeWorkspace) {
      nameDraft = $activeWorkspace.name;
      bodyDraft = $activeWorkspace.body;
      selectedLanguage = $activeWorkspace.language;
    }
  }

  function handleCreateWorkspace() {
    createNewWorkspace();
  }

  function handleSelectWorkspace(id: string) {
    setActiveWorkspace(id);
  }

  function showContextMenu(event: MouseEvent, workspace: any) {
    event.preventDefault();
    contextMenu = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      workspace,
    };
  }

  async function handleSaveWorkspace() {
    if (!$activeWorkspace) return;
    await updateActiveWorkspace({
      name: nameDraft,
      body: bodyDraft,
      language: selectedLanguage,
    });
    success('Workspace saved');
  }

  async function handleChangeLanguage(lang: string) {
    selectedLanguage = lang;
    await updateActiveWorkspace({ language: lang });
    showLanguageSelect = false;
    success(`Language changed to ${lang}`);
  }

  async function handleDeleteWorkspace(workspace: any) {
    if (confirm(`Delete "${workspace.name}"?`)) {
      await deleteWorkspace(workspace.id);
      success('Workspace deleted');
    }
  }

  async function handleDuplicate(workspace: any) {
    try {
      const dup = await createNewWorkspace();
      await updateActiveWorkspace({
        name: `${workspace.name} (copy)`,
        body: workspace.body,
        language: workspace.language,
      });
      success('Workspace duplicated');
    } catch (err) {
      error('Failed to duplicate workspace');
    }
  }

  $: filteredWorkspaces = searchQuery
    ? $workspaces.filter(w =>
        w.name.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : $workspaces;

  function getContextItems(workspace: any) {
    return [
      { label: 'Duplicate', action: () => handleDuplicate(workspace) },
      { label: 'Delete', action: () => handleDeleteWorkspace(workspace), danger: true },
    ];
  }
</script>

<div class="workspace-view">
  <div class="sidebar">
    <div class="header">
      <h2>Workspaces</h2>
      <button class="create-btn" on:click={handleCreateWorkspace}>
        <Icons.Plus size={16} />
        New
      </button>
    </div>

    <div class="search-bar">
      <Icons.Search size={16} />
      <input
        type="text"
        placeholder="Search workspaces..."
        bind:value={searchQuery}
      />
    </div>

    <div class="workspaces-list">
      {#if isLoading}
        <div class="loading">
          <div class="spinner" />
          Loading...
        </div>
      {:else if filteredWorkspaces.length === 0}
        <div class="empty">
          <Icons.Code size={40} />
          <h3>No workspaces</h3>
          <p>Create a new workspace to get started</p>
        </div>
      {:else}
        {#each filteredWorkspaces as workspace (workspace.id)}
          <div
            class="workspace-row"
            class:active={$activeWorkspace?.id === workspace.id}
            on:click={() => handleSelectWorkspace(workspace.id)}
            on:contextmenu={(e) => showContextMenu(e, workspace)}
          >
            <div class="workspace-icon">
              <Icons.Code size={14} />
            </div>
            <div class="workspace-content">
              <div class="workspace-name">{workspace.name}</div>
              <div class="workspace-lang">{workspace.language}</div>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <div class="editor-area">
    {#if !$activeWorkspace}
      <div class="empty-editor">
        <Icons.Code size={48} />
        <h3>Select a workspace</h3>
        <p>Click a workspace to edit or create a new one</p>
      </div>
    {:else}
      <div class="editor-header">
        <div class="header-left">
          <input
            type="text"
            class="name-input"
            placeholder="Untitled workspace"
            bind:value={nameDraft}
            on:change={handleSaveWorkspace}
          />
        </div>

        <div class="header-right">
          <div class="language-selector">
            {#if showLanguageSelect}
              <div class="language-menu">
                {#each languages as lang}
                  <button
                    class="lang-item"
                    class:selected={selectedLanguage === lang.id}
                    on:click={() => handleChangeLanguage(lang.id)}
                  >
                    {lang.label}
                  </button>
                {/each}
              </div>
            {:else}
              <button
                class="lang-button"
                on:click={() => showLanguageSelect = true}
              >
                {languages.find(l => l.id === selectedLanguage)?.label || 'Select language'}
              </button>
            {/if}
          </div>

          {#if $isSaving}
            <span class="saving">Saving...</span>
          {/if}

          <button class="action-btn" on:click={handleSaveWorkspace}>
            <Icons.Save size={14} />
            Save
          </button>

          <button
            class="action-btn"
            on:click={(e) => showContextMenu(e, $activeWorkspace)}
          >
            <Icons.MoreVertical size={14} />
          </button>
        </div>
      </div>

      <div class="editor-content">
        <textarea
          class="code-editor"
          placeholder="Start coding..."
          bind:value={bodyDraft}
          on:change={handleSaveWorkspace}
        ></textarea>
      </div>

      <div class="editor-footer">
        <div class="stats">
          <span>Lines: {bodyDraft.split('\n').length}</span>
          <span>Characters: {bodyDraft.length}</span>
          <span>Language: {selectedLanguage}</span>
        </div>
      </div>
    {/if}
  </div>
</div>

<ContextMenu
  bind:visible={contextMenu.visible}
  x={contextMenu.x}
  y={contextMenu.y}
  items={contextMenu.workspace ? getContextItems(contextMenu.workspace) : []}
/>

<style>
  .workspace-view {
    flex: 1;
    display: flex;
    overflow: hidden;
    background: var(--bg);
  }

  .sidebar {
    flex-shrink: 0;
    width: 240px;
    height: 100%;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header {
    padding: 20px 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--surface);
  }

  .header h2 {
    margin: 0;
    font-size: var(--fs-20);
    color: var(--text);
  }

  .create-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: var(--r-2);
    border: 1px solid var(--border);
    background: var(--blue);
    color: #0a1228;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s;
    font-size: var(--fs-12);
  }

  .create-btn:hover {
    opacity: 0.9;
  }

  .search-bar {
    padding: 12px 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--surface);
    color: var(--muted);
  }

  .search-bar input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-13);
    outline: none;
  }

  .search-bar input::placeholder {
    color: var(--muted);
  }

  .workspaces-list {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
  }

  .workspace-row {
    padding: 12px;
    border-bottom: 1px solid var(--hairline);
    cursor: pointer;
    transition: background 0.15s;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .workspace-row:hover {
    background: var(--surface-2);
  }

  .workspace-row.active {
    background: var(--surface-3);
  }

  .workspace-icon {
    flex-shrink: 0;
    color: var(--blue);
  }

  .workspace-content {
    flex: 1;
    min-width: 0;
  }

  .workspace-name {
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: var(--fs-13);
  }

  .workspace-lang {
    font-size: 11px;
    color: var(--muted);
    margin-top: 2px;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .empty h3 {
    margin: 0;
    color: var(--text-2);
  }

  .empty p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .editor-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .empty-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .empty-editor h3 {
    margin: 0;
    color: var(--text-2);
  }

  .empty-editor p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .editor-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: var(--surface);
  }

  .header-left {
    flex: 1;
  }

  .name-input {
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-18);
    font-weight: 600;
    outline: none;
    max-width: 400px;
  }

  .name-input::placeholder {
    color: var(--muted);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .language-selector {
    position: relative;
  }

  .lang-button {
    padding: 6px 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    font-size: var(--fs-12);
    transition: all 0.15s;
  }

  .lang-button:hover {
    background: var(--surface-2);
  }

  .language-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 8px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    overflow: hidden;
    display: grid;
    max-height: 300px;
    overflow-y: auto;
    z-index: 100;
    min-width: 160px;
  }

  .lang-item {
    padding: 8px 12px;
    border: none;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    font-size: var(--fs-12);
    text-align: left;
    transition: background 0.15s;
  }

  .lang-item:hover {
    background: var(--surface-3);
  }

  .lang-item.selected {
    background: var(--blue-subtle);
    color: var(--blue);
  }

  .saving {
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .action-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .action-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .editor-content {
    flex: 1;
    overflow: hidden;
  }

  .code-editor {
    width: 100%;
    height: 100%;
    border: none;
    background: var(--bg);
    color: var(--text);
    padding: 20px;
    font-family: var(--ff-mono);
    font-size: var(--fs-13);
    line-height: 1.6;
    outline: none;
    resize: none;
    tab-size: 2;
  }

  .code-editor::placeholder {
    color: var(--muted);
  }

  .editor-footer {
    padding: 8px 20px;
    border-top: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .stats {
    display: flex;
    gap: 20px;
  }
</style>
