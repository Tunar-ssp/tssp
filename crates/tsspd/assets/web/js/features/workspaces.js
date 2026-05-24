window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

T.allWorkspaces = [];

  function renderWorkspaceCards() {
    const container = T.$("#workspaces-grid");
    if (!container) return;
    const query = (T.$("#workspaces-search")?.value || "").toLowerCase().trim();
    const lang = (T.$("#workspaces-lang-filter")?.value || "").toLowerCase();
    let items = T.allWorkspaces.slice();
    if (lang) items = items.filter((w) => (w.language || "").toLowerCase() === lang);
    if (query) items = items.filter((w) =>
      (w.name || "").toLowerCase().includes(query) || (w.body || "").toLowerCase().includes(query)
    );
    if (!items.length) {
      container.innerHTML = T.allWorkspaces.length === 0
        ? '<div class="notes-empty-state">No workspaces yet. Create one to store scripts and text files.</div>'
        : '<div class="notes-empty-state">No workspaces match your filters.</div>';
      return;
    }
    container.innerHTML = `<div class="workspace-cards">${items
      .map((workspace) => {
        const id = T.escapeHtml(workspace.id);
        const lineCount = (workspace.body || "").split("\n").length;
        const preview = (workspace.body || "").trim().slice(0, 160);
        return `<article class="workspace-card">
          <div class="workspace-card-head">
            <div class="workspace-card-title-row">
              <strong class="workspace-card-name">${T.escapeHtml(workspace.name)}</strong>
              <span class="type-pill">${T.escapeHtml(workspace.language)}</span>
            </div>
            <div class="workspace-card-meta">${lineCount} lines · Updated ${T.escapeHtml(T.formatDate(workspace.updated_at))}</div>
          </div>
          ${preview ? `<pre class="workspace-card-preview">${T.escapeHtml(preview)}</pre>` : ""}
          <div class="workspace-card-actions">
            <button type="button" class="btn btn-secondary btn-sm" data-ws-edit="${id}">Edit</button>
            <button type="button" class="btn btn-text btn-sm btn-danger" data-ws-del="${id}">Delete</button>
          </div>
        </article>`;
      })
      .join("")}</div>`;
  }

  T.loadWorkspaces = async function loadWorkspaces() {
    const container = T.$("#workspaces-grid");
    if (!container) return;
    container.innerHTML = '<div class="notes-loading">Loading workspaces…</div>';
    try {
      const data = await T.api("/workspaces");
      T.allWorkspaces = data.workspaces || [];
      renderWorkspaceCards();
      const searchEl = T.$("#workspaces-search");
      const langEl = T.$("#workspaces-lang-filter");
      if (searchEl && !searchEl.dataset.bound) {
        searchEl.dataset.bound = "1";
        searchEl.addEventListener("input", renderWorkspaceCards);
      }
      if (langEl && !langEl.dataset.bound) {
        langEl.dataset.bound = "1";
        langEl.addEventListener("change", renderWorkspaceCards);
      }
    } catch (error) {
      container.innerHTML = `<div class="notes-empty-state">${T.escapeHtml(error.message)}</div>`;
    }
  };

  T.openWorkspaceDialog = function openWorkspaceDialog(workspace, options) {
    T.workspaceDialogSource = options?.source || "workspaces";
    T.editingWorkspaceId = workspace ? workspace.id : null;
    T.$("#workspace-dialog-title").textContent = workspace ? "Edit workspace" : "New workspace";
    T.$("#workspace-name-input").value = workspace ? workspace.name || "" : "";
    const langEl = T.$("#workspace-language-input");
    if (langEl) langEl.value = workspace ? workspace.language || "text" : "text";
    T.$("#workspace-body-input").value = workspace ? workspace.body || "" : "";
    T.$("#workspace-dialog").showModal();
  };

  T.saveWorkspace = async function saveWorkspace() {
    const payload = {
      name: T.$("#workspace-name-input").value.trim(),
      language: (T.$("#workspace-language-input").value || "text").trim(),
      body: T.$("#workspace-body-input").value,
    };
    if (!payload.name) {
      T.showBanner("Workspace name is required", "error");
      return;
    }
    try {
      let savedId = T.editingWorkspaceId;
      if (savedId) {
        await T.api("/workspaces/" + encodeURIComponent(savedId), {
          method: "PUT",
          body: JSON.stringify(payload),
        });
      } else {
        const created = await T.api("/workspaces", { method: "POST", body: JSON.stringify(payload) });
        savedId = created.id;
      }
      T.$("#workspace-dialog").close();
      T.showBanner("Workspace saved", "success");
      T.loadWorkspaces();
      if (typeof T.loadEditorWorkspaces === "function") {
        await T.loadEditorWorkspaces();
      }
      if (T.workspaceDialogSource === "editor" && savedId && typeof T.openEditorWorkspace === "function") {
        await T.openEditorWorkspace(savedId);
      }
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.openWorkspace = async function openWorkspace(id) {
    try {
      const workspace = await T.api("/workspaces/" + encodeURIComponent(id));
      T.openWorkspaceDialog(workspace);
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.deleteWorkspace = async function deleteWorkspace(id) {
    if (!confirm("Delete this workspace?")) return;
    try {
      await T.api("/workspaces/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("Workspace deleted", "success");
      T.loadWorkspaces();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

})(window.Tssp);
