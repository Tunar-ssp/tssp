window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  // ── IDE state ─────────────────────────────────────────────────────────────

  let allFiles = [];       // all workspaces from server
  let openTabs = [];       // [{id, name, language, body, dirty, origBody}]
  let activeId = null;     // currently focused tab id
  let saveTimer = null;

  // ── Boot / load ───────────────────────────────────────────────────────────

  T.loadWorkspaces = async function loadWorkspaces() {
    const list = T.$("#ide-file-list");
    if (!list) return;
    list.innerHTML = '<div class="ide-file-loading">Loading…</div>';
    try {
      const data = await T.api("/workspaces");
      allFiles = data.workspaces || [];
      renderFileList();
      // Re-bind search
      const s = T.$("#workspaces-search");
      if (s && !s.dataset.bound) {
        s.dataset.bound = "1";
        s.addEventListener("input", renderFileList);
      }
    } catch (e) {
      if (list) list.innerHTML = `<div class="ide-file-error">${T.escapeHtml(e.message)}</div>`;
    }
  };

  // ── File tree ─────────────────────────────────────────────────────────────

  const langIcons = {
    rust: "rs", python: "py", javascript: "js", typescript: "ts",
    json: "{}",  yaml: "yml", toml: "tom", bash: "sh", sh: "sh",
    html: "htm", css: "css", sql: "sql", markdown: "md", md: "md",
    text: "txt",
  };
  const langColors = {
    rust: "var(--orange)", python: "var(--blue)", javascript: "var(--yellow)",
    typescript: "var(--blue)", json: "var(--cyan)", yaml: "var(--green)",
    toml: "var(--orange)", bash: "var(--green)", sh: "var(--green)",
    html: "var(--red)", css: "var(--violet)", sql: "var(--cyan)",
    markdown: "var(--text-muted)", md: "var(--text-muted)", text: "var(--text-dim)",
  };

  function renderFileList() {
    const container = T.$("#ide-file-list");
    if (!container) return;
    const q = (T.$("#workspaces-search")?.value || "").toLowerCase();
    const items = q ? allFiles.filter(f =>
      (f.name || "").toLowerCase().includes(q) ||
      (f.language || "").toLowerCase().includes(q)
    ) : allFiles;

    if (!items.length) {
      container.innerHTML = allFiles.length === 0
        ? `<div class="ide-file-empty">No files yet.<br>Press ＋ to create one.</div>`
        : `<div class="ide-file-empty">No matches.</div>`;
      return;
    }

    container.innerHTML = items.map(f => {
      const lang = (f.language || "text").toLowerCase();
      const icon = langIcons[lang] || "txt";
      const color = langColors[lang] || "var(--text-dim)";
      const isActive = f.id === activeId;
      const tab = openTabs.find(t => t.id === f.id);
      const dirty = tab?.dirty ? ' ide-file-dirty' : '';
      return `<button type="button" class="ide-file-item${isActive ? " active" : ""}${dirty}" data-ide-open="${T.escapeHtml(f.id)}" title="${T.escapeHtml(f.name)}">
        <span class="ide-file-icon" style="color:${color}">${icon}</span>
        <span class="ide-file-name">${T.escapeHtml(f.name)}</span>
        <span class="ide-file-actions-inline">
          <span class="ide-file-del" data-ide-del="${T.escapeHtml(f.id)}" title="Delete">×</span>
        </span>
      </button>`;
    }).join("");
  }

  // ── Tab management ────────────────────────────────────────────────────────

  function renderTabs() {
    const bar = T.$("#ide-tabs");
    if (!bar) return;
    if (!openTabs.length) {
      bar.innerHTML = '<span class="ide-tabs-empty">No files open</span>';
      return;
    }
    bar.innerHTML = openTabs.map(tab => {
      const dirty = tab.dirty ? '<span class="ide-tab-dot"></span>' : '';
      return `<button type="button" class="ide-tab${tab.id === activeId ? " active" : ""}" data-ide-tab="${T.escapeHtml(tab.id)}">
        ${T.escapeHtml(tab.name)}${dirty}
        <span class="ide-tab-close" data-ide-close="${T.escapeHtml(tab.id)}" title="Close">×</span>
      </button>`;
    }).join("");
  }

  function openFile(id) {
    const file = allFiles.find(f => f.id === id);
    if (!file) return;

    // If already open, just switch to it
    if (!openTabs.find(t => t.id === id)) {
      openTabs.push({ id: file.id, name: file.name, language: file.language || "text", body: file.body || "", origBody: file.body || "", dirty: false });
    }
    activeId = id;
    renderTabs();
    renderFileList();
    showEditor(id);
  }

  function closeTab(id) {
    const tab = openTabs.find(t => t.id === id);
    if (tab?.dirty) {
      if (!confirm(`"${tab.name}" has unsaved changes. Discard?`)) return;
    }
    openTabs = openTabs.filter(t => t.id !== id);
    if (activeId === id) {
      activeId = openTabs.length ? openTabs[openTabs.length - 1].id : null;
    }
    renderTabs();
    renderFileList();
    if (activeId) showEditor(activeId);
    else showWelcome();
  }

  function showEditor(id) {
    const tab = openTabs.find(t => t.id === id);
    if (!tab) return;
    const welcome = T.$("#ide-welcome");
    const editor = T.$("#ide-editor");
    const textarea = T.$("#ide-textarea");
    if (!textarea) return;
    welcome?.classList.add("hidden");
    editor?.classList.remove("hidden");
    textarea.value = tab.body;
    updateStatusBar(tab);
    textarea.focus();
  }

  function showWelcome() {
    T.$("#ide-welcome")?.classList.remove("hidden");
    T.$("#ide-editor")?.classList.add("hidden");
  }

  function updateStatusBar(tab) {
    if (!tab) return;
    const langEl = T.$("#ide-lang-badge");
    const posEl = T.$("#ide-cursor-pos");
    const charEl = T.$("#ide-char-count");
    const savedEl = T.$("#ide-save-status");
    if (langEl) langEl.textContent = (tab.language || "text");
    if (charEl) charEl.textContent = `${(tab.body || "").length} chars`;
    if (savedEl) savedEl.textContent = tab.dirty ? "Unsaved changes" : "All saved";
    if (posEl) posEl.textContent = "Ln 1, Col 1";
  }

  function updateCursorPos(textarea) {
    const posEl = T.$("#ide-cursor-pos");
    if (!posEl) return;
    const val = textarea.value.substring(0, textarea.selectionStart);
    const lines = val.split("\n");
    posEl.textContent = `Ln ${lines.length}, Col ${lines[lines.length - 1].length + 1}`;
  }

  // ── Save ──────────────────────────────────────────────────────────────────

  async function saveTab(id) {
    const tab = openTabs.find(t => t.id === id);
    if (!tab || !tab.dirty) return;
    const savedEl = T.$("#ide-save-status");
    if (savedEl) savedEl.textContent = "Saving…";
    try {
      await T.api("/workspaces/" + encodeURIComponent(id), {
        method: "PUT",
        body: JSON.stringify({ name: tab.name, language: tab.language, body: tab.body }),
      });
      tab.origBody = tab.body;
      tab.dirty = false;
      renderTabs();
      renderFileList();
      if (savedEl) savedEl.textContent = "All saved";
    } catch (e) {
      if (savedEl) savedEl.textContent = "Save failed";
      T.showBanner(e.message, "error");
    }
  }

  // ── Create / delete ───────────────────────────────────────────────────────

  async function createNewFile() {
    const name = prompt("File name (e.g. notes.md, script.py):");
    if (!name || !name.trim()) return;
    const trimmed = name.trim();
    const ext = trimmed.split(".").pop().toLowerCase();
    const extToLang = { md: "markdown", py: "python", js: "javascript", ts: "typescript",
      rs: "rust", sh: "bash", yml: "yaml", yaml: "yaml", toml: "toml", json: "json",
      html: "html", css: "css", sql: "sql", txt: "text" };
    const language = extToLang[ext] || "text";
    try {
      const created = await T.api("/workspaces", {
        method: "POST",
        body: JSON.stringify({ name: trimmed, language, body: "" }),
      });
      await T.loadWorkspaces();
      openFile(created.id);
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  }

  async function deleteFile(id) {
    const file = allFiles.find(f => f.id === id);
    if (!confirm(`Delete "${file?.name || id}"?`)) return;
    try {
      await T.api("/workspaces/" + encodeURIComponent(id), { method: "DELETE" });
      openTabs = openTabs.filter(t => t.id !== id);
      if (activeId === id) {
        activeId = openTabs.length ? openTabs[openTabs.length - 1].id : null;
        if (activeId) showEditor(activeId); else showWelcome();
      }
      await T.loadWorkspaces();
      renderTabs();
      T.showBanner("File deleted", "success");
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  }

  // ── Public API (used by app.js and other code) ─────────────────────────────

  T.openWorkspaceDialog = function openWorkspaceDialog() {
    createNewFile();
  };

  T.saveWorkspace = async function saveWorkspace() {
    if (activeId) await saveTab(activeId);
  };

  T.deleteWorkspace = async function deleteWorkspace(id) {
    await deleteFile(id);
  };

  T.openWorkspace = function openWorkspace(id) {
    openFile(id);
  };

  // ── Event binding ─────────────────────────────────────────────────────────

  T.bindWorkspaceIDE = function bindWorkspaceIDE() {
    // File list: open / delete
    T.$("#ide-file-list")?.addEventListener("click", (e) => {
      const del = e.target.closest("[data-ide-del]");
      if (del) { e.stopPropagation(); deleteFile(del.dataset.ideDel); return; }
      const open = e.target.closest("[data-ide-open]");
      if (open) openFile(open.dataset.ideOpen);
    });

    // Tab bar: switch / close
    T.$("#ide-tabs")?.addEventListener("click", (e) => {
      const close = e.target.closest("[data-ide-close]");
      if (close) { e.stopPropagation(); closeTab(close.dataset.ideClose); return; }
      const tab = e.target.closest("[data-ide-tab]");
      if (tab) { activeId = tab.dataset.ideTab; renderTabs(); renderFileList(); showEditor(activeId); }
    });

    // New file buttons
    T.$("#ws-new-file-btn")?.addEventListener("click", createNewFile);
    T.$("#ide-new-file-cta")?.addEventListener("click", createNewFile);

    // Textarea: track changes, autosave, cursor
    const textarea = T.$("#ide-textarea");
    if (textarea) {
      textarea.addEventListener("input", () => {
        const tab = openTabs.find(t => t.id === activeId);
        if (!tab) return;
        tab.body = textarea.value;
        tab.dirty = tab.body !== tab.origBody;
        const saved = T.$("#ide-save-status");
        if (saved) saved.textContent = tab.dirty ? "Unsaved changes" : "All saved";
        const char = T.$("#ide-char-count");
        if (char) char.textContent = `${tab.body.length} chars`;
        renderTabs();
        clearTimeout(saveTimer);
        saveTimer = setTimeout(() => { if (tab.dirty) saveTab(activeId); }, 1200);
      });

      textarea.addEventListener("keydown", (e) => {
        // Ctrl+S — save immediately
        if ((e.ctrlKey || e.metaKey) && e.key === "s") {
          e.preventDefault();
          clearTimeout(saveTimer);
          saveTab(activeId);
          return;
        }
        // Tab key inserts 2 spaces
        if (e.key === "Tab") {
          e.preventDefault();
          const start = textarea.selectionStart;
          const end = textarea.selectionEnd;
          textarea.value = textarea.value.substring(0, start) + "  " + textarea.value.substring(end);
          textarea.selectionStart = textarea.selectionEnd = start + 2;
          const tab = openTabs.find(t => t.id === activeId);
          if (tab) { tab.body = textarea.value; tab.dirty = true; renderTabs(); }
        }
      });

      textarea.addEventListener("keyup", () => updateCursorPos(textarea));
      textarea.addEventListener("click", () => updateCursorPos(textarea));
      textarea.addEventListener("select", () => updateCursorPos(textarea));
    }

    // Ctrl+N creates new file when workspace view is active
    document.addEventListener("keydown", (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "n") {
        const ws = T.$("#view-workspaces");
        if (ws && !ws.classList.contains("hidden")) {
          e.preventDefault();
          createNewFile();
        }
      }
    });
  };

})(window.Tssp);
