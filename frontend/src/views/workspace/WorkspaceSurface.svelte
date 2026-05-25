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
  import FindWidget from '$lib/components/FindWidget.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import Outline from '$lib/components/Outline.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { consumeSelectionIntent } from '$lib/stores/ui';
  import { renderMarkdownLite } from '$lib/utils/markdown';

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
    } else {
      nameDraft = '';
      bodyDraft = '';
      selectedLanguage = 'text';
      isModified = false;
    }
  });

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

  function handleFind(query: string, options: { matchCase: boolean; wholeWord: boolean }) {
    if (!query.trim()) return;
    const text = options.matchCase ? bodyDraft : bodyDraft.toLowerCase();
    const search = options.matchCase ? query : query.toLowerCase();
    const index = text.indexOf(search);

    if (index >= 0) {
      updateCursorPositionFromOffset(index);
      success('Match Found', `Line ${cursorLine}, column ${cursorColumn}`);
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

  <aside class="workspace-sidebar">
    <div class="sidebar-head">
      <div>
        <div class="sidebar-label">Explorer</div>
        <h2>Projects</h2>
      </div>
      <button type="button" class="create-btn" onclick={handleCreateWorkspace}>
        <Icons.Plus size={16} />
      </button>
    </div>

    <label class="workspace-search">
      <Icons.Search size={16} />
      <input bind:value={workspaceFilterQuery} placeholder="Find workspace..." />
    </label>

    <div class="workspace-list">
      {#if filteredWorkspaces.length === 0}
        <div class="workspace-empty">
          <Icons.FilePlus2 size={24} />
          <strong>No workspaces</strong>
          <p>Create a new document-backed workspace to start editing.</p>
        </div>
      {:else}
        {#each filteredWorkspaces as workspace (workspace.id)}
          <button
            type="button"
            class="workspace-item"
            class:active={$activeWorkspace?.id === workspace.id}
            onclick={() => handleSelectWorkspace(workspace.id)}
            oncontextmenu={(event) => showContextMenu(event, workspace)}
          >
            <div class="workspace-item-head">
              <strong>{workspace.name || 'untitled'}</strong>
              <span class="lang-chip" style="--tone: {languageAccent(workspace.language)}">{workspace.language}</span>
            </div>
            <p>{summarizeBody(workspace.body).slice(0, 90) || 'Empty workspace document'}</p>
            <div class="workspace-item-meta">
              <span>{formatRelative(workspace.updated_at)}</span>
              <span>{getWordCount(workspace.body)} words</span>
            </div>
          </button>
        {/each}
      {/if}
    </div>

    <div class="sidebar-foot">
      <span class="sync-pill"><span class="status-dot"></span>tssp.local synced</span>
      <small>{$workspaces.length} docs · {languageCount} languages</small>
    </div>
  </aside>

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
      <header class="stage-head">
        <div class="stage-breadcrumbs">
          <span>Workspace</span>
          <span>/</span>
          <strong>{nameDraft || $activeWorkspace.name || 'untitled'}</strong>
        </div>

        <div class="stage-actions">
          <button type="button" class="ghost-btn" onclick={() => (showFindWidget = !showFindWidget)}>
            <Icons.Search size={14} />
            Find
          </button>
          <button type="button" class="ghost-btn" onclick={() => handleDeleteWorkspace($activeWorkspace)}>
            <Icons.Trash2 size={14} />
            Delete
          </button>
          <button type="button" class="primary-btn compact" onclick={() => handleSaveWorkspace()}>
            <Icons.Save size={14} />
            Save
          </button>
        </div>
      </header>

      <TabBar
        tabs={openTabs}
        activeTabId={activeTabId}
        onSelectTab={handleTabSelect}
        onCloseTab={handleTabClose}
      />

      <FindWidget
        isOpen={showFindWidget}
        onClose={() => (showFindWidget = false)}
        onFind={handleFind}
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
          <div class="editor-header">
            <input
              type="text"
              class="name-input"
              placeholder="Untitled workspace"
              bind:value={nameDraft}
              oninput={() => {
                isModified = true;
                syncOpenTabs();
                scheduleWorkspaceSave();
              }}
              onchange={() => handleSaveWorkspace()}
            />

            <div class="editor-actions">
              <select
                class="language-select"
                bind:value={selectedLanguage}
                onchange={(event) => handleChangeLanguage((event.currentTarget as HTMLSelectElement).value)}
              >
                {#each languages as language}
                  <option value={language.id}>{language.label}</option>
                {/each}
              </select>

              {#if $isSaving}
                <span class="saving">Saving...</span>
              {/if}
            </div>
          </div>

          <div class="monaco-shell">
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
          </div>
        </div>

        <aside class="workspace-inspector">
          <div class="inspector-tabs">
            <button type="button" class:active={inspectorTab === 'preview'} onclick={() => (inspectorTab = 'preview')}>Preview</button>
            <button type="button" class:active={inspectorTab === 'outline'} onclick={() => (inspectorTab = 'outline')}>Outline</button>
            <button type="button" class:active={inspectorTab === 'terminal'} onclick={() => (inspectorTab = 'terminal')}>Terminal</button>
          </div>

          {#if inspectorTab === 'preview'}
            <div class="preview-panel">
              {#if selectedLanguage === 'markdown'}
                <div class="markdown-preview">{@html previewHtml}</div>
              {:else}
                <div class="plain-preview-card">
                  <span class="sidebar-label">Preview</span>
                  <p>Rich preview is currently optimized for markdown workspaces. Other languages render as raw text below.</p>
                  <pre>{bodyDraft || '// No content yet'}</pre>
                </div>
              {/if}
            </div>
          {:else if inspectorTab === 'outline'}
            <div class="outline-panel">
              <Outline content={bodyDraft} onSelectItem={() => {}} />
            </div>
          {:else}
            <div class="terminal-panel">
              <div class="terminal-warning">
                <Icons.AlertTriangle size={18} />
                <div>
                  <strong>Run is sandboxed</strong>
                  <p>Execution and terminal access stay locked until the backend exposes a safe sandbox. Admin maintenance tools remain available separately.</p>
                </div>
              </div>
              <div class="terminal-placeholder">
                <code>// scratch: run requires sandbox unlock</code>
              </div>
            </div>
          {/if}
        </aside>
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
  .workspace-sidebar,
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

  .workspace-sidebar {
    border-right: 1px solid var(--border);
    background: rgba(16, 18, 24, 0.98);
    display: flex;
    flex-direction: column;
    padding: 20px 16px;
    gap: 16px;
  }

  .sidebar-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .sidebar-label {
    font-family: var(--ff-mono);
    font-size: 12px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--dim);
  }

  .sidebar-head h2 {
    margin: 8px 0 0;
    font-size: 20px;
  }

  .create-btn {
    width: 38px;
    height: 38px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: rgba(24, 28, 38, 0.98);
    color: var(--text);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .workspace-search {
    height: 46px;
    padding: 0 14px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(12, 14, 20, 0.98);
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
  }

  .workspace-search input {
    width: 100%;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
  }

  .workspace-list {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .workspace-empty {
    min-height: 180px;
    padding: 20px;
    border-radius: 22px;
    border: 1px dashed var(--border);
    background: rgba(13, 15, 21, 0.92);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--muted);
    text-align: center;
  }

  .workspace-empty strong {
    color: var(--text);
  }

  .workspace-item {
    padding: 16px;
    border-radius: 20px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .workspace-item:hover,
  .workspace-item.active {
    background: rgba(29, 33, 43, 0.98);
    border-color: var(--border);
  }

  .workspace-item-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .workspace-item-head strong {
    font-size: 15px;
  }

  .lang-chip {
    height: 26px;
    padding: 0 10px;
    border-radius: 999px;
    border: 1px solid color-mix(in srgb, var(--tone) 32%, transparent);
    background: color-mix(in srgb, var(--tone) 12%, transparent);
    color: var(--tone);
    display: inline-flex;
    align-items: center;
    font-size: 12px;
    font-family: var(--ff-mono);
  }

  .workspace-item p {
    margin: 10px 0 0;
    color: var(--muted);
    line-height: 1.5;
  }

  .workspace-item-meta {
    margin-top: 12px;
    display: flex;
    justify-content: space-between;
    gap: 8px;
    color: var(--dim);
    font-family: var(--ff-mono);
    font-size: 12px;
  }

  .sidebar-foot {
    padding-top: 8px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sync-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--green);
    font-size: 14px;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 999px;
    background: currentColor;
    box-shadow: 0 0 18px currentColor;
  }

  .sidebar-foot small {
    color: var(--muted);
    font-family: var(--ff-mono);
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

  .hero-actions,
  .stage-actions {
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

  .stage-head {
    min-height: 74px;
    padding: 16px 22px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: rgba(12, 14, 20, 0.98);
  }

  .stage-breadcrumbs {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
    flex-wrap: wrap;
  }

  .stage-breadcrumbs strong {
    color: var(--text);
    font-size: 18px;
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

  .editor-header {
    padding: 14px 18px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    background: rgba(15, 17, 23, 0.96);
  }

  .name-input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 28px;
    line-height: 1.1;
    font-weight: 700;
    letter-spacing: -0.03em;
  }

  .name-input::placeholder {
    color: var(--dim);
  }

  .editor-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .language-select {
    height: 38px;
    padding: 0 12px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: rgba(12, 14, 20, 0.98);
    color: var(--text);
  }

  .saving {
    color: var(--warning);
    font-size: 13px;
  }

  .monaco-shell {
    flex: 1;
    min-height: 0;
    padding: 16px;
  }

  .workspace-inspector {
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: rgba(14, 16, 22, 0.98);
  }

  .inspector-tabs {
    padding: 12px;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
    border-bottom: 1px solid var(--border);
  }

  .inspector-tabs button {
    height: 40px;
    border-radius: 14px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
  }

  .inspector-tabs button.active {
    background: rgba(34, 42, 58, 0.95);
    color: var(--text);
  }

  .preview-panel,
  .outline-panel,
  .terminal-panel {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .preview-panel {
    padding: 18px;
  }

  .outline-panel :global(.outline) {
    height: 100%;
    border-right: 0;
    background: transparent;
  }

  .markdown-preview {
    color: var(--text);
    line-height: 1.7;
  }

  .markdown-preview :global(h1),
  .markdown-preview :global(h2),
  .markdown-preview :global(h3) {
    margin: 0 0 12px;
    line-height: 1.05;
    letter-spacing: -0.03em;
  }

  .markdown-preview :global(h1) {
    font-size: 34px;
  }

  .markdown-preview :global(h2) {
    font-size: 26px;
  }

  .markdown-preview :global(p),
  .markdown-preview :global(ul),
  .markdown-preview :global(ol),
  .markdown-preview :global(pre),
  .markdown-preview :global(blockquote) {
    margin: 0 0 14px;
  }

  .markdown-preview :global(pre) {
    padding: 14px;
    border-radius: 18px;
    overflow: auto;
    background: rgba(12, 14, 20, 0.98);
    border: 1px solid var(--border);
  }

  .plain-preview-card,
  .terminal-warning,
  .terminal-placeholder {
    padding: 18px;
    border-radius: 20px;
    border: 1px solid var(--border);
    background: rgba(18, 21, 29, 0.98);
  }

  .plain-preview-card p,
  .terminal-warning p {
    color: var(--muted);
    line-height: 1.6;
  }

  .plain-preview-card pre,
  .terminal-placeholder code {
    display: block;
    margin-top: 14px;
    padding: 14px;
    border-radius: 16px;
    background: rgba(12, 14, 20, 0.98);
    border: 1px solid var(--border);
    color: var(--text-2);
    font-family: var(--ff-mono);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .terminal-panel {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .terminal-warning {
    display: flex;
    gap: 12px;
    color: var(--warning);
  }

  .terminal-warning strong {
    display: block;
    color: var(--text);
    margin-bottom: 6px;
  }

  @media (max-width: 1400px) {
    .hub-metrics {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .editor-layout {
      grid-template-columns: minmax(0, 1fr);
    }

    .workspace-inspector {
      border-top: 1px solid var(--border);
      min-height: 320px;
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

    .workspace-sidebar {
      border-right: 0;
      border-bottom: 1px solid var(--border);
      max-height: 360px;
    }

    .home-hero,
    .stage-head,
    .editor-header {
      flex-direction: column;
      align-items: stretch;
    }

    .hub-metrics {
      grid-template-columns: 1fr;
    }
  }
</style>
