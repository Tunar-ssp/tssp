<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import {
    workspaces,
    activeWorkspace,
    loadWorkspaces,
    setActiveWorkspace,
    createNewWorkspace,
    deleteWorkspace,
    updateActiveWorkspace,
    isSaving,
  } from '$lib/stores/workspace';
  import { success, error } from '$lib/stores/notifications';
  import TabBar from '$lib/components/TabBar.svelte';
  import FindReplaceWidget from '$lib/components/FindReplaceWidget.svelte';
  import MarkdownPreview from '$lib/components/MarkdownPreview.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { consumeSelectionIntent, navigateTo } from '$lib/stores/ui';
  import { renderMarkdownLite } from '$lib/utils/markdown';
  import { formatRelative, getWordCount, registerKeyboardShortcuts } from '$lib/utils';
  import { getWorkspaceCapabilities } from '$lib/services/workspaceService';
  import type { WorkspaceCapabilities } from '$lib/api';
  import { findMatches, replaceMatches } from '$lib/services/workspaceSearchService';
  import type { SearchOptions } from '$lib/services/workspaceSearchService';
  import WorkspaceSidebar from './WorkspaceSidebar.svelte';
  import WorkspaceEditorHeader from './components/editors/WorkspaceEditorHeader.svelte';
  import WorkspaceInspector from './WorkspaceInspector.svelte';
  import WorkspaceStageHead from './WorkspaceStageHead.svelte';
  import WorkspaceHomepage from './WorkspaceHomepage.svelte';

  // Lazy load Monaco Editor to reduce initial bundle size
  let MonacoEditor = $state<any>(null);
  let monacoLoading = $state(false);

  type InspectorTab = 'preview' | 'outline' | 'terminal';

  let showSidebar = $state(true);
  let showBottomPanel = $state(false);
  let contextMenu = $state({ visible: false, x: 0, y: 0, workspace: null as any });
  let isLoading = $state(true);
  let bodyDraft = $state('');
  let nameDraft = $state('');
  let selectedLanguage = $state('');
  let workspaceFilterQuery = $state('');
  let showFindWidget = $state(false);
  let inspectorTab = $state<InspectorTab>('terminal');
  let cursorLine = $state(1);
  let cursorColumn = $state(1);
  let isModified = $state(false);
  let capabilities = $state<WorkspaceCapabilities | null>(null);
  let matchCount = $state(0);
  let currentMatchIndex = $state(0);

  let openTabs: Array<{ id: string; label: string; isDirty?: boolean; language?: string }> = $state([]);
  let activeTabId: string | null = $state(null);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

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
    const intent = consumeSelectionIntent();
    if (intent?.kind === 'workspace') {
      setActiveWorkspace(intent.id);
      activeTabId = intent.id;
    }
    isLoading = false;
  });

  const handleWorkspaceKeydown = (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'b') {
      showSidebar = !showSidebar;
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
      showFindWidget = !showFindWidget;
    }
  };

  $effect(() => {
    if (typeof document === 'undefined') return;
    const cleanup = registerKeyboardShortcuts(
      [
        { key: 'b', ctrl: true, handler: handleWorkspaceKeydown },
        { key: 'f', ctrl: true, handler: handleWorkspaceKeydown },
      ],
      document
    );
    return cleanup;
  });

  onDestroy(() => {
    if (saveTimer) clearTimeout(saveTimer);
  });

  let lastActiveId = $state<string | null>(null);

  $effect(() => {
    if ($activeWorkspace) {
      if ($activeWorkspace.id !== lastActiveId) {
        nameDraft = $activeWorkspace.name;
        bodyDraft = $activeWorkspace.body;
        selectedLanguage = $activeWorkspace.language || 'text';
        isModified = false;
        syncOpenTabs();
        activeTabId = $activeWorkspace.id;
        loadCapabilities($activeWorkspace.id);
        lastActiveId = $activeWorkspace.id;
      }
    } else {
      nameDraft = '';
      bodyDraft = '';
      selectedLanguage = 'text';
      isModified = false;
      capabilities = null;
      lastActiveId = null;
    }
  });

  async function loadCapabilities(workspaceId: string) {
    try {
      const caps = await getWorkspaceCapabilities(workspaceId);
      capabilities = caps;
    } catch (err) {
      console.error('Failed to load workspace capabilities:', err);
      capabilities = null;
    }
  }

  let filteredWorkspaces = $derived.by(() =>
    $workspaces.filter((workspace) => {
      if (!workspaceFilterQuery.trim()) return true;
      const query = workspaceFilterQuery.toLowerCase();
      return (
        workspace.name.toLowerCase().includes(query) ||
        workspace.language.toLowerCase().includes(query) ||
        workspace.body.toLowerCase().includes(query)
      );
    })
  );

  let previewHtml = $derived(renderMarkdownLite(bodyDraft));
  let languageCount = $derived(new Set($workspaces.map((workspace) => workspace.language)).size);
  let recentWorkspaces = $derived(filteredWorkspaces.slice(0, 6));
  let isMarkdownFile = $derived(selectedLanguage === 'markdown');

  async function loadMonacoEditor() {
    if (MonacoEditor) return;
    if (monacoLoading) return;
    monacoLoading = true;
    try {
      const module = await import('$lib/components/MonacoEditor.svelte');
      MonacoEditor = module.default;
    } catch (err) {
      console.error('Failed to load Monaco Editor:', err);
      error('Editor', 'Could not load code editor');
    } finally {
      monacoLoading = false;
    }
  }

  $effect(() => {
    if (!isMarkdownFile && $activeWorkspace?.id) {
      void loadMonacoEditor();
    }
  });

  function syncOpenTabs() {
    if (!$activeWorkspace) return;
    const nextTab = {
      id: $activeWorkspace.id,
      label: nameDraft || $activeWorkspace.name || 'untitled',
      isDirty: isModified,
      language: selectedLanguage,
    };

    const existingIndex = openTabs.findIndex((tab) => tab.id === nextTab.id);
    if (existingIndex >= 0) {
      openTabs = openTabs.map((tab) => (tab.id === nextTab.id ? nextTab : tab));
    } else {
      openTabs = [nextTab, ...openTabs];
    }
  }

  function handleSelectWorkspace(id: string) {
    setActiveWorkspace(id);
    activeTabId = id;
  }

  function handleTabSelect(id: string) {
    setActiveWorkspace(id);
    activeTabId = id;
  }

  function handleTabClose(id: string) {
    const remaining = openTabs.filter((tab) => tab.id !== id);
    openTabs = remaining;

    if (activeTabId === id) {
      const nextTab = remaining[0];
      if (nextTab) {
        setActiveWorkspace(nextTab.id);
        activeTabId = nextTab.id;
      } else {
        setActiveWorkspace(null);
        activeTabId = null;
      }
    }
  }

  function scheduleWorkspaceSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      void handleSaveWorkspace(false);
    }, 900);
  }

  function handleEditorInput(newValue: string) {
    bodyDraft = newValue;
    isModified = true;
    syncOpenTabs();
    scheduleWorkspaceSave();
  }

  async function handleSaveWorkspace(showToast = true) {
    if (!$activeWorkspace) return;
    try {
      await updateActiveWorkspace({
        name: nameDraft,
        body: bodyDraft,
        language: selectedLanguage,
      });
      isModified = false;
      syncOpenTabs();
      if (showToast) success('Workspace Saved', 'Changes were written to TSSP');
    } catch (err) {
      error('Save Failed', err instanceof Error ? err.message : 'Failed to save workspace');
    }
  }

  async function handleChangeLanguage(lang: string) {
    selectedLanguage = lang;
    isModified = true;
    syncOpenTabs();
    await handleSaveWorkspace(false);
    success('Language Updated', `Workspace language is now ${lang}`);
  }

  function handleEditorKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
      event.preventDefault();
      void handleSaveWorkspace();
    }
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'f') {
      event.preventDefault();
      showFindWidget = !showFindWidget;
    }
  }

  function handleFind(query: string, options: SearchOptions) {
    if (!query.trim()) return;
    const matches = findMatches(bodyDraft, query, options);
    matchCount = matches.length;
    currentMatchIndex = 0;

    if (matches.length > 0) {
      const match = matches[0];
      updateCursorPositionFromOffset(match.matchStart);
      success('Match Found', `${matches.length} matches found, viewing 1/${matchCount}`);
    } else {
      error('No Match', `"${query}" was not found`);
    }
  }

  function handleReplace(query: string, replacement: string, options: SearchOptions) {
    if (!query.trim()) return;
    const result = replaceMatches(bodyDraft, query, replacement, options);
    bodyDraft = result.content;
    isModified = true;
    syncOpenTabs();
    scheduleWorkspaceSave();

    if (result.replacementCount > 0) {
      success('Replaced', `${result.replacementCount} match${result.replacementCount === 1 ? '' : 'es'} replaced`);
      matchCount = 0;
      currentMatchIndex = 0;
    } else {
      error('No Match', `"${query}" was not found`);
    }
  }

  function handleReplaceAll(query: string, replacement: string, options: SearchOptions) {
    if (!query.trim()) return;
    const result = replaceMatches(bodyDraft, query, replacement, options);
    bodyDraft = result.content;
    isModified = true;
    syncOpenTabs();
    scheduleWorkspaceSave();

    if (result.replacementCount > 0) {
      success('Replaced All', `${result.replacementCount} match${result.replacementCount === 1 ? '' : 'es'} replaced`);
      matchCount = 0;
      currentMatchIndex = 0;
    } else {
      error('No Match', `"${query}" was not found`);
    }
  }

  function updateCursorPositionFromOffset(offset: number) {
    const before = bodyDraft.slice(0, offset);
    cursorLine = before.split('\n').length;
    cursorColumn = before.split('\n').pop()?.length || 1;
  }

  async function handleDeleteWorkspace(workspace: any) {
    if (!confirm(`Delete "${workspace.name}"?`)) return;
    try {
      await deleteWorkspace(workspace.id);
      openTabs = openTabs.filter((tab) => tab.id !== workspace.id);
      success('Workspace Deleted', 'The workspace was removed');
    } catch (err) {
      error('Delete Failed', err instanceof Error ? err.message : 'Failed to delete workspace');
    }
  }

  async function handleCreateWorkspace() {
    try {
      const workspace = await createNewWorkspace();
      setActiveWorkspace(workspace.id);
      activeTabId = workspace.id;
      success('Workspace Created', 'A new workspace is ready');
    } catch (err) {
      error('Create Failed', err instanceof Error ? err.message : 'Failed to create workspace');
    }
  }

  function showContextMenu(event: MouseEvent, workspace: any) {
    event.preventDefault();
    event.stopPropagation();
    contextMenu = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      workspace,
    };
  }

  function getContextItems(workspace: any) {
    return [
      {
        label: 'Open',
        action: () => handleSelectWorkspace(workspace.id),
      },
      {
        label: 'Delete',
        action: () => handleDeleteWorkspace(workspace),
        danger: true,
      },
    ];
  }

  function languageLabel(language: string) {
    return languages.find((item) => item.id === language)?.label || language || 'Plain Text';
  }
</script>

<svelte:window onkeydown={handleEditorKeydown} />

<div class:editor-mode={!!$activeWorkspace} class:home-mode={!$activeWorkspace} class:sidebar-hidden={!showSidebar} class:bottom-panel-open={showBottomPanel} class="workspace-view">
  <aside class="workspace-activity">
    <button type="button" class="activity-btn" class:active={showSidebar} title="Explorer" onclick={() => showSidebar = !showSidebar}>
      <Icons.Files size={20} />
    </button>
    <button type="button" class="activity-btn" title="Find" onclick={() => (showFindWidget = true)}>
      <Icons.Search size={20} />
    </button>
    <button type="button" class="activity-btn" class:active={showBottomPanel && inspectorTab === 'terminal'} title="Terminal" onclick={() => { showBottomPanel = true; inspectorTab = 'terminal'; }}>
      <Icons.TerminalSquare size={20} />
    </button>
    <div class="activity-spacer"></div>
    <button type="button" class="activity-btn" title="Back to Launcher" onclick={() => navigateTo('home')}>
      <Icons.LayoutGrid size={20} />
    </button>
  </aside>

  {#if showSidebar}
    <WorkspaceSidebar
      workspaces={filteredWorkspaces}
      filterQuery={workspaceFilterQuery}
      activeWorkspaceId={$activeWorkspace?.id ?? null}
      {languageCount}
      onFilterChange={(q) => (workspaceFilterQuery = q)}
      onSelectWorkspace={handleSelectWorkspace}
      onShowContextMenu={showContextMenu}
      onCreateWorkspace={handleCreateWorkspace}
    />
  {/if}

  {#if !$activeWorkspace}
    <WorkspaceHomepage
      workspaces={$workspaces}
      {languageCount}
      {recentWorkspaces}
      onCreateWorkspace={handleCreateWorkspace}
      onSelectWorkspace={handleSelectWorkspace}
    />
  {:else}
    <section class="workspace-stage">
      <div class="stage-main">
        <div class="editor-column">
          <TabBar
            tabs={openTabs}
            activeTabId={activeTabId}
            onSelectTab={handleTabSelect}
            onCloseTab={handleTabClose}
          />

          <WorkspaceEditorHeader
            name={nameDraft}
            selectedLanguage={selectedLanguage}
            isSaving={$isSaving}
            {languages}
            onNameChange={(value) => {
              nameDraft = value;
              isModified = true;
              syncOpenTabs();
              scheduleWorkspaceSave();
            }}
            onLanguageChange={handleChangeLanguage}
          />

          <div class="editor-shell">
            {#if isMarkdownFile}
              <MarkdownPreview
                content={bodyDraft}
                showPreview={true}
                onTogglePreview={() => {}}
                onChange={handleEditorInput}
              />
            {:else if MonacoEditor && !monacoLoading}
              <svelte:component
                this={MonacoEditor}
                value={bodyDraft}
                language={selectedLanguage}
                onChange={handleEditorInput}
                onCursorChange={(position: any) => {
                  cursorLine = position.line;
                  cursorColumn = position.column;
                }}
                height="100%"
                showToolbar={false}
              />
            {:else if monacoLoading}
              <div class="loading-state">
                <div class="spinner"></div>
                <span>Loading Monaco Editor...</span>
              </div>
            {:else}
              <div class="error-state">
                <Icons.AlertCircle size={32} />
                <span>Editor unavailable</span>
              </div>
            {/if}
          </div>
        </div>

        {#if !showBottomPanel}
          <div class="inspector-side">
            <WorkspaceInspector
              tab={inspectorTab}
              workspaceId={$activeWorkspace?.id ?? ''}
              {previewHtml}
              content={bodyDraft}
              {selectedLanguage}
              terminalCapability={capabilities?.terminal ?? null}
              lspCapability={capabilities?.lsp ?? null}
              onTabChange={(t) => (inspectorTab = t)}
            />
          </div>
        {/if}
      </div>

      {#if showBottomPanel}
        <div class="stage-bottom">
          <div class="bottom-tabs">
            <button class:active={inspectorTab === 'terminal'} onclick={() => inspectorTab = 'terminal'}>Terminal</button>
            <button class:active={inspectorTab === 'preview'} onclick={() => inspectorTab = 'preview'}>Preview</button>
            <button class:active={inspectorTab === 'outline'} onclick={() => inspectorTab = 'outline'}>Outline</button>
            <button class="close-bottom" onclick={() => showBottomPanel = false}><Icons.X size={14} /></button>
          </div>
          <div class="bottom-content">
            <WorkspaceInspector
              tab={inspectorTab}
              workspaceId={$activeWorkspace?.id ?? ''}
              {previewHtml}
              content={bodyDraft}
              {selectedLanguage}
              terminalCapability={capabilities?.terminal ?? null}
              lspCapability={capabilities?.lsp ?? null}
              onTabChange={(t) => (inspectorTab = t)}
            />
          </div>
        </div>
      {/if}

      <StatusBar
        language={selectedLanguage}
        lines={bodyDraft.split('\n').length}
        cursorLine={cursorLine}
        cursorColumn={cursorColumn}
        wordCount={getWordCount(bodyDraft)}
        charCount={bodyDraft.length}
        isDirty={isModified}
      />
    </section>
  {/if}

  <FindReplaceWidget
    isOpen={showFindWidget}
    onClose={() => (showFindWidget = false)}
    onFind={handleFind}
    onReplace={handleReplace}
    onReplaceAll={handleReplaceAll}
    {matchCount}
    currentMatchIndex={currentMatchIndex}
  />
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
    height: 100vh;
    display: flex;
    background: #090a0f;
    overflow: hidden;
  }

  .workspace-activity {
    width: 50px;
    background: #0d0f14;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 0;
    gap: 8px;
    flex-shrink: 0;
  }

  .activity-btn {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-3);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
  }

  .activity-btn:hover,
  .activity-btn.active {
    color: var(--text);
    background: rgba(255, 255, 255, 0.05);
  }

  .activity-spacer {
    flex: 1;
  }

  .workspace-stage {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: #0c0d12;
  }

  .stage-main {
    flex: 1;
    display: flex;
    min-height: 0;
  }

  .editor-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    border-right: 1px solid var(--border);
  }

  .editor-shell {
    flex: 1;
    min-height: 0;
    background: #1e1e1e;
  }

  .inspector-side {
    width: 320px;
    background: #0d0f14;
    flex-shrink: 0;
  }

  .stage-bottom {
    height: 280px;
    background: #0d0f14;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .bottom-tabs {
    display: flex;
    align-items: center;
    padding: 0 12px;
    border-bottom: 1px solid var(--border);
    gap: 16px;
    height: 36px;
    flex-shrink: 0;
  }

  .bottom-tabs button {
    height: 100%;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-3);
    font-family: var(--ff-mono);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    cursor: pointer;
    padding: 0 4px;
  }

  .bottom-tabs button.active {
    color: var(--blue);
    border-bottom-color: var(--blue);
  }

  .close-bottom {
    margin-left: auto;
    color: var(--text-3);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
  }

  .bottom-content {
    flex: 1;
    overflow: hidden;
  }

  .loading-state,
  .error-state {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-3);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin { to { transform: rotate(360deg); } }

  @media (max-width: 1000px) {
    .inspector-side {
      display: none;
    }
  }
</style>
