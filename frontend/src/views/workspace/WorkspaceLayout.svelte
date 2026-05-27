<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    activePanel?: 'editor' | 'explorer' | 'outline' | 'debug';
    activeFile?: { name: string; path: string };
    onPanelChange?: (panel: string) => void;
    onFileOpen?: (path: string) => void;
    children?: import('svelte').Snippet;
    footer?: import('svelte').Snippet;
  }

  let {
    activePanel = 'editor',
    activeFile,
    onPanelChange,
    onFileOpen,
    children,
    footer,
  }: Props = $props();

  let explorerOpen = $state(true);
  let rightPanelOpen = $state(true);
  let bottomPanelOpen = $state(false);

  const panels = [
    { id: 'explorer', label: 'Explorer', icon: Icons.FileText },
    { id: 'outline', label: 'Outline', icon: Icons.List },
    { id: 'debug', label: 'Debug', icon: Icons.Bug },
  ];
</script>

<div class="workspace-layout">
  <!-- Left Sidebar - Explorer -->
  <aside class="workspace-sidebar sidebar-left" class:collapsed={!explorerOpen}>
    <div class="sidebar-header">
      <h3 class="sidebar-title">Explorer</h3>
      <button
        class="sidebar-toggle"
        onclick={() => (explorerOpen = !explorerOpen)}
        title={explorerOpen ? 'Collapse' : 'Expand'}
      >
        <Icons.ChevronRight size={16} />
      </button>
    </div>

    {#if explorerOpen}
      <div class="sidebar-content">
        <!-- File Explorer Content Here -->
        <div class="placeholder">
          <Icons.FileText size={32} />
          <p>Explorer</p>
        </div>
      </div>
    {/if}
  </aside>

  <!-- Main Editor Area -->
  <main class="workspace-main">
    <div class="editor-header">
      <div class="editor-tabs">
        {#if activeFile}
          <div class="tab active">
            <Icons.FileCode size={14} />
            <span>{activeFile.name}</span>
            <button class="tab-close">
              <Icons.X size={12} />
            </button>
          </div>
        {/if}
      </div>

      <div class="editor-toolbar">
        <div class="toolbar-group">
          <button class="icon-btn" title="Format (Shift+Alt+F)">
            <Icons.Code size={16} />
          </button>
          <button class="icon-btn" title="Save (Ctrl+S)">
            <Icons.Save size={16} />
          </button>
        </div>
      </div>
    </div>

    <div class="editor-content">
      {#if children}
        {@render children()}
      {:else}
        <div class="editor-placeholder">
          <Icons.Code2 size={48} />
          <p>Select a file to start editing</p>
          <small>Ctrl+O to open file</small>
        </div>
      {/if}
    </div>

    {#if bottomPanelOpen}
      <div class="bottom-panel">
        <div class="panel-header">
          <span>Terminal</span>
          <button
            class="panel-close"
            onclick={() => (bottomPanelOpen = false)}
          >
            <Icons.X size={16} />
          </button>
        </div>
        <div class="panel-content">
          <!-- Terminal Content Here -->
        </div>
      </div>
    {/if}
  </main>

  <!-- Right Sidebar - Panels -->
  <aside class="workspace-sidebar sidebar-right" class:collapsed={!rightPanelOpen}>
    <div class="sidebar-header">
      <div class="panel-tabs">
        {#each panels as panel}
          <button
            class="panel-tab"
            class:active={activePanel === panel.id}
            onclick={() => {
              onPanelChange?.(panel.id);
            }}
            title={panel.label}
          >
            <svelte:component this={panel.icon} size={16} />
          </button>
        {/each}
      </div>
      <button
        class="sidebar-toggle"
        onclick={() => (rightPanelOpen = !rightPanelOpen)}
        title={rightPanelOpen ? 'Collapse' : 'Expand'}
      >
        <Icons.ChevronLeft size={16} />
      </button>
    </div>

    {#if rightPanelOpen}
      <div class="sidebar-content panel-content">
        <div class="placeholder">
          <p>{activePanel}</p>
        </div>
      </div>
    {/if}
  </aside>
</div>

{#if footer}
  <footer class="workspace-footer">
    {@render footer()}
  </footer>
{/if}

<style>
  .workspace-layout {
    display: grid;
    grid-template-columns: minmax(220px, 1fr) 2fr minmax(200px, 1fr);
    height: 100%;
    gap: 0;
    background: var(--bg);
  }

  .workspace-sidebar {
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    background: var(--surface);
    overflow: hidden;
    transition: all 200ms var(--ease-smooth);
  }

  .workspace-sidebar.collapsed {
    grid-column-start: span 0;
    width: 0;
    border: none;
  }

  .sidebar-left.collapsed {
    display: none;
  }

  .sidebar-right.collapsed {
    display: none;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-3) var(--s-3);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .sidebar-title {
    font-size: var(--fs-12);
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0;
  }

  .sidebar-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    border-radius: var(--r-2);
    transition: all 150ms;
  }

  .sidebar-toggle:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-2);
  }

  .workspace-main {
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  .editor-header {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    flex-shrink: 0;
  }

  .editor-tabs {
    display: flex;
    gap: var(--s-1);
    flex: 1;
    min-width: 0;
    overflow-x: auto;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    padding: var(--s-2) var(--s-2);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--r-2) var(--r-2) 0 0;
    color: var(--text-2);
    font-size: var(--fs-12);
    white-space: nowrap;
    border-bottom: none;
  }

  .tab.active {
    background: var(--bg);
    color: var(--text);
  }

  .tab-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border: none;
    background: transparent;
    cursor: pointer;
    color: inherit;
  }

  .editor-toolbar {
    display: flex;
    gap: var(--s-2);
    flex-shrink: 0;
  }

  .toolbar-group {
    display: flex;
    gap: var(--s-1);
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    border-radius: var(--r-2);
    transition: all 150ms;
  }

  .icon-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .editor-content {
    flex: 1;
    overflow: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg);
  }

  .editor-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-3);
    color: var(--muted);
    text-align: center;
  }

  .editor-placeholder p {
    margin: 0;
    font-size: var(--fs-14);
  }

  .editor-placeholder small {
    font-size: var(--fs-12);
    color: var(--dim);
  }

  .bottom-panel {
    border-top: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    max-height: 200px;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--border);
    font-size: var(--fs-12);
    font-weight: 600;
    flex-shrink: 0;
  }

  .panel-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    border-radius: var(--r-1);
  }

  .panel-close:hover {
    background: var(--surface-2);
  }

  .panel-content {
    overflow: auto;
    flex: 1;
  }

  .placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    padding: var(--s-6);
    color: var(--muted);
  }

  .placeholder p {
    margin: 0;
    text-transform: capitalize;
  }

  .panel-tabs {
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
  }

  .panel-tab {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    border-radius: var(--r-2);
    transition: all 150ms;
  }

  .panel-tab:hover {
    background: var(--surface-2);
    color: var(--text-2);
  }

  .panel-tab.active {
    background: var(--blue-subtle);
    color: var(--blue);
  }

  .workspace-footer {
    padding: var(--s-2);
    border-top: 1px solid var(--border);
    background: var(--surface);
    font-size: var(--fs-12);
  }

  @media (max-width: 1200px) {
    .workspace-layout {
      grid-template-columns: 1fr minmax(200px, 1fr);
    }

    .sidebar-left {
      display: none;
    }
  }

  @media (max-width: 768px) {
    .workspace-layout {
      grid-template-columns: 1fr;
    }

    .sidebar-right {
      display: none;
    }
  }
</style>
