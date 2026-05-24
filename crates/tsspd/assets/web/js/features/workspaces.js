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
      if (T.allWorkspaces.length === 0) {
        container.innerHTML = `<div class="notes-empty-hero">
          <div class="notes-empty-icon">⌨️</div>
          <div class="notes-empty-title">No workspaces yet</div>
          <div class="notes-empty-sub">Create a workspace to store scripts, configs, and text files with syntax highlighting.</div>
          <button type="button" class="btn btn-primary" id="ws-empty-cta">New workspace</button>
        </div>`;
        container.querySelector("#ws-empty-cta")?.addEventListener("click", () => T.openWorkspaceDialog(null));
      } else {
        container.innerHTML = '<div class="notes-empty-state">No workspaces match your filters.</div>';
      }
      return;
    }
    const langColors = {
      rust: "var(--orange)", python: "var(--blue)", js: "var(--yellow)",
      javascript: "var(--yellow)", typescript: "var(--blue)", json: "var(--cyan)",
      yaml: "var(--green)", toml: "var(--orange)", bash: "var(--green)",
      sh: "var(--green)", html: "var(--red)", css: "var(--violet)",
      sql: "var(--cyan)", markdown: "var(--text-muted)", md: "var(--text-muted)",
    };
    container.innerHTML = `<div class="workspace-cards">${items
      .map((workspace) => {
        const id = T.escapeHtml(workspace.id);
        const lang = (workspace.language || "text").toLowerCase();
        const lineCount = (workspace.body || "").split("\n").filter(Boolean).length;
        const charCount = (workspace.body || "").length;
        const preview = (workspace.body || "").trim().split("\n").slice(0, 3).join("\n");
        const langColor = langColors[lang] || "var(--text-dim)";
        return `<article class="workspace-card">
          <div class="workspace-card-head">
            <div class="workspace-card-title-row">
              <strong class="workspace-card-name">${T.escapeHtml(workspace.name)}</strong>
              <span class="workspace-lang-badge" style="color:${langColor}">${T.escapeHtml(workspace.language || "text")}</span>
            </div>
            <div class="workspace-card-meta">${lineCount} lines · ${charCount} chars · <span title="${T.escapeHtml(T.formatDate(workspace.updated_at))}">${T.escapeHtml(T.formatRelativeTime(workspace.updated_at) || T.formatDate(workspace.updated_at))}</span></div>
          </div>
          ${preview ? `<pre class="workspace-card-preview">${T.escapeHtml(preview)}</pre>` : '<div class="workspace-card-empty">Empty file</div>'}
          <div class="workspace-card-actions">
            <button type="button" class="btn btn-primary btn-sm" data-ws-open-editor="${id}">Open in editor</button>
            <button type="button" class="btn btn-secondary btn-sm" data-ws-edit="${id}">Settings</button>
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
