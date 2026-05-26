<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Workspace {
    id: string;
    name: string;
    language: string;
    body: string;
    updated_at: number;
  }

  interface $$Props {
    workspaces: Workspace[];
    filterQuery: string;
    activeWorkspaceId: string | null;
    languageCount: number;
    showFileTree: boolean;
    onFilterChange: (query: string) => void;
    onSelectWorkspace: (id: string) => void;
    onShowContextMenu: (event: MouseEvent, workspace: Workspace) => void;
    onCreateWorkspace: () => void;
    onToggleFileTree: (show: boolean) => void;
  }

  let {
    workspaces,
    filterQuery,
    activeWorkspaceId,
    languageCount,
    showFileTree,
    onFilterChange,
    onSelectWorkspace,
    onShowContextMenu,
    onCreateWorkspace,
    onToggleFileTree,
  }: $$Props = $props();

  function formatRelative(timestamp: number) {
    const delta = Math.max(0, Math.floor(Date.now() / 1000) - timestamp);
    if (delta < 60) return 'just now';
    if (delta < 3_600) return `${Math.floor(delta / 60)}m`;
    if (delta < 86_400) return `${Math.floor(delta / 3_600)}h`;
    if (delta < 604_800) return `${Math.floor(delta / 86_400)}d`;
    return `${Math.floor(delta / 604_800)}w`;
  }

  function summarizeBody(text: string) {
    return text.replace(/\s+/g, ' ').trim();
  }

  function languageAccent(language: string) {
    if (language === 'rust') return 'var(--orange)';
    if (language === 'typescript' || language === 'javascript') return 'var(--blue)';
    if (language === 'markdown') return 'var(--green)';
    if (language === 'python') return 'var(--warning)';
    return 'var(--violet)';
  }
</script>

<aside class="enhanced-sidebar">
  <div class="sidebar-head">
    <div>
      <div class="sidebar-label">Workspace</div>
      <h2>{showFileTree ? 'Files' : 'Projects'}</h2>
    </div>
    <div class="header-actions">
      <button
        type="button"
        class="view-toggle"
        class:active={showFileTree}
        onclick={() => onToggleFileTree(!showFileTree)}
        title={showFileTree ? 'Show projects' : 'Show file tree'}
        aria-label="Toggle view"
      >
        <Icons.FileText size={16} />
      </button>
      <button type="button" class="create-btn" onclick={onCreateWorkspace}>
        <Icons.Plus size={16} />
      </button>
    </div>
  </div>

  <label class="workspace-search">
    <Icons.Search size={16} />
    <input
      value={filterQuery}
      oninput={(e) => onFilterChange((e.target as HTMLInputElement).value)}
      placeholder={showFileTree ? 'Find files...' : 'Find workspace...'}
    />
  </label>

  {#if !showFileTree}
    <div class="workspace-list">
      {#if workspaces.length === 0}
        <div class="workspace-empty">
          <Icons.FilePlus2 size={24} />
          <strong>No workspaces</strong>
          <p>Create a new document-backed workspace to start editing.</p>
        </div>
      {:else}
        {#each workspaces as workspace (workspace.id)}
          <button
            type="button"
            class="workspace-item"
            class:active={activeWorkspaceId === workspace.id}
            onclick={() => onSelectWorkspace(workspace.id)}
            oncontextmenu={(event) => onShowContextMenu(event, workspace)}
          >
            <div class="workspace-item-head">
              <strong>{workspace.name || 'untitled'}</strong>
              <span class="lang-chip" style="--tone: {languageAccent(workspace.language)}">
                {workspace.language}
              </span>
            </div>
            <p>{summarizeBody(workspace.body).slice(0, 90) || 'Empty workspace document'}</p>
            <div class="workspace-item-meta">
              <span>{formatRelative(workspace.updated_at)}</span>
            </div>
          </button>
        {/each}
      {/if}
    </div>

    <div class="sidebar-foot">
      <span class="sync-pill"><span class="status-dot"></span>tssp.local synced</span>
      <small>{workspaces.length} docs · {languageCount} languages</small>
    </div>
  {:else}
    <div class="file-tree-placeholder">
      <Icons.FolderOpen size={32} />
      <p>File tree view</p>
      <small>Coming soon in workspace</small>
    </div>
  {/if}
</aside>

<style>
  .enhanced-sidebar {
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

  .sidebar-head > div:first-child {
    flex: 1;
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

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .view-toggle,
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
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .view-toggle:hover,
  .create-btn:hover {
    background: var(--surface-2);
  }

  .view-toggle.active {
    background: var(--blue-soft);
    border-color: var(--blue);
    color: var(--blue);
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
    transition: all var(--duration-quick) var(--ease-smooth);
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

  .file-tree-placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
    text-align: center;
  }

  .file-tree-placeholder p {
    margin: 0;
    font-size: 14px;
  }

  .file-tree-placeholder small {
    color: var(--dim);
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
</style>
