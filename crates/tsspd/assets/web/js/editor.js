window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  const LANGUAGE_OPTIONS = [
    "text",
    "markdown",
    "javascript",
    "typescript",
    "python",
    "rust",
    "go",
    "bash",
    "sql",
    "json",
    "yaml",
    "toml",
    "html",
    "css",
  ];

  function languageOptions(selected) {
    return LANGUAGE_OPTIONS.map(
      (language) =>
        `<option value="${language}"${selected === language ? " selected" : ""}>${language}</option>`
    ).join("");
  }

  function setSaveStatus(status) {
    const el = T.$("#editor-save-status");
    if (!el) return;
    if (status === "saved") {
      el.textContent = "Saved";
      el.className = "editor-save-status saved";
    } else if (status === "dirty") {
      el.textContent = "Unsaved changes";
      el.className = "editor-save-status dirty";
    } else if (status === "saving") {
      el.textContent = "Saving…";
      el.className = "editor-save-status saving";
    } else {
      el.textContent = "";
      el.className = "editor-save-status";
    }
  }

  function updateStatusBar() {
    const area = T.$("#editor-area");
    const lineCount = T.$("#editor-line-count");
    const charCount = T.$("#editor-char-count");
    if (!area) return;
    const text = area.value;
    if (lineCount) lineCount.textContent = `${text.split("\n").length} lines`;
    if (charCount) charCount.textContent = `${text.length} chars`;
  }

  function updateDeleteButton() {
    const button = T.$("#editor-delete-btn");
    if (!button) return;
    const disabled = !T.editorCurrentWorkspaceId;
    button.disabled = disabled;
    if (disabled) {
      button.textContent = "Delete";
      return;
    }
    button.textContent = T.editorDocuments.length > 1 ? "Delete file" : "Delete workspace";
  }

  function markDirty() {
    T.editorDirty = true;
    setSaveStatus("dirty");
  }

  function renderWorkspaceList() {
    const list = T.$("#editor-ws-list");
    if (!list) return;
    if (!T.editorWorkspaces.length) {
      list.innerHTML = '<div class="editor-empty-state">No workspaces yet</div>';
      return;
    }
    list.innerHTML = T.editorWorkspaces
      .map(
        (workspace) => `<button
            type="button"
            class="editor-ws-item${T.editorCurrentWorkspaceId === workspace.id ? " active" : ""}"
            data-editor-workspace="${T.escapeHtml(workspace.id)}"
            title="${T.escapeHtml(workspace.name)}">
            <span class="editor-ws-name">${T.escapeHtml(workspace.name)}</span>
            <span class="editor-ws-lang">${T.escapeHtml(workspace.language)} · ${T.escapeHtml(workspace.owner_id)}</span>
          </button>`
      )
      .join("");
  }

  function renderDocumentList() {
    const list = T.$("#editor-doc-list");
    if (!list) return;
    if (!T.editorCurrentWorkspaceId) {
      list.innerHTML = '<div class="editor-empty-state">Open a workspace to load documents.</div>';
      return;
    }
    if (!T.editorDocuments.length) {
      list.innerHTML = '<div class="editor-empty-state">No files yet in this workspace.</div>';
      return;
    }
    list.innerHTML = T.editorDocuments
      .map(
        (document) => `<button
            type="button"
            class="editor-doc-item${T.editorCurrentDocumentId === document.id ? " active" : ""}"
            data-editor-document="${T.escapeHtml(document.id)}">
            <span class="editor-doc-path">${T.escapeHtml(document.path)}</span>
            <span class="editor-doc-meta">${T.escapeHtml(document.language)} · ${T.escapeHtml(T.formatBytes(document.size_bytes || 0))}${document.is_primary ? " · entry" : ""}</span>
          </button>`
      )
      .join("");
  }

  function setTopBar() {
    const workspaceName = T.$("#editor-workspace-name");
    const fileName = T.$("#editor-filename");
    const langBadge = T.$("#editor-lang-badge");
    if (workspaceName) {
      workspaceName.textContent = T.editorCurrentWorkspace ? T.editorCurrentWorkspace.name : "No workspace open";
    }
    if (fileName) {
      fileName.textContent = T.editorCurrentDocument ? T.editorCurrentDocument.path : "No file open";
    }
    if (langBadge) {
      langBadge.textContent = T.editorCurrentDocument ? T.editorCurrentDocument.language : "";
    }
    updateDeleteButton();
  }

  function bindInfoPanelInputs() {
    const pathInput = T.$("#editor-doc-path-input");
    const langSelect = T.$("#editor-doc-language-select");
    const makePrimary = T.$("#editor-make-primary-btn");
    if (pathInput) {
      pathInput.addEventListener("input", () => {
        markDirty();
        const fileName = T.$("#editor-filename");
        if (fileName) fileName.textContent = pathInput.value.trim() || "Untitled file";
      });
    }
    if (langSelect) {
      langSelect.addEventListener("change", () => {
        markDirty();
        const langBadge = T.$("#editor-lang-badge");
        if (langBadge) langBadge.textContent = langSelect.value;
      });
    }
    if (makePrimary) {
      makePrimary.addEventListener("click", () => {
        T.saveEditorDocument(true);
      });
    }
  }

  function setInfoPanel() {
    const el = T.$("#editor-info-panel");
    if (!el) return;
    const workspace = T.editorCurrentWorkspace;
    const document = T.editorCurrentDocument;
    if (!workspace) {
      el.innerHTML = '<p class="muted">Open a workspace to see details.</p>';
      return;
    }

    const documentFields = document
      ? `
        <div class="info-row">
          <span class="info-label">Path</span>
          <input id="editor-doc-path-input" class="editor-text-input mono" type="text" maxlength="160" value="${T.escapeHtml(document.path)}">
        </div>
        <div class="info-row">
          <span class="info-label">Language</span>
          <select id="editor-doc-language-select" class="editor-lang-select">
            ${languageOptions(document.language)}
          </select>
        </div>
        <div class="info-row">
          <span class="info-label">Entry file</span>
          <span class="info-value">${document.is_primary ? "Yes" : "No"}</span>
        </div>
        <div class="editor-info-actions">
          ${document.is_primary ? '<span class="editor-info-chip">Primary document</span>' : '<button type="button" class="btn btn-secondary btn-sm" id="editor-make-primary-btn">Set as entry file</button>'}
        </div>
      `
      : '<p class="muted">Open a document to inspect and edit metadata.</p>';

    el.innerHTML = `
      <div class="info-row"><span class="info-label">Workspace</span><span class="info-value">${T.escapeHtml(workspace.name)}</span></div>
      <div class="info-row"><span class="info-label">Workspace ID</span><span class="info-value mono">${T.escapeHtml(workspace.id)}</span></div>
      <div class="info-row"><span class="info-label">Owner</span><span class="info-value mono">${T.escapeHtml(workspace.owner_id)}</span></div>
      <div class="info-row"><span class="info-label">Updated</span><span class="info-value">${T.escapeHtml(T.formatDate(workspace.updated_at))}</span></div>
      ${documentFields}
    `;
    bindInfoPanelInputs();
  }

  function resetEditor() {
    T.editorCurrentWorkspaceId = null;
    T.editorCurrentWorkspace = null;
    T.editorCurrentDocumentId = null;
    T.editorCurrentDocument = null;
    T.editorDocuments = [];
    T.editorDirty = false;
    const area = T.$("#editor-area");
    if (area) area.value = "";
    renderWorkspaceList();
    renderDocumentList();
    setTopBar();
    setInfoPanel();
    setSaveStatus("");
    updateStatusBar();
  }

  async function confirmDiscard(message) {
    if (!T.editorDirty) return true;
    return confirm(message);
  }

  async function fetchWorkspaceDetail(id) {
    return T.api(`/admin/editor/workspaces/${encodeURIComponent(id)}`);
  }

  T.loadEditorWorkspaces = async function loadEditorWorkspaces() {
    try {
      const data = await T.api("/admin/editor/workspaces");
      T.editorWorkspaces = data.workspaces || [];
      renderWorkspaceList();
      return T.editorWorkspaces;
    } catch (error) {
      const list = T.$("#editor-ws-list");
      if (list) list.innerHTML = `<div class="editor-error">${T.escapeHtml(error.message)}</div>`;
      return [];
    }
  };

  async function loadWorkspaceIntoEditor(id, preferredDocumentId) {
    const detail = await fetchWorkspaceDetail(id);
    T.editorCurrentWorkspaceId = detail.workspace.id;
    T.editorCurrentWorkspace = detail.workspace;
    T.editorDocuments = detail.documents || [];
    renderWorkspaceList();
    renderDocumentList();
    const nextDocument =
      T.editorDocuments.find((item) => item.id === preferredDocumentId) ||
      T.editorDocuments.find((item) => item.is_primary) ||
      T.editorDocuments[0] ||
      null;

    if (!nextDocument) {
      T.editorCurrentDocumentId = null;
      T.editorCurrentDocument = null;
      const area = T.$("#editor-area");
      if (area) area.value = "";
      T.editorDirty = false;
      setTopBar();
      setInfoPanel();
      setSaveStatus("saved");
      updateStatusBar();
      return;
    }
    await T.openEditorDocument(nextDocument.id, true);
  }

  T.openEditorWorkspace = async function openEditorWorkspace(id) {
    if (!(await confirmDiscard("You have unsaved changes. Discard them and open another workspace?"))) {
      return;
    }
    try {
      await loadWorkspaceIntoEditor(id, null);
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.openEditorDocument = async function openEditorDocument(documentId, bypassConfirm) {
    if (!T.editorCurrentWorkspaceId) return;
    if (!bypassConfirm && !(await confirmDiscard("You have unsaved changes. Discard them and open another file?"))) {
      return;
    }
    try {
      const workspaceId = T.editorCurrentWorkspaceId;
      const document = await T.api(
        `/admin/editor/workspaces/${encodeURIComponent(workspaceId)}/documents/${encodeURIComponent(documentId)}`
      );
      T.editorCurrentDocumentId = document.id;
      T.editorCurrentDocument = document;
      T.editorDirty = false;
      const area = T.$("#editor-area");
      if (area) area.value = document.body || "";
      renderDocumentList();
      setTopBar();
      setInfoPanel();
      setSaveStatus("saved");
      updateStatusBar();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  function currentDocumentPayload() {
    const area = T.$("#editor-area");
    return {
      path: T.$("#editor-doc-path-input")?.value.trim() || T.editorCurrentDocument?.path || "untitled.txt",
      language: T.$("#editor-doc-language-select")?.value || T.editorCurrentDocument?.language || "text",
      body: area ? area.value : "",
    };
  }

  T.saveEditorDocument = async function saveEditorDocument(makePrimary) {
    if (!T.editorCurrentWorkspaceId || !T.editorCurrentDocumentId || T.editorSaving) return;
    T.editorSaving = true;
    setSaveStatus("saving");
    try {
      const payload = currentDocumentPayload();
      const updated = await T.api(
        `/admin/editor/workspaces/${encodeURIComponent(T.editorCurrentWorkspaceId)}/documents/${encodeURIComponent(T.editorCurrentDocumentId)}`,
        {
          method: "PUT",
          body: JSON.stringify({
            ...payload,
            make_primary: Boolean(makePrimary),
          }),
        }
      );
      T.editorDirty = false;
      T.editorCurrentDocument = updated;
      await T.loadEditorWorkspaces();
      await loadWorkspaceIntoEditor(T.editorCurrentWorkspaceId, updated.id);
      setSaveStatus("saved");
      if (makePrimary) T.showBanner("Entry file updated", "success");
    } catch (error) {
      T.showBanner(error.message, "error");
      setSaveStatus("dirty");
    } finally {
      T.editorSaving = false;
    }
  };

  T.deleteEditorTarget = async function deleteEditorTarget() {
    if (!T.editorCurrentWorkspaceId) return;
    try {
      if (T.editorDocuments.length > 1 && T.editorCurrentDocumentId) {
        if (!confirm("Delete the current file?")) return;
        await T.api(
          `/admin/editor/workspaces/${encodeURIComponent(T.editorCurrentWorkspaceId)}/documents/${encodeURIComponent(T.editorCurrentDocumentId)}`,
          { method: "DELETE" }
        );
        T.showBanner("File deleted", "success");
        await loadWorkspaceIntoEditor(T.editorCurrentWorkspaceId, null);
      } else {
        if (!confirm("Delete this workspace and its remaining file?")) return;
        const removedId = T.editorCurrentWorkspaceId;
        await T.api(`/workspaces/${encodeURIComponent(removedId)}`, { method: "DELETE" });
        T.showBanner("Workspace deleted", "success");
        await T.loadEditorWorkspaces();
        const nextWorkspace = T.editorWorkspaces.find((item) => item.id !== removedId);
        if (nextWorkspace) {
          await loadWorkspaceIntoEditor(nextWorkspace.id, null);
        } else {
          resetEditor();
        }
      }
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.openEditorDocumentDialog = function openEditorDocumentDialog() {
    if (!T.editorCurrentWorkspaceId) {
      T.showBanner("Open a workspace first", "error");
      return;
    }
    T.$("#editor-document-dialog-title").textContent = "New file";
    T.$("#editor-document-path-input").value = "";
    T.$("#editor-document-language-input").value = "text";
    T.$("#editor-document-primary-input").checked = false;
    T.$("#editor-document-dialog").showModal();
  };

  T.createEditorDocument = async function createEditorDocument() {
    if (!T.editorCurrentWorkspaceId) return;
    const payload = {
      path: T.$("#editor-document-path-input").value.trim(),
      language: T.$("#editor-document-language-input").value.trim() || "text",
      body: "",
      make_primary: T.$("#editor-document-primary-input").checked,
    };
    if (!payload.path) {
      T.showBanner("File path is required", "error");
      return;
    }
    try {
      const document = await T.api(
        `/admin/editor/workspaces/${encodeURIComponent(T.editorCurrentWorkspaceId)}/documents`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        }
      );
      T.$("#editor-document-dialog").close();
      await T.loadEditorWorkspaces();
      await loadWorkspaceIntoEditor(T.editorCurrentWorkspaceId, document.id);
      T.showBanner("File created", "success");
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.editCurrentWorkspace = function editCurrentWorkspace() {
    if (!T.editorCurrentWorkspace) {
      T.showBanner("Open a workspace first", "error");
      return;
    }
    T.openWorkspaceDialog(T.editorCurrentWorkspace, { source: "editor" });
  };

  T.loadEditor = async function loadEditor() {
    const workspaces = await T.loadEditorWorkspaces();
    updateStatusBar();
    if (!workspaces.length) {
      resetEditor();
      return;
    }
    if (T.editorCurrentWorkspaceId) {
      const current = workspaces.find((item) => item.id === T.editorCurrentWorkspaceId);
      if (current) {
        await loadWorkspaceIntoEditor(current.id, T.editorCurrentDocumentId);
        return;
      }
    }
    await loadWorkspaceIntoEditor(workspaces[0].id, null);
  };

  T.bindEditorEvents = function bindEditorEvents() {
    const area = T.$("#editor-area");
    if (!area) return;

    area.addEventListener("keydown", (event) => {
      if (event.key === "Tab") {
        event.preventDefault();
        const start = area.selectionStart;
        const end = area.selectionEnd;
        area.value = `${area.value.slice(0, start)}  ${area.value.slice(end)}`;
        area.selectionStart = area.selectionEnd = start + 2;
        markDirty();
      }
      if ((event.ctrlKey || event.metaKey) && event.key === "s") {
        event.preventDefault();
        T.saveEditorDocument(false);
      }
    });

    area.addEventListener("input", () => {
      markDirty();
      updateStatusBar();
    });

    T.$("#editor-save-btn")?.addEventListener("click", () => T.saveEditorDocument(false));
    T.$("#editor-new-btn")?.addEventListener("click", () => T.openWorkspaceDialog(null, { source: "editor" }));
    T.$("#editor-edit-workspace-btn")?.addEventListener("click", () => T.editCurrentWorkspace());
    T.$("#editor-new-document-btn")?.addEventListener("click", () => T.openEditorDocumentDialog());
    T.$("#editor-delete-btn")?.addEventListener("click", () => T.deleteEditorTarget());
    T.$("#editor-document-form")?.addEventListener("submit", (event) => {
      event.preventDefault();
      T.createEditorDocument();
    });
    T.$("#editor-document-cancel")?.addEventListener("click", () => T.$("#editor-document-dialog")?.close());
    T.$("#editor-document-close")?.addEventListener("click", () => T.$("#editor-document-dialog")?.close());

    document.addEventListener("click", (event) => {
      const workspaceButton = event.target.closest("[data-editor-workspace]");
      if (workspaceButton && !T.$("#view-editor")?.classList.contains("hidden")) {
        T.openEditorWorkspace(workspaceButton.dataset.editorWorkspace);
        return;
      }
      const documentButton = event.target.closest("[data-editor-document]");
      if (documentButton && !T.$("#view-editor")?.classList.contains("hidden")) {
        T.openEditorDocument(documentButton.dataset.editorDocument);
      }
    });
  };
})(window.Tssp);
