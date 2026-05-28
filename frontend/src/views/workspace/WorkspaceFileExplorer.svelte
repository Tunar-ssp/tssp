<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { workspaceApi, type WorkspaceFileEntry } from '$lib/api';
  import { error, success } from '$lib/stores/notifications';
  import FileIcon from '$lib/components/FileIcon.svelte';

  interface Props {
    workspaceId: string;
    activeFilePath?: string | null;
    onSelectFile?: (path: string) => void;
  }

  let { workspaceId, activeFilePath = null, onSelectFile = () => {} }: Props = $props();

  type TreeNode = WorkspaceFileEntry & {
    name: string;
    children?: TreeNode[];
    loading?: boolean;
  };

  let root = $state<TreeNode[]>([]);
  let expanded = $state<Record<string, boolean>>({});
  let isLoading = $state(true);
  let renaming = $state<string | null>(null);
  let renameValue = $state('');
  let menu = $state<{ x: number; y: number; node: TreeNode | null; visible: boolean }>({
    x: 0, y: 0, node: null, visible: false,
  });
  let newDialog = $state<{ open: boolean; parent: string; type: 'file' | 'folder'; value: string }>({
    open: false, parent: '', type: 'file', value: '',
  });

  function entryName(path: string): string {
    return path.split('/').filter(Boolean).pop() ?? path;
  }

  function toNodes(entries: WorkspaceFileEntry[]): TreeNode[] {
    return entries
      .map((e) => ({ ...e, name: entryName(e.path) }))
      .sort((a, b) => {
        if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
        return a.name.localeCompare(b.name);
      });
  }

  async function loadRoot() {
    isLoading = true;
    try {
      const res = await workspaceApi.listWorkspaceFiles(workspaceId, '');
      root = toNodes(res.entries || []);
    } catch (err) {
      error('Load Files Failed', err instanceof Error ? err.message : 'Could not load files');
      root = [];
    } finally {
      isLoading = false;
    }
  }

  async function loadChildren(node: TreeNode) {
    try {
      node.loading = true;
      const res = await workspaceApi.listWorkspaceFiles(workspaceId, node.path);
      node.children = toNodes(res.entries || []);
    } catch (err) {
      error('Load Failed', err instanceof Error ? err.message : 'Could not load folder');
      node.children = [];
    } finally {
      node.loading = false;
      root = root;
    }
  }

  async function toggleFolder(node: TreeNode) {
    if (!node.is_dir) return;
    if (expanded[node.path]) {
      delete expanded[node.path];
      expanded = expanded;
    } else {
      expanded[node.path] = true;
      expanded = expanded;
      if (!node.children) await loadChildren(node);
    }
  }

  function handleSelect(node: TreeNode) {
    if (node.is_dir) {
      void toggleFolder(node);
    } else {
      onSelectFile(node.path);
    }
  }

  function openMenu(e: MouseEvent, node: TreeNode | null) {
    e.preventDefault();
    e.stopPropagation();
    menu = { x: e.clientX, y: e.clientY, node, visible: true };
  }

  function closeMenu() {
    menu = { ...menu, visible: false };
  }

  async function refreshParent(path: string) {
    const parent = path.includes('/') ? path.slice(0, path.lastIndexOf('/')) : '';
    if (!parent) {
      await loadRoot();
    } else {
      const node = findNode(root, parent);
      if (node) await loadChildren(node);
      else await loadRoot();
    }
  }

  function findNode(nodes: TreeNode[], path: string): TreeNode | null {
    for (const n of nodes) {
      if (n.path === path) return n;
      if (n.children) {
        const r = findNode(n.children, path);
        if (r) return r;
      }
    }
    return null;
  }

  function startRename(node: TreeNode) {
    renaming = node.path;
    renameValue = node.name;
    closeMenu();
  }

  async function commitRename(node: TreeNode) {
    const newName = renameValue.trim();
    renaming = null;
    if (!newName || newName === node.name) return;
    const parentPath = node.path.includes('/') ? node.path.slice(0, node.path.lastIndexOf('/')) : '';
    const target = parentPath ? `${parentPath}/${newName}` : newName;
    try {
      await workspaceApi.moveWorkspaceFile(workspaceId, node.path, target);
      success('Renamed', target);
      await refreshParent(node.path);
    } catch (err) {
      error('Rename Failed', err instanceof Error ? err.message : 'Could not rename');
    }
  }

  async function deleteNode(node: TreeNode) {
    closeMenu();
    if (!confirm(`Delete ${node.is_dir ? 'folder' : 'file'} "${node.name}"?`)) return;
    try {
      await workspaceApi.deleteWorkspaceFile(workspaceId, node.path);
      success('Deleted', node.path);
      await refreshParent(node.path);
    } catch (err) {
      error('Delete Failed', err instanceof Error ? err.message : 'Could not delete');
    }
  }

  function openNewDialog(parent: string, type: 'file' | 'folder') {
    newDialog = { open: true, parent, type, value: '' };
    closeMenu();
  }

  async function commitNew() {
    const name = newDialog.value.trim();
    if (!name) return;
    const fullPath = newDialog.parent ? `${newDialog.parent}/${name}` : name;
    try {
      if (newDialog.type === 'file') {
        await workspaceApi.createWorkspaceFile(workspaceId, fullPath, '');
      } else {
        await workspaceApi.createWorkspaceDirectory(workspaceId, fullPath);
      }
      success(newDialog.type === 'file' ? 'File Created' : 'Folder Created', fullPath);
      if (newDialog.parent) {
        expanded[newDialog.parent] = true;
        expanded = expanded;
      }
      await refreshParent(fullPath);
      newDialog = { open: false, parent: '', type: 'file', value: '' };
      if (newDialog.type === 'file') onSelectFile(fullPath);
    } catch (err) {
      error('Create Failed', err instanceof Error ? err.message : 'Could not create');
    }
  }

  $effect(() => {
    if (workspaceId) void loadRoot();
  });
</script>

<svelte:window onclick={() => menu.visible && closeMenu()} />

<aside class="explorer">
  <header class="hdr">
    <div class="title">
      <Icons.Files size={14} />
      <span>EXPLORER</span>
    </div>
    <div class="actions">
      <button type="button" class="ibtn" title="New File" onclick={() => openNewDialog('', 'file')}>
        <Icons.FilePlus size={14} />
      </button>
      <button type="button" class="ibtn" title="New Folder" onclick={() => openNewDialog('', 'folder')}>
        <Icons.FolderPlus size={14} />
      </button>
      <button type="button" class="ibtn" title="Refresh" onclick={loadRoot}>
        <Icons.RefreshCw size={14} />
      </button>
    </div>
  </header>

  <div class="tree" oncontextmenu={(e) => openMenu(e, null)} role="tree" tabindex="-1">
    {#if isLoading}
      <div class="state"><span class="spin"></span>Loading…</div>
    {:else if root.length === 0}
      <div class="state empty">
        <Icons.FolderOpen size={20} />
        <span>Empty workspace</span>
        <button type="button" class="link" onclick={() => openNewDialog('', 'file')}>Create first file</button>
      </div>
    {:else}
      {#snippet treeRow(node: TreeNode, depth: number)}
        <div class="row-wrap">
          <button
            type="button"
            class="row"
            class:active={activeFilePath === node.path}
            class:dir={node.is_dir}
            style="padding-left: {6 + depth * 12}px"
            onclick={() => handleSelect(node)}
            oncontextmenu={(e) => openMenu(e, node)}
          >
            {#if node.is_dir}
              <span class="chev" class:open={expanded[node.path]}>
                <Icons.ChevronRight size={12} />
              </span>
              <Icons.Folder size={14} />
            {:else}
              <span class="chev"></span>
              <FileIcon name={node.name} size={14} />
            {/if}
            {#if renaming === node.path}
              <input
                class="rename"
                bind:value={renameValue}
                onblur={() => commitRename(node)}
                onkeydown={(e) => {
                  if (e.key === 'Enter') commitRename(node);
                  else if (e.key === 'Escape') renaming = null;
                }}
              />
            {:else}
              <span class="name">{node.name}</span>
            {/if}
          </button>
          {#if node.is_dir && expanded[node.path] && node.children}
            {#each node.children as child (child.path)}
              {@render treeRow(child, depth + 1)}
            {/each}
            {#if node.children.length === 0}
              <div class="row empty-folder" style="padding-left: {6 + (depth + 1) * 12}px">(empty)</div>
            {/if}
          {/if}
        </div>
      {/snippet}

      {#each root as node (node.path)}
        {@render treeRow(node, 0)}
      {/each}
    {/if}
  </div>
</aside>

{#if menu.visible}
  <div class="menu" style="left:{menu.x}px;top:{menu.y}px">
    {#if menu.node}
      {#if !menu.node.is_dir}
        <button type="button" onclick={() => { if (menu.node) { onSelectFile(menu.node.path); closeMenu(); } }}>
          <Icons.FileText size={13} /> Open
        </button>
      {/if}
      {#if menu.node.is_dir}
        <button type="button" onclick={() => { if (menu.node) openNewDialog(menu.node.path, 'file'); }}>
          <Icons.FilePlus size={13} /> New File
        </button>
        <button type="button" onclick={() => { if (menu.node) openNewDialog(menu.node.path, 'folder'); }}>
          <Icons.FolderPlus size={13} /> New Folder
        </button>
      {/if}
      <button type="button" onclick={() => { if (menu.node) startRename(menu.node); }}>
        <Icons.Edit3 size={13} /> Rename
      </button>
      <button type="button" class="danger" onclick={() => { if (menu.node) deleteNode(menu.node); }}>
        <Icons.Trash2 size={13} /> Delete
      </button>
    {:else}
      <button type="button" onclick={() => openNewDialog('', 'file')}>
        <Icons.FilePlus size={13} /> New File
      </button>
      <button type="button" onclick={() => openNewDialog('', 'folder')}>
        <Icons.FolderPlus size={13} /> New Folder
      </button>
    {/if}
  </div>
{/if}

{#if newDialog.open}
  <div
    class="overlay"
    role="presentation"
    onclick={() => newDialog = { ...newDialog, open: false }}
  >
    <div class="dialog" role="dialog" onclick={(e) => e.stopPropagation()}>
      <h3>New {newDialog.type === 'file' ? 'File' : 'Folder'}{newDialog.parent ? ` in ${newDialog.parent}` : ''}</h3>
      <input
        type="text"
        placeholder={newDialog.type === 'file' ? 'filename.ext' : 'folder-name'}
        bind:value={newDialog.value}
        onkeydown={(e) => {
          if (e.key === 'Enter') commitNew();
          else if (e.key === 'Escape') newDialog = { ...newDialog, open: false };
        }}
        autofocus
      />
      <div class="dialog-actions">
        <button type="button" class="btn" onclick={() => newDialog = { ...newDialog, open: false }}>Cancel</button>
        <button type="button" class="btn primary" onclick={commitNew}>Create</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .explorer {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 280px;
    flex-shrink: 0;
    background: rgba(14, 16, 22, 0.98);
    border-right: 1px solid var(--border);
    overflow: hidden;
  }
  .hdr {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-2);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
  }
  .actions { display: flex; gap: 2px; }
  .ibtn {
    width: 22px; height: 22px;
    border: none; background: transparent; color: var(--text-2);
    display: flex; align-items: center; justify-content: center;
    border-radius: 4px; cursor: pointer;
  }
  .ibtn:hover { background: rgba(255,255,255,0.08); color: var(--text); }
  .tree {
    flex: 1; overflow-y: auto; overflow-x: hidden;
    padding: 4px 0;
    font-size: 13px;
  }
  .state {
    display: flex; align-items: center; gap: 8px;
    padding: 14px; color: var(--muted); font-size: 12px;
  }
  .state.empty { flex-direction: column; padding: 24px 12px; text-align: center; }
  .link {
    background: none; border: none; color: var(--blue);
    cursor: pointer; font-size: 12px; padding: 0;
  }
  .row-wrap { display: contents; }
  .row {
    width: 100%;
    display: flex; align-items: center; gap: 4px;
    padding: 2px 8px 2px 6px;
    border: none; background: transparent;
    color: var(--text-2); text-align: left;
    cursor: pointer; font-size: 13px;
    line-height: 22px; min-height: 22px;
    overflow: hidden;
  }
  .row:hover { background: rgba(255,255,255,0.05); color: var(--text); }
  .row.active { background: rgba(59,130,246,0.18); color: var(--text); }
  .row.empty-folder { color: var(--muted); font-style: italic; pointer-events: none; }
  .chev {
    width: 14px; height: 14px;
    display: inline-flex; align-items: center; justify-content: center;
    transition: transform 0.12s;
    color: var(--muted);
    flex-shrink: 0;
  }
  .chev.open { transform: rotate(90deg); }
  .name {
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
    flex: 1; min-width: 0;
  }
  .rename {
    flex: 1;
    background: var(--bg);
    border: 1px solid var(--blue);
    color: var(--text);
    border-radius: 3px;
    padding: 1px 4px;
    font-size: 13px;
    outline: none;
    min-width: 0;
  }
  .menu {
    position: fixed; z-index: 1000;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    min-width: 160px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
  }
  .menu button {
    width: 100%;
    display: flex; align-items: center; gap: 8px;
    padding: 6px 10px;
    border: none; background: transparent;
    color: var(--text);
    cursor: pointer;
    border-radius: 4px;
    font-size: 13px;
    text-align: left;
  }
  .menu button:hover { background: rgba(255,255,255,0.08); }
  .menu button.danger { color: #ef4444; }
  .menu button.danger:hover { background: rgba(239,68,68,0.12); }
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.5);
    display: flex; align-items: center; justify-content: center;
    z-index: 1100;
  }
  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 20px;
    min-width: 360px;
    display: flex; flex-direction: column; gap: 12px;
  }
  .dialog h3 { margin: 0; font-size: 14px; color: var(--text); }
  .dialog input {
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg); color: var(--text);
    font-size: 14px; outline: none;
  }
  .dialog input:focus { border-color: var(--blue); }
  .dialog-actions { display: flex; justify-content: flex-end; gap: 8px; }
  .btn {
    padding: 6px 14px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
  }
  .btn.primary { background: var(--blue); border-color: var(--blue); color: white; }
  .btn:hover { background: rgba(255,255,255,0.06); }
  .btn.primary:hover { filter: brightness(1.1); }
  .spin {
    width: 12px; height: 12px;
    border: 2px solid rgba(255,255,255,0.15);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
