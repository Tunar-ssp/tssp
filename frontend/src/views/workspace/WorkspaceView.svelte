<script lang="ts">
  import { onMount } from "svelte";
  import {
    createAdminWorkspaceDocument,
    deleteAdminWorkspaceDocument,
    getAdminWorkspaceDetail,
    getAdminWorkspaceDocument,
    getEditorExecutionState,
    listWorkspaces,
    updateAdminWorkspaceDocument,
    type WorkspaceDocumentRecord,
    type WorkspaceRecord,
  } from "../../lib/api";
  import { formatRelativeDate } from "../../lib/utils/format";
  import WorkspaceEditor from "./WorkspaceEditor.svelte";
  import WorkspaceExplorer from "./WorkspaceExplorer.svelte";

  let workspaces: WorkspaceRecord[] = [];
  let activeWorkspace: WorkspaceRecord | null = null;
  let documents: WorkspaceDocumentRecord[] = [];
  let activeDocumentId: string | null = null;
  let loading = true;
  let error = "";
  let saveStatus = "";
  let executionMessage = "";
  let bodyDraft = "";

  async function loadWorkspaceDetail(workspaceId: string) {
    const detail = await getAdminWorkspaceDetail(workspaceId);
    activeWorkspace = detail.workspace;
    documents = detail.documents.map((document) => ({
      ...document,
      body: "",
      created_at: detail.workspace.created_at,
    }));
    activeDocumentId = detail.documents.find((document) => document.is_primary)?.id || detail.documents[0]?.id || null;
    if (activeDocumentId) {
      const document = await getAdminWorkspaceDocument(workspaceId, activeDocumentId);
      documents = documents.map((item) => (item.id === document.id ? document : item));
      bodyDraft = document.body;
    }
  }

  async function loadWorkspaces() {
    loading = true;
    error = "";
    try {
      const [workspaceResponse, executionCheck] = await Promise.all([
        listWorkspaces(),
        getEditorExecutionState(),
      ]);
      workspaces = workspaceResponse.workspaces || [];
      executionMessage = executionCheck.message;
      if (workspaces[0]) {
        await loadWorkspaceDetail(workspaces[0].id);
      }
    } catch (nextError) {
      error = nextError instanceof Error ? nextError.message : "Failed to load workspaces";
    } finally {
      loading = false;
    }
  }

  function activeDocument() {
    return documents.find((document) => document.id === activeDocumentId) || null;
  }

  async function selectWorkspace(workspace: WorkspaceRecord) {
    try {
      await loadWorkspaceDetail(workspace.id);
      saveStatus = `Loaded ${workspace.name}`;
    } catch (nextError) {
      saveStatus = nextError instanceof Error ? nextError.message : "Failed to load workspace";
    }
  }

  async function selectDocument(documentId: string) {
    if (!activeWorkspace) return;
    activeDocumentId = documentId;
    const document = await getAdminWorkspaceDocument(activeWorkspace.id, documentId);
    documents = documents.map((item) => (item.id === document.id ? document : item));
    bodyDraft = document.body;
  }

  async function saveDocument() {
    const workspace = activeWorkspace;
    const document = activeDocument();
    if (!workspace || !document) return;
    try {
      const updated = await updateAdminWorkspaceDocument(workspace.id, document.id, {
        path: document.path,
        language: document.language,
        body: bodyDraft,
        make_primary: document.is_primary,
      });
      documents = documents.map((item) => (item.id === updated.id ? updated : item));
      bodyDraft = updated.body;
      saveStatus = `Saved ${formatRelativeDate(updated.updated_at)}`;
    } catch (nextError) {
      saveStatus = nextError instanceof Error ? nextError.message : "Save failed";
    }
  }

  async function createDocument() {
    const workspace = activeWorkspace;
    if (!workspace) return;
    try {
      const created = await createAdminWorkspaceDocument(workspace.id, {
        path: `notes/${workspace.name.toLowerCase().replace(/\s+/g, "-")}.md`,
        language: workspace.language,
        body: "# Workspace note\n\nStart here.",
        make_primary: false,
      });
      documents = [created, ...documents];
      activeDocumentId = created.id;
      bodyDraft = created.body;
      saveStatus = "Document created";
    } catch (nextError) {
      saveStatus = nextError instanceof Error ? nextError.message : "Create failed";
    }
  }

  async function deleteDocument() {
    const workspace = activeWorkspace;
    const document = activeDocument();
    if (!workspace || !document) return;
    try {
      await deleteAdminWorkspaceDocument(workspace.id, document.id);
      documents = documents.filter((item) => item.id !== document.id);
      const nextDocument = documents[0] || null;
      activeDocumentId = nextDocument?.id || null;
      bodyDraft = nextDocument?.body || "";
      saveStatus = "Document deleted";
    } catch (nextError) {
      saveStatus = nextError instanceof Error ? nextError.message : "Delete failed";
    }
  }

  onMount(() => {
    void loadWorkspaces();
  });

  $: activeWorkspaceLanguage = activeWorkspace?.language || "text";
</script>

<section class="ide-scaffold">
  <WorkspaceExplorer
    workspace={activeWorkspace}
    documents={documents}
    activeDocumentId={activeDocumentId}
    onSelectWorkspace={(workspace) => void selectWorkspace(workspace)}
    onSelectDocument={(documentId) => void selectDocument(documentId)}
  />

  <div class="ide-main">
    <div class="ide-tabs">
      {#each documents.slice(0, 3) as document}
        <button
          type="button"
          class:active={activeDocumentId === document.id}
          class="ide-tab"
          on:click={() => void selectDocument(document.id)}
        >
          {document.path}
        </button>
      {/each}
    </div>

    <WorkspaceEditor
      workspace={activeWorkspace}
      document={activeDocument()}
      bodyDraft={bodyDraft}
      saveStatus={saveStatus || executionMessage}
      onBodyChange={(value) => (bodyDraft = value)}
      onSave={saveDocument}
      onCreateDocument={createDocument}
      onDeleteDocument={deleteDocument}
    />
  </div>

  <aside class="ide-panel">
    <header class="panel-head">
      <strong>Workspace metadata</strong>
      <span>project health</span>
    </header>
    <div class="detail-stack">
      <div class="detail-row"><span>Project</span><strong>{activeWorkspace?.name || "No project open"}</strong></div>
      <div class="detail-row"><span>Workspaces</span><strong>{workspaces.length}</strong></div>
      <div class="detail-row"><span>Language</span><strong>{activeWorkspaceLanguage}</strong></div>
      <div class="detail-row"><span>Execution</span><strong>{executionMessage || "Disabled until sandbox"}</strong></div>
    </div>
  </aside>
</section>
