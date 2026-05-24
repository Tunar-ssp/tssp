(function () {
  "use strict";
  const T = window.Tssp;

  function bindEvents() {
    T.$("#login-form")?.addEventListener("submit", async (ev) => {
      ev.preventDefault();
      const err = T.$("#login-error");
      if (err) err.classList.add("hidden");
      const name = T.$("#login-name")?.value.trim() || "";
      const code = T.$("#login-code")?.value || "";
      const remember_device = T.$("#login-remember")?.checked || false;
      try {
        await fetch(T.API + "/auth/login", {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            name,
            code,
            remember_device,
            device_name: navigator.userAgent.slice(0, 80),
          }),
        }).then(async (res) => {
          if (!res.ok) {
            const b = await res.json().catch(() => ({}));
            throw new Error(b.error?.message || b.error || "Invalid credentials");
          }
        });
        T.showApp();
      } catch (e) {
        if (err) {
          err.textContent = e.message;
          err.classList.remove("hidden");
        }
      }
    });

    T.$$(".nav-item").forEach((a) => {
      a.addEventListener("click", (ev) => {
        ev.preventDefault();
        T.setView(a.dataset.view);
        T.$("#side-nav")?.classList.remove("open");
      });
    });

    T.$("#nav-toggle")?.addEventListener("click", () => {
      T.$("#side-nav")?.classList.toggle("open");
    });

    T.$("#logout-btn")?.addEventListener("click", async () => {
      try {
        await fetch(T.API + "/auth/logout", { method: "POST", credentials: "same-origin" });
      } catch {
        /* ignore */
      }
      T.showLogin();
    });

    T.$("#refresh-btn")?.addEventListener("click", () => {
      T.showBanner("");
      if (typeof T.refreshCurrentView === "function") T.refreshCurrentView();
      if (typeof T.loadFolderTree === "function") T.loadFolderTree();
    });

    T.$("#pinned-only")?.addEventListener("change", () => {
      if (typeof T.loadFiles === "function") T.loadFiles();
    });

    T.$("#global-search")?.addEventListener("input", (ev) => {
      const q = ev.target.value.trim();
      clearTimeout(T.searchTimer);
      if (typeof T.runSearch === "function") {
        T.searchTimer = setTimeout(() => T.runSearch(q), T.SEARCH_DEBOUNCE_MS);
      }
    });

    T.$("#new-note-btn")?.addEventListener("click", () => {
      if (typeof T.openNoteDialog === "function") T.openNoteDialog(null);
    });

    T.$("#note-form")?.addEventListener("submit", (ev) => {
      ev.preventDefault();
      if (typeof T.saveNote === "function") T.saveNote();
    });

    T.$("#note-cancel")?.addEventListener("click", () => {
      if (typeof T.closeNoteEditor === "function") T.closeNoteEditor();
    });

    T.$("#note-back")?.addEventListener("click", () => {
      if (typeof T.closeNoteEditor === "function") T.closeNoteEditor();
    });

    T.$("#search-clear-filters")?.addEventListener("click", () => {
      const kind = T.$("#search-kind");
      const tag = T.$("#search-tag");
      const type = T.$("#search-type");
      const vis = T.$("#search-visibility");
      const pin = T.$("#search-pinned");
      const search = T.$("#global-search");

      if (kind) kind.value = "all";
      if (tag) tag.value = "";
      if (type) type.value = "";
      if (vis) vis.value = "";
      if (pin) pin.checked = false;

      const q = search ? search.value.trim() : "";
      if (q && typeof T.runSearch === "function") T.runSearch(q);
    });

    ["search-kind", "search-tag", "search-type", "search-visibility", "search-pinned"].forEach(
      (id) => {
        const el = T.$(`#${id}`);
        if (!el) return;
        el.addEventListener("change", () => {
          const q = T.$("#global-search")?.value.trim();
          if (q && typeof T.runSearch === "function") T.runSearch(q);
        });
        if (el.tagName === "INPUT" && el.type === "text") {
          el.addEventListener(
            "input",
            () => {
              clearTimeout(T.searchFilterTimer);
              T.searchFilterTimer = setTimeout(() => {
                const query = T.$("#global-search")?.value.trim();
                if (query && typeof T.runSearch === "function") T.runSearch(query);
              }, T.SEARCH_DEBOUNCE_MS);
            }
          );
        }
      }
    );

    T.$("#note-preview-refresh")?.addEventListener("click", () => {
      if (typeof T.refreshNotePreview === "function") T.refreshNotePreview();
    });

    if (typeof T.bindNoteEditorEvents === "function") T.bindNoteEditorEvents();

    T.$("#new-workspace-btn")?.addEventListener("click", () => {
      if (typeof T.openWorkspaceDialog === "function") T.openWorkspaceDialog(null);
    });

    T.$("#workspace-form")?.addEventListener("submit", (ev) => {
      ev.preventDefault();
      if (typeof T.saveWorkspace === "function") T.saveWorkspace();
    });

    T.$("#workspace-cancel")?.addEventListener("click", () => T.$("#workspace-dialog")?.close());
    T.$("#workspace-close")?.addEventListener("click", () => T.$("#workspace-dialog")?.close());

    T.$("#admin-cleanup-temp")?.addEventListener("click", () => {
      if (typeof T.adminCleanup === "function") T.adminCleanup("temp");
    });

    T.$("#admin-cleanup-sessions")?.addEventListener("click", () => {
      if (typeof T.adminCleanup === "function") T.adminCleanup("sessions");
    });

    T.$("#admin-refresh-files")?.addEventListener("click", () => {
      if (typeof T.loadAdminFiles === "function") T.loadAdminFiles();
    });

    T.$("#console-clear-history")?.addEventListener("click", () => {
      T.$$("#console-history button").forEach((b) => b.remove());
      const h = T.$("#console-history");
      if (h) h.innerHTML = '<span class="console-hint">No commands run yet</span>';
    });

    document.addEventListener("click", (ev) => {
      const consoleBtn = ev.target.closest("[data-console-cmd]");
      if (consoleBtn && typeof T.runConsoleCommand === "function") {
        T.runConsoleCommand(consoleBtn.dataset.consoleCmd);
      }
    });

    T.$("#admin-create-user-form")?.addEventListener("submit", (ev) => {
      ev.preventDefault();
      if (typeof T.createAdminUser === "function") T.createAdminUser();
    });

    T.$("#select-all-files")?.addEventListener("change", (ev) => {
      if (typeof T.setAllVisibleFilesSelected === "function") {
        T.setAllVisibleFilesSelected(ev.target.checked);
      }
    });

    T.$("#preview-close")?.addEventListener("click", () => T.$("#preview-dialog")?.close());

    document.addEventListener("change", (ev) => {
      const fileSelect = ev.target.closest("[data-file-select]");
      if (fileSelect && typeof T.setSelectedFile === "function") {
        T.setSelectedFile(fileSelect.dataset.fileSelect, fileSelect.checked);
      }
    });

    document.addEventListener("click", (ev) => {
      const bulkAction = ev.target.closest("[data-bulk-action]");
      if (bulkAction && typeof T.bulkFileAction === "function") {
        T.bulkFileAction(bulkAction.dataset.bulkAction);
        return;
      }
      const previewFile = ev.target.closest("[data-preview-file]");
      if (previewFile && typeof T.previewFile === "function") {
        T.previewFile(previewFile.dataset.previewFile);
        return;
      }
      const renameFile = ev.target.closest("[data-rename-file]");
      if (renameFile && typeof T.renameFile === "function") {
        T.renameFile(renameFile.dataset.renameFile);
        return;
      }
      const copyLink = ev.target.closest("[data-copy-link]");
      if (copyLink && typeof T.copyText === "function") {
        T.copyText(copyLink.dataset.copyLink)
          .then(() => typeof T.showBanner === "function" && T.showBanner("Link copied", "success"))
          .catch((e) => typeof T.showBanner === "function" && T.showBanner(e.message, "error"));
        return;
      }
      const adminRole = ev.target.closest("[data-admin-role]");
      if (adminRole && typeof T.adminSetUserRole === "function") {
        T.adminSetUserRole(adminRole.dataset.adminRole, adminRole.dataset.role);
        return;
      }
      const adminResetCode = ev.target.closest("[data-admin-reset-code]");
      if (adminResetCode && typeof T.adminResetCode === "function") {
        T.adminResetCode(adminResetCode.dataset.adminResetCode);
        return;
      }
      const adminDeleteUser = ev.target.closest("[data-admin-delete-user]");
      if (adminDeleteUser && typeof T.adminDeleteUser === "function") {
        T.adminDeleteUser(adminDeleteUser.dataset.adminDeleteUser);
        return;
      }
      const adminRevokeUserDevices = ev.target.closest("[data-admin-revoke-user-devices]");
      if (adminRevokeUserDevices && typeof T.adminRevokeUserDevices === "function") {
        T.adminRevokeUserDevices(adminRevokeUserDevices.dataset.adminRevokeUserDevices);
        return;
      }
      const adminRevokeUserSessions = ev.target.closest("[data-admin-revoke-user-sessions]");
      if (adminRevokeUserSessions && typeof T.adminRevokeUserSessions === "function") {
        T.adminRevokeUserSessions(adminRevokeUserSessions.dataset.adminRevokeUserSessions);
        return;
      }
      const adminRevokeDevice = ev.target.closest("[data-admin-revoke-device]");
      if (adminRevokeDevice && typeof T.adminRevokeDevice === "function") {
        T.adminRevokeDevice(adminRevokeDevice.dataset.adminRevokeDevice);
        return;
      }
      const adminRevokeSession = ev.target.closest("[data-admin-revoke-session]");
      if (adminRevokeSession && typeof T.adminRevokeSession === "function") {
        T.adminRevokeSession(adminRevokeSession.dataset.adminRevokeSession);
        return;
      }
      const adminDeleteFile = ev.target.closest("[data-admin-delete-file]");
      if (adminDeleteFile && typeof T.adminDeleteFile === "function") {
        T.adminDeleteFile(adminDeleteFile.dataset.adminDeleteFile);
        return;
      }
      const delFile = ev.target.closest("[data-delete-file]");
      if (delFile && typeof T.deleteFile === "function") {
        T.deleteFile(delFile.dataset.deleteFile);
        return;
      }
      const pinFile = ev.target.closest("[data-pin-file]");
      if (pinFile && typeof T.toggleFilePin === "function") {
        T.toggleFilePin(
          pinFile.dataset.pinFile,
          pinFile.dataset.pinned === "1"
        );
        return;
      }
      const delNote = ev.target.closest("[data-delete-note]");
      if (delNote && typeof T.deleteNote === "function") {
        T.deleteNote(delNote.dataset.deleteNote);
        return;
      }
      const editNote = ev.target.closest("[data-edit-note]");
      if (editNote && typeof T.openNote === "function") {
        T.openNote(editNote.dataset.editNote);
        return;
      }
      const pinNote = ev.target.closest("[data-pin-note]");
      if (pinNote && typeof T.toggleNotePin === "function") {
        T.toggleNotePin(pinNote.dataset.pinNote, pinNote.dataset.pinned === "1");
        return;
      }
      const editWorkspace = ev.target.closest("[data-ws-edit]");
      if (editWorkspace && typeof T.openWorkspace === "function") {
        T.openWorkspace(editWorkspace.dataset.wsEdit);
        return;
      }
      const deleteWorkspace = ev.target.closest("[data-ws-del]");
      if (deleteWorkspace && typeof T.deleteWorkspace === "function") {
        T.deleteWorkspace(deleteWorkspace.dataset.wsDel);
        return;
      }
      const vis = ev.target.closest("[data-vis]");
      if (vis && typeof T.setFileVisibility === "function") {
        T.setFileVisibility(vis.dataset.vis, vis.dataset.v);
      }
    });

    document.addEventListener("keydown", (ev) => {
      if (ev.key !== "Enter" && ev.key !== " ") return;
      const card = ev.target.closest(".note-card[data-edit-note]");
      if (card && ev.target === card && typeof T.openNote === "function") {
        ev.preventDefault();
        T.openNote(card.dataset.editNote);
      }
    });

    if (typeof T.bindUpload === "function") T.bindUpload();
    if (typeof T.bindEditorEvents === "function") T.bindEditorEvents();
  }

  try {
    bindEvents();
    T.probeAuth().catch((err) => {
      console.error("Auth probe failed:", err);
      T.showApp();
    });
  } catch (err) {
    console.error("Initialization failed:", err);
    document.body.innerHTML = `<div style="min-height:100vh;display:flex;align-items:center;justify-content:center;background:#08080b;color:#ececf1;font-family:system-ui;padding:24px"><div style="max-width:480px;text-align:center"><h2 style="color:#f87171;margin-bottom:12px">Dashboard failed to load</h2><p style="color:#9b9ba6;margin-bottom:16px">A JavaScript initialisation error prevented the app from starting.</p><pre style="background:#15151b;border:1px solid #282832;border-radius:8px;padding:14px;text-align:left;overflow:auto;font-size:12px;color:#fbbf24">${T.escapeHtml(err?.stack || err?.message || String(err))}</pre><button onclick="location.reload()" style="margin-top:18px;padding:10px 20px;background:#a855f7;color:#fff;border:none;border-radius:6px;cursor:pointer;font-size:14px">Reload</button></div></div>`;
  }
})();
