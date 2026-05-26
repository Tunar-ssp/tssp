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
  import MonacoEditor from '$lib/components/MonacoEditor.svelte';
  import TabBar from '$lib/components/TabBar.svelte';
  import FindReplaceWidget from '$lib/components/FindReplaceWidget.svelte';
  import MarkdownPreview from '$lib/components/MarkdownPreview.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { consumeSelectionIntent } from '$lib/stores/ui';
  import { renderMarkdownLite } from '$lib/utils/markdown';
  import { getWorkspaceCapabilities } from '$lib/services/workspaceService';
  import type { WorkspaceCapabilities } from '$lib/api';
  import { findMatches, replaceMatches } from '$lib/services/workspaceSearchService';
  import type { SearchOptions } from '$lib/services/workspaceSearchService';
  import WorkspaceSidebar from './WorkspaceSidebar.svelte';
  import WorkspaceEditorHeader from './WorkspaceEditorHeader.svelte';
  import WorkspaceInspector from './WorkspaceInspector.svelte';
  import WorkspaceStageHead from './WorkspaceStageHead.svelte';

  type InspectorTab = 'preview' | 'outline' | 'terminal';

  let contextMenu = $state({ visible: false, x: 0, y: 0, workspace: null as any });
  let isLoading = $state(true);
  let bodyDraft = $state('');
  let nameDraft = $state('');
  let selectedLanguage = $state('');
  let workspaceFilterQuery = $state('');
  let showFindWidget = $state(false);
  let inspectorTab = $state<InspectorTab>('preview');
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

  onDestroy(() => {
    if (saveTimer) clearTimeout(saveTimer);
  });

  $effect(() => {
    if ($activeWorkspace) {
      nameDraft = $activeWorkspace.name;
      bodyDraft = $activeWorkspace.body;
      selectedLanguage = $activeWorkspace.language || 'text';
      isModified = false;
      syncOpenTabs();
      activeTabId = $activeWorkspace.id;
      loadCapabilities($activeWorkspace.id);
    } else {
      nameDraft = '';
      bodyDraft = '';
      selectedLanguage = 'text';
      isModified = false;
      capabilities = null;
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

  function getWordCount(text: string) {
    return text.trim().split(/\s+/).filter((word) => word.length > 0).length;
  }

  function summarizeBody(text: string) {
    return text.replace(/\s+/g, ' ').trim();
  }

  function formatRelative(timestamp: number) {
    const delta = Math.max(0, Math.floor(Date.now() / 1000) - timestamp);
    if (delta < 60) return 'just now';
    if (delta < 3_600) return `${Math.floor(delta / 60)}m`;
    if (delta < 86_400) return `${Math.floor(delta / 3_600)}h`;
    if (delta < 604_800) return `${Math.floor(delta / 86_400)}d`;
    return `${Math.floor(delta / 604_800)}w`;
  }

  function languageLabel(language: string) {
    return languages.find((item) => item.id === language)?.label || language || 'Plain Text';
  }

  function languageAccent(language: string) {
    if (language === 'rust') return 'var(--orange)';
    if (language === 'typescript' || language === 'javascript') return 'var(--blue)';
    if (language === 'markdown') return 'var(--green)';
    if (language === 'python') return 'var(--warning)';
    return 'var(--violet)';
  }
</script>

<svelte:window onkeydown={handleEditorKeydown} />

<div class:editor-mode={!!$activeWorkspace} class:home-mode={!$activeWorkspace} class="workspace-view">
  <aside class="workspace-activity">
    <button type="button" class="activity-btn active" title="Explorer">
      <Icons.Files size={18} />
    </button>
    <button type="button" class="activity-btn" title="Find in document" onclick={() => (showFindWidget = true)}>
      <Icons.Search size={18} />
    </button>
    <button type="button" class="activity-btn" title="Preview panel" onclick={() => (inspectorTab = 'preview')}>
      <Icons.PanelRightOpen size={18} />
    </button>
    <button type="button" class="activity-btn" title="Terminal panel" onclick={() => (inspectorTab = 'terminal')}>
      <Icons.TerminalSquare size={18} />
    </button>
  </aside>

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

  {#if !$activeWorkspace}
    <section class="workspace-home">
      <header class="home-hero">
        <div>
          <div class="eyebrow">Workspace</div>
          <h1>Local editor foundation</h1>
          <p>
            TSSP currently stores single-document workspaces. Nested trees, folder actions, and execution are intentionally
            hidden until backend support exists.
          </p>
        </div>

        <div class="hero-actions">
          <button type="button" class="primary-btn" onclick={handleCreateWorkspace}>
            <Icons.Plus size={16} />
            New workspace
          </button>
          <button type="button" class="ghost-btn" onclick={() => recentWorkspaces[0] && handleSelectWorkspace(recentWorkspaces[0].id)}>
            <Icons.FolderOpen size={16} />
            Open latest
          </button>
        </div>
      </header>

      <div class="hub-metrics">
        <article class="metric-card">
          <span>Documents</span>
          <strong>{$workspaces.length}</strong>
          <p>Storage-backed workspace entries</p>
        </article>
        <article class="metric-card">
          <span>Languages</span>
          <strong>{languageCount}</strong>
          <p>Inferred from the saved workspace metadata</p>
        </article>
        <article class="metric-card">
          <span>Sync model</span>
          <strong>Autosave</strong>
          <p>Changes persist to the local backend after short idle windows</p>
        </article>
        <article class="metric-card warning">
          <span>Execution</span>
          <strong>Locked</strong>
          <p>Terminal and run controls remain disabled until a safe sandbox exists</p>
        </article>
      </div>

      <section class="hub-section">
        <div class="section-head">
          <span class="sidebar-label">Recent</span>
        </div>
        <div class="hub-grid">
          {#each recentWorkspaces as workspace (workspace.id)}
            <button type="button" class="hub-card" onclick={() => handleSelectWorkspace(workspace.id)}>
              <div class="hub-card-head">
                <strong>{workspace.name || 'untitled'}</strong>
                <span class="lang-chip" style="--tone: {languageAccent(workspace.language)}">{workspace.language}</span>
              </div>
              <p>{summarizeBody(workspace.body).slice(0, 180) || 'No content yet'}</p>
              <small>{formatRelative(workspace.updated_at)}</small>
            </button>
          {/each}
        </div>
      </section>
    </section>
  {:else}
    <section class="workspace-stage">
      <WorkspaceStageHead
        workspaceName={nameDraft || $activeWorkspace.name || 'untitled'}
        onFind={() => (showFindWidget = !showFindWidget)}
        onDelete={() => handleDeleteWorkspace($activeWorkspace)}
        onSave={() => handleSaveWorkspace()}
      />

      <TabBar
        tabs={openTabs}
        activeTabId={activeTabId}
        onSelectTab={handleTabSelect}
        onCloseTab={handleTabClose}
      />

      <FindReplaceWidget
        isOpen={showFindWidget}
        onClose={() => (showFindWidget = false)}
        onFind={handleFind}
        onReplace={handleReplace}
        onReplaceAll={handleReplaceAll}
        {matchCount}
        currentMatchIndex={currentMatchIndex}
      />

      <div class="editor-banner">
        <div class="banner-copy">
          <Icons.Lock size={14} />
          <span>Sandbox execution is disabled. This workspace is currently editor-only.</span>
        </div>
        <span class="banner-meta">{languageLabel(selectedLanguage)} · autosave enabled</span>
      </div>

      <div class="editor-layout">
        <div class="editor-column">
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

          <div class="monaco-shell">
            {#if isMarkdownFile}
              <MarkdownPreview
                content={bodyDraft}
                showPreview={true}
                onTogglePreview={() => {}}
                onChange={handleEditorInput}
              />
            {:else}
              <MonacoEditor
                value={bodyDraft}
                language={selectedLanguage}
                onChange={handleEditorInput}
                onCursorChange={(position) => {
                  cursorLine = position.line;
                  cursorColumn = position.column;
                }}
                height="100%"
                showToolbar={false}
              />
            {/if}
          </div>
        </div>

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
    min-height: 0;
    display: grid;
    background: linear-gradient(180deg, rgba(11, 13, 18, 1), rgba(8, 10, 14, 1));
  }

  .workspace-view.home-mode {
    grid-template-columns: 56px 300px minmax(0, 1fr);
  }

  .workspace-view.editor-mode {
    grid-template-columns: 56px 300px minmax(0, 1fr);
  }

  .workspace-activity,
  .workspace-home,
  .workspace-stage {
    min-height: 0;
  }

  .workspace-activity {
    border-right: 1px solid var(--border);
    background: rgba(12, 14, 20, 0.98);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 18px 8px;
  }

  .activity-btn {
    width: 40px;
    height: 40px;
    border-radius: 14px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .activity-btn.active,
  .activity-btn:hover {
    background: rgba(29, 34, 46, 0.98);
    border-color: var(--border);
    color: var(--text);
  }


  .workspace-home {
    padding: 28px 32px 36px;
    overflow: auto;
  }

  .home-hero {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    margin-bottom: 28px;
  }

  .eyebrow {
    font-family: var(--ff-mono);
    font-size: 12px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--dim);
  }

  .home-hero h1 {
    margin: 8px 0 0;
    font-size: clamp(42px, 4vw, 64px);
    line-height: 0.98;
    letter-spacing: -0.04em;
  }

  .home-hero p {
    margin: 14px 0 0;
    max-width: 760px;
    color: var(--muted);
    font-size: 18px;
    line-height: 1.6;
  }

  .hero-actions {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .primary-btn,
  .ghost-btn {
    height: 44px;
    padding: 0 16px;
    border-radius: 16px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .primary-btn {
    border: 1px solid rgba(110, 168, 255, 0.24);
    background: rgba(110, 168, 255, 0.14);
    color: var(--blue);
  }

  .primary-btn.compact {
    height: 40px;
  }

  .ghost-btn {
    border: 1px solid var(--border);
    background: rgba(18, 22, 31, 0.96);
    color: var(--text-2);
  }

  .ghost-btn:hover,
  .primary-btn:hover {
    color: var(--text);
  }

  .hub-metrics {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 16px;
    margin-bottom: 28px;
  }

  .metric-card {
    padding: 20px;
    border-radius: 24px;
    border: 1px solid var(--border);
    background: rgba(18, 21, 29, 0.98);
  }

  .metric-card.warning {
    border-color: rgba(251, 191, 36, 0.2);
    background: rgba(45, 32, 16, 0.38);
  }

  .metric-card span {
    color: var(--muted);
    font-family: var(--ff-mono);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.16em;
  }

  .metric-card strong {
    display: block;
    margin-top: 18px;
    font-size: 36px;
    line-height: 1;
  }

  .metric-card p {
    margin: 12px 0 0;
    color: var(--muted);
    line-height: 1.5;
  }

  .hub-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .hub-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }

  .hub-card {
    padding: 22px;
    border-radius: 24px;
    border: 1px solid var(--border);
    background: rgba(17, 20, 28, 0.98);
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .hub-card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .hub-card-head strong {
    font-size: 20px;
  }

  .hub-card p {
    margin: 16px 0;
    color: var(--muted);
    line-height: 1.55;
  }

  .hub-card small {
    color: var(--dim);
    font-family: var(--ff-mono);
  }

  .workspace-stage {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .editor-banner {
    padding: 10px 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    background: rgba(18, 21, 29, 0.96);
    color: var(--text-2);
    font-size: 13px;
  }

  .banner-copy {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .banner-meta {
    color: var(--muted);
    font-family: var(--ff-mono);
    font-size: 12px;
  }

  .editor-layout {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 320px;
  }

  .editor-column {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border);
  }


  .monaco-shell {
    flex: 1;
    min-height: 0;
    padding: 16px;
  }


  @media (max-width: 1400px) {
    .hub-metrics {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .editor-layout {
      grid-template-columns: minmax(0, 1fr);
    }

    .editor-column {
      border-right: 0;
    }
  }

  @media (max-width: 960px) {
    .workspace-view.home-mode,
    .workspace-view.editor-mode {
      grid-template-columns: 1fr;
    }

    .workspace-activity {
      flex-direction: row;
      justify-content: center;
      border-right: 0;
      border-bottom: 1px solid var(--border);
    }

    .home-hero {
      flex-direction: column;
      align-items: stretch;
    }

    .hub-metrics {
      grid-template-columns: 1fr;
    }
  }
</style>
