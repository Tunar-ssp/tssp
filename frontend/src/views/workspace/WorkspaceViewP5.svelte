<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { workspaces, activeWorkspace, loadWorkspaces, setActiveWorkspace, createNewWorkspace, deleteWorkspace, updateActiveWorkspace, isSaving } from '$lib/stores/workspace';
  import { success, error } from '$lib/stores/notifications';
  import TabBar from '$lib/components/TabBar.svelte';
  import FileExplorer from '$lib/components/FileExplorer.svelte';
  import FindWidget from '$lib/components/FindWidget.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { onMount } from 'svelte';

  let contextMenu = { visible: false, x: 0, y: 0, workspace: null as any };
  let isLoading = true;
  let bodyDraft = $state('');
  let nameDraft = $state('');
  let selectedLanguage = $state('');
  let showFindWidget = $state(false);
  let searchQuery = $state('');
  let cursorLine = $state(1);
  let cursorColumn = $state(1);
  let isModified = $state(false);
  let editorElement: HTMLTextAreaElement | null = null;

  let openTabs: any[] = $state([]);
  let activeTabId: string | null = $state(null);

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

  $effect(() => {
    if ($activeWorkspace) {
      nameDraft = $activeWorkspace.name;
      bodyDraft = $activeWorkspace.body;
      selectedLanguage = $activeWorkspace.language;

      if (activeTabId !== $activeWorkspace.id) {
        openTabs = openTabs.filter(t => t.id !== $activeWorkspace.id);
        openTabs = [{ id: $activeWorkspace.id, label: $activeWorkspace.name, isDirty: isModified, language: selectedLanguage }, ...openTabs];
        activeTabId = $activeWorkspace.id;
      }
    }
  });

  function handleSelectWorkspace(id: string) {
    setActiveWorkspace(id);
  }

  function handleTabSelect(id: string) {
    setActiveWorkspace(id);
    activeTabId = id;
  }

  function handleTabClose(id: string) {
    openTabs = openTabs.filter(t => t.id !== id);
    if (activeTabId === id) {
      const newActive = openTabs[0];
      if (newActive) {
        setActiveWorkspace(newActive.id);
        activeTabId = newActive.id;
      } else {
        setActiveWorkspace(null);
        activeTabId = null;
      }
    }
  }

  function handleEditorInput(newValue: string) {
    bodyDraft = newValue;
    isModified = true;
    updateTabs();
  }

  function updateTabs() {
    openTabs = openTabs.map(t =>
      t.id === activeTabId ? { ...t, isDirty: isModified } : t
    );
  }

  async function handleSaveWorkspace() {
    if (!$activeWorkspace) return;
    try {
      await updateActiveWorkspace({
        name: nameDraft,
        body: bodyDraft,
        language: selectedLanguage,
      });
      isModified = false;
      updateTabs();
      success('Workspace saved');
    } catch (err) {
      error('Failed to save workspace');
    }
  }

  async function handleChangeLanguage(lang: string) {
    selectedLanguage = lang;
    await updateActiveWorkspace({ language: lang });
    success(`Language changed to ${lang}`);
  }

  function handleEditorKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
      e.preventDefault();
      handleSaveWorkspace();
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      showFindWidget = !showFindWidget;
    }
  }

  function handleFind(query: string, options: any) {
    if (!editorElement) return;
    const text = bodyDraft.toLowerCase();
    const search = options.matchCase ? query : query.toLowerCase();
    const idx = text.indexOf(search);

    if (idx >= 0) {
      editorElement.focus();
      editorElement.setSelectionRange(idx, idx + query.length);
      updateCursorPosition();
    }
  }

  function handleEditorClick() {
    updateCursorPosition();
  }

  function updateCursorPosition() {
    if (!editorElement) return;
    const before = bodyDraft.substring(0, editorElement.selectionStart);
    cursorLine = before.split('\n').length;
    cursorColumn = before.split('\n').pop()?.length || 1;
  }

  function getWordCount(text: string) {
    return text.trim().split(/\s+/).filter(w => w.length > 0).length;
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

  async function handleDeleteWorkspace(workspace: any) {
    if (confirm(`Delete "${workspace.name}"?`)) {
      await deleteWorkspace(workspace.id);
      success('Workspace deleted');
    }
  }

  function getContextItems(workspace: any) {
    return [
      { label: 'Delete', action: () => handleDeleteWorkspace(workspace), danger: true },
    ];
  }

  async function handleCreateWorkspace() {
    try {
      await createNewWorkspace();
      success('New workspace created');
    } catch (err) {
      error('Failed to create workspace');
    }
  }

  let fileTree = $derived(
    [
      {
        id: 'root',
        name: 'root',
        type: 'folder',
        children: $workspaces.map(w => ({
          id: w.id,
          name: w.name + (languages.find(l => l.id === w.language)?.ext || ''),
          type: 'file',
          path: w.name,
        })),
      },
    ].filter(f => f.children.length > 0)
  );
</script>

<div class="workspace-view">
  <div class="sidebar">
    <FileExplorer
      files={fileTree}
      selectedFileId={activeTabId}
      onSelectFile={handleSelectWorkspace}
      onCreateFile={handleCreateWorkspace}
      onDeleteFile={() => {}}
    />
  </div>

  <div class="editor-area">
    {#if !$activeWorkspace}
      <div class="no-workspace">
        <Icons.Code size={48} />
        <h3>Select a workspace</h3>
        <p>Click a workspace to edit or create a new one</p>
      </div>
    {:else}
      <TabBar
        tabs={openTabs}
        activeTabId={activeTabId}
        onSelectTab={handleTabSelect}
        onCloseTab={handleTabClose}
      />

      <FindWidget
        isOpen={showFindWidget}
        onClose={() => showFindWidget = false}
        onFind={handleFind}
      />

      <div class="editor-main">
        <div class="editor-header">
          <input
            type="text"
            class="name-input"
            placeholder="Untitled workspace"
            bind:value={nameDraft}
            onchange={handleSaveWorkspace}
          />
          <div class="editor-actions">
            <select
              class="language-select"
              bind:value={selectedLanguage}
              onchange={(e) => handleChangeLanguage(e.currentTarget.value)}
            >
              {#each languages as lang}
                <option value={lang.id}>{lang.label}</option>
              {/each}
            </select>

            {#if $isSaving}
              <span class="saving">Saving...</span>
            {/if}

            <button class="save-btn" onclick={handleSaveWorkspace}>
              <Icons.Save size={14} />
              Save
            </button>
          </div>
        </div>

        <textarea
          bind:this={editorElement}
          class="code-editor"
          placeholder="Start coding..."
          bind:value={bodyDraft}
          onchange={handleSaveWorkspace}
          oninput={(e) => handleEditorInput(e.currentTarget.value)}
          onkeydown={handleEditorKeydown}
          onclick={handleEditorClick}
          onkeyup={updateCursorPosition}
        ></textarea>
      </div>

      <StatusBar
        language={selectedLanguage}
        lines={bodyDraft.split('\n').length}
        cursorLine={cursorLine}
        cursorColumn={cursorColumn}
        wordCount={getWordCount(bodyDraft)}
        charCount={bodyDraft.length}
        isDirty={isModified}
      />
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
    width: 260px;
    height: 100%;
    border-right: 1px solid var(--border);
    overflow: hidden;
  }

  .editor-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .no-workspace {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    color: var(--muted);
  }

  .no-workspace h3 {
    margin: 0;
    color: var(--text-2);
  }

  .no-workspace p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .editor-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .editor-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    background: var(--surface);
    flex-shrink: 0;
  }

  .name-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-16);
    font-weight: 600;
    outline: none;
  }

  .name-input::placeholder {
    color: var(--muted);
  }

  .editor-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .language-select {
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--bg);
    color: var(--text);
    font-size: var(--fs-12);
    cursor: pointer;
  }

  .saving {
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .save-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-12);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .save-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .code-editor {
    flex: 1;
    border: none;
    background: var(--bg);
    color: var(--text);
    padding: 16px;
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
</style>
