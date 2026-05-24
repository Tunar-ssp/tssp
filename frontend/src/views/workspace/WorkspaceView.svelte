<script lang="ts">
  import { onMount } from "svelte";
  import {
    createAdminWorkspaceDocument,
    createWorkspace,
    deleteAdminWorkspaceDocument,
    getAdminWorkspaceDetail,
    getAdminWorkspaceDocument,
    listWorkspaces,
    updateAdminWorkspaceDocument,
    type WorkspaceDocumentRecord,
    type WorkspaceRecord,
  } from "../../lib/api";
  import { inferLanguageFromPath } from "../../lib/utils/workspace";
  import CodeEditor from "./CodeEditor.svelte";
  import WorkspaceExplorer from "./WorkspaceExplorer.svelte";

  interface OpenTab {
    id: string;
    path: string;
    language: string;
    body: string;
    dirty: boolean;
  }

  let workspaces: WorkspaceRecord[] = [];
  let activeWorkspace: WorkspaceRecord | null = null;
  let documents: WorkspaceDocumentRecord[] = [];
  let tabs: OpenTab[] = [];
  let activeTabId: string | null = null;
  let loading = true;
  let error = "";
  let saveStatus = "";

  function activeTab(): OpenTab | null {
    return tabs.find((t) => t.id === activeTabId) || null;
  }

  async function loadWorkspaceDetail(workspaceId: string) {
    const detail = await getAdminWorkspaceDetail(workspaceId);
    activeWorkspace = detail.workspace;
    documents = [];
    tabs = [];
    activeTabId = null;
    for (const summary of detail.documents) {
      const doc = await getAdminWorkspaceDocument(workspaceId, summary.id);
      documents = [...documents, doc];
    }
    if (documents[0]) await openTab(documents[0].id);
  }

  async function loadWorkspaces() {
    loading = true;
    error = "";
    try {
      const res = await listWorkspaces();
      workspaces = res.workspaces || [];
      if (workspaces[0]) await loadWorkspaceDetail(workspaces[0].id);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load workspaces";
    } finally {
      loading = false;
    }
  }

  async function openTab(documentId: string) {
    if (!activeWorkspace) return;
    const existing = tabs.find((t) => t.id === documentId);
    if (existing) {
      activeTabId = documentId;
      return;
    }
    const doc = documents.find((d) => d.id === documentId) || (await getAdminWorkspaceDocument(activeWorkspace.id, documentId));
    if (!documents.some((d) => d.id === doc.id)) documents = [...documents, doc];
    tabs = [
      ...tabs,
      {
        id: doc.id,
        path: doc.path,
        language: doc.language || inferLanguageFromPath(doc.path),
        body: doc.body,
        dirty: false,
      },
    ];
    activeTabId = doc.id;
  }

  async function saveTab() {
    const tab = activeTab();
    const ws = activeWorkspace;
    if (!tab || !ws) return;
    const updated = await updateAdminWorkspaceDocument(ws.id, tab.id, {
      path: tab.path,
      language: tab.language,
      body: tab.body,
      make_primary: false,
    });
    documents = documents.map((d) => (d.id === updated.id ? updated : d));
    tabs = tabs.map((t) => (t.id === tab.id ? { ...t, body: updated.body, dirty: false } : t));
    saveStatus = "Saved";
  }

  async function createFile() {
    const ws = activeWorkspace;
    if (!ws) return;
    const path = prompt("File path (e.g. src/main.rs)", "src/main.rs");
    if (!path?.trim()) return;
    const created = await createAdminWorkspaceDocument(ws.id, {
      path: path.trim(),
      language: inferLanguageFromPath(path),
      body: "",
      make_primary: false,
    });
    documents = [created, ...documents];
    await openTab(created.id);
  }

  async function createProject() {
    const name = prompt("Project name", "my-project");
    if (!name?.trim()) return;
    const created = await createWorkspace({ name: name.trim(), language: "rust", body: "" });
    workspaces = [created, ...workspaces];
    await loadWorkspaceDetail(created.id);
  }

  async function deleteTab() {
    const tab = activeTab();
    const ws = activeWorkspace;
    if (!tab || !ws || !confirm(`Delete ${tab.path}?`)) return;
    await deleteAdminWorkspaceDocument(ws.id, tab.id);
    documents = documents.filter((d) => d.id !== tab.id);
    tabs = tabs.filter((t) => t.id !== tab.id);
    activeTabId = tabs[0]?.id || null;
  }

  function closeTab(id: string) {
    tabs = tabs.filter((t) => t.id !== id);
    if (activeTabId === id) activeTabId = tabs[0]?.id || null;
  }

  function updateBody(value: string) {
    const id = activeTabId;
    if (!id) return;
    tabs = tabs.map((t) => (t.id === id ? { ...t, body: value, dirty: true } : t));
  }

  onMount(() => void loadWorkspaces());
</script>

<section class="workspace">
  {#if loading}
    <div class="empty-state"><strong>Loading workspace…</strong></div>
  {:else if error}
    <div class="empty-state"><strong>{error}</strong></div>
  {:else}
    <div class="workspace-body">
      <WorkspaceExplorer
        {workspaces}
        workspace={activeWorkspace}
        {documents}
        activeDocumentId={activeTabId}
        onSelectWorkspace={(ws) => void loadWorkspaceDetail(ws.id)}
        onSelectDocument={(id) => void openTab(id)}
        onCreateProject={createProject}
        onCreateFile={createFile}
      />

      <div class="editor-column">
        <div class="tab-bar">
          {#each tabs as tab}
            <button
              type="button"
              class="tab"
              class:active={activeTabId === tab.id}
              on:click={() => (activeTabId = tab.id)}
            >
              {tab.dirty ? "● " : ""}{tab.path.split("/").pop()}
              <span
                role="button"
                tabindex="0"
                class="tab-close"
                on:click|stopPropagation={() => closeTab(tab.id)}
                on:keydown|stopPropagation={(e) => e.key === "Enter" && closeTab(tab.id)}
              >×</span>
            </button>
          {/each}
          <button type="button" class="btn btn-sm btn-ghost" on:click={createFile}>+ File</button>
        </div>

        {#if activeTab()}
          {@const tab = activeTab()!}
          <CodeEditor
            value={tab.body}
            language={tab.language}
            dirty={tab.dirty}
            onChange={updateBody}
            onSave={saveTab}
            onCloseTab={() => activeTabId && closeTab(activeTabId)}
          />
          <div class="editor-actions">
            <button type="button" class="btn btn-sm btn-primary" on:click={saveTab}>Save (Ctrl+S)</button>
            <button type="button" class="btn btn-sm btn-danger" on:click={deleteTab}>Delete file</button>
            <span class="muted">{saveStatus}</span>
          </div>
        {:else}
          <div class="empty-state"><strong>No file open</strong>Create or select a file from the explorer.</div>
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .workspace { height: 100%; min-height: 0; display: flex; flex-direction: column; }
  .workspace-body { flex: 1; min-height: 0; display: grid; grid-template-columns: 260px minmax(0, 1fr); }
  .editor-column { display: flex; flex-direction: column; min-height: 0; padding: 12px; gap: 8px; }
  .tab-bar { display: flex; flex-wrap: wrap; gap: 4px; align-items: center; border-bottom: 1px solid var(--border); padding-bottom: 8px; }
  .tab { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border); border-radius: var(--radius-sm) var(--radius-sm) 0 0; background: var(--bg-card); font-size: 12px; }
  .tab.active { background: var(--brand-dim); border-color: rgba(37,99,235,0.4); }
  .tab-close { opacity: 0.6; padding: 0 4px; }
  .editor-actions { display: flex; gap: 8px; align-items: center; }
  .muted { color: var(--text-muted); font-size: 12px; }
  @media (max-width: 900px) { .workspace-body { grid-template-columns: 1fr; } }
</style>
