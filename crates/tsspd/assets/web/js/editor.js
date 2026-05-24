window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  // Editor state
  T.editorCurrentId = null;
  T.editorDirty = false;
  T.editorSaving = false;
  T._editorWorkspaces = [];

  // --- Sidebar ---

  function renderSidebar(workspaces) {
    const list = T.$("#editor-ws-list");
    if (!list) return;
    if (!workspaces.length) {
      list.innerHTML = '<div class="editor-empty-state">No workspaces yet</div>';
      return;
    }
    list.innerHTML = workspaces
      .map(
        (w) =>
          `<button type="button" class="editor-ws-item${T.editorCurrentId === w.id ? " active" : ""}"
            data-ws-id="${T.escapeHtml(w.id)}"
            title="${T.escapeHtml(w.name)}">
            <span class="editor-ws-name">${T.escapeHtml(w.name)}</span>
            <span class="editor-ws-lang">${T.escapeHtml(w.language)}</span>
          </button>`
      )
      .join("");
  }

  // --- Top bar ---

  function setTopBar(ws) {
    const nameEl = T.$("#editor-filename");
    const langEl = T.$("#editor-lang-badge");
    if (nameEl) nameEl.textContent = ws ? ws.name : "No workspace open";
    if (langEl) langEl.textContent = ws ? ws.language : "";
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

  // --- Info panel ---

  function setInfoPanel(ws) {
    const el = T.$("#editor-info-panel");
    if (!el) return;
    if (!ws) {
      el.innerHTML = '<p class="muted">Open a workspace to see details.</p>';
      return;
    }
    el.innerHTML = `
      <div class="info-row"><span class="info-label">ID</span><span class="info-value mono">${T.escapeHtml(ws.id)}</span></div>
      <div class="info-row"><span class="info-label">Owner</span><span class="info-value mono">${T.escapeHtml(ws.owner_id)}</span></div>
      <div class="info-row"><span class="info-label">Language</span>
        <select id="editor-lang-select" class="editor-lang-select">
          ${["text","javascript","typescript","python","rust","go","bash","sql","json","yaml","toml","markdown","html","css"]
            .map((l) => `<option value="${l}"${ws.language === l ? " selected" : ""}>${l}</option>`)
            .join("")}
        </select>
      </div>
      <div class="info-row"><span class="info-label">Created</span><span class="info-value">${T.escapeHtml(T.formatDate(ws.created_at))}</span></div>
      <div class="info-row"><span class="info-label">Updated</span><span class="info-value">${T.escapeHtml(T.formatDate(ws.updated_at))}</span></div>
    `;
    const langSelect = T.$("#editor-lang-select");
    if (langSelect) {
      langSelect.addEventListener("change", () => {
        T.editorDirty = true;
        setSaveStatus("dirty");
        const langBadge = T.$("#editor-lang-badge");
        if (langBadge) langBadge.textContent = langSelect.value;
      });
    }
  }

  // --- Status bar ---

  function updateStatusBar() {
    const area = T.$("#editor-area");
    const lineCount = T.$("#editor-line-count");
    const charCount = T.$("#editor-char-count");
    if (!area) return;
    const text = area.value;
    if (lineCount) lineCount.textContent = `${text.split("\n").length} lines`;
    if (charCount) charCount.textContent = `${text.length} chars`;
  }

  // --- Load workspaces into sidebar ---

  T.loadEditorWorkspaces = async function loadEditorWorkspaces() {
    try {
      const data = await T.api("/admin/editor/workspaces");
      T._editorWorkspaces = data.workspaces || [];
      renderSidebar(T._editorWorkspaces);
    } catch (err) {
      const list = T.$("#editor-ws-list");
      if (list) list.innerHTML = `<div class="editor-error">${T.escapeHtml(err.message)}</div>`;
    }
  };

  // --- Open a workspace ---

  T.openEditorWorkspace = async function openEditorWorkspace(id) {
    if (T.editorDirty) {
      if (!confirm("You have unsaved changes. Discard and open another workspace?")) return;
    }
    try {
      const ws = await T.api(`/admin/editor/workspaces/${encodeURIComponent(id)}`);
      T.editorCurrentId = ws.id;
      T.editorDirty = false;
      const area = T.$("#editor-area");
      if (area) area.value = ws.body || "";
      setTopBar(ws);
      setInfoPanel(ws);
      setSaveStatus("saved");
      updateStatusBar();
      renderSidebar(T._editorWorkspaces);
    } catch (err) {
      T.showBanner(err.message, "error");
    }
  };

  // --- Save current workspace ---

  T.saveEditorWorkspace = async function saveEditorWorkspace() {
    if (!T.editorCurrentId || T.editorSaving) return;
    const area = T.$("#editor-area");
    const langSelect = T.$("#editor-lang-select");
    const nameEl = T.$("#editor-filename");
    if (!area) return;

    T.editorSaving = true;
    setSaveStatus("saving");
    try {
      const body = area.value;
      const language = langSelect ? langSelect.value : "text";
      const name = nameEl ? nameEl.textContent.trim() : "untitled";
      const updated = await T.api(`/workspaces/${encodeURIComponent(T.editorCurrentId)}`, {
        method: "PUT",
        body: JSON.stringify({ name, language, body }),
      });
      T.editorDirty = false;
      setSaveStatus("saved");
      setTopBar(updated);
      setInfoPanel(updated);
      // Refresh sidebar list
      await T.loadEditorWorkspaces();
    } catch (err) {
      T.showBanner(err.message, "error");
      setSaveStatus("dirty");
    } finally {
      T.editorSaving = false;
    }
  };

  // --- Create new workspace ---

  T.newEditorWorkspace = async function newEditorWorkspace() {
    const rawName = prompt("Workspace name:");
    if (!rawName || !rawName.trim()) return;
    const language = prompt("Language (e.g. javascript, python, text):", "text") || "text";
    try {
      const created = await T.api("/workspaces", {
        method: "POST",
        body: JSON.stringify({ name: rawName.trim(), language: language.trim(), body: "" }),
      });
      await T.loadEditorWorkspaces();
      T.openEditorWorkspace(created.id);
    } catch (err) {
      T.showBanner(err.message, "error");
    }
  };

  // --- Delete current workspace ---

  T.deleteEditorWorkspace = async function deleteEditorWorkspace() {
    if (!T.editorCurrentId) return;
    if (!confirm("Delete this workspace? This cannot be undone.")) return;
    try {
      await T.api(`/workspaces/${encodeURIComponent(T.editorCurrentId)}`, { method: "DELETE" });
      T.editorCurrentId = null;
      T.editorDirty = false;
      const area = T.$("#editor-area");
      if (area) area.value = "";
      setTopBar(null);
      setInfoPanel(null);
      setSaveStatus("");
      updateStatusBar();
      await T.loadEditorWorkspaces();
    } catch (err) {
      T.showBanner(err.message, "error");
    }
  };

  // --- Bind editor events ---

  T.bindEditorEvents = function bindEditorEvents() {
    const area = T.$("#editor-area");
    if (!area) return;

    // Tab key inserts 2 spaces
    area.addEventListener("keydown", (ev) => {
      if (ev.key === "Tab") {
        ev.preventDefault();
        const start = area.selectionStart;
        const end = area.selectionEnd;
        area.value = area.value.slice(0, start) + "  " + area.value.slice(end);
        area.selectionStart = area.selectionEnd = start + 2;
        T.editorDirty = true;
        setSaveStatus("dirty");
      }
      // Ctrl+S / Cmd+S to save
      if ((ev.ctrlKey || ev.metaKey) && ev.key === "s") {
        ev.preventDefault();
        T.saveEditorWorkspace();
      }
    });

    area.addEventListener("input", () => {
      T.editorDirty = true;
      setSaveStatus("dirty");
      updateStatusBar();
    });

    T.$("#editor-save-btn")?.addEventListener("click", () => T.saveEditorWorkspace());
    T.$("#editor-new-btn")?.addEventListener("click", () => T.newEditorWorkspace());
    T.$("#editor-delete-btn")?.addEventListener("click", () => T.deleteEditorWorkspace());

    document.addEventListener("click", (ev) => {
      const wsBtn = ev.target.closest("[data-ws-id]");
      if (wsBtn && T.$(`#view-editor`) && !T.$(`#view-editor`).classList.contains("hidden")) {
        T.openEditorWorkspace(wsBtn.dataset.wsId);
      }
    });
  };

  // --- Load editor view ---

  T.loadEditor = async function loadEditor() {
    await T.loadEditorWorkspaces();
    updateStatusBar();
  };
})(window.Tssp);
