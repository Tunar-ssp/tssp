(function () {
  "use strict";
  const T = window.Tssp;

  T.setAdminTab = function setAdminTab(name) {
    T.$$(".admin-tab").forEach((button) => {
      button.classList.toggle("active", button.dataset.adminTab === name);
      button.setAttribute("aria-selected", button.dataset.adminTab === name ? "true" : "false");
    });
    T.$$(".admin-panel").forEach((panel) => {
      panel.classList.toggle("hidden", panel.dataset.adminPanel !== name);
      panel.classList.toggle("active", panel.dataset.adminPanel === name);
    });
  };

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

    T.$$(".admin-tab").forEach((button) => {
      button.addEventListener("click", () => T.setAdminTab(button.dataset.adminTab));
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

    T.$("#files-sort")?.addEventListener("change", () => {
      if (typeof T.loadFiles === "function") T.loadFiles();
    });

    T.$("#global-search")?.addEventListener("input", (ev) => {
      if (typeof T.openCommandPalette === "function") {
        T.openCommandPalette(ev.target.value.trim());
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

    T.$("#preview-prev")?.addEventListener("click", () => {
      if (T.currentPreviewId && T.currentFiles) {
        const idx = T.currentFiles.findIndex((f) => f.id === T.currentPreviewId);
        if (idx > 0 && typeof T.previewFile === "function") T.previewFile(T.currentFiles[idx - 1].id);
      }
    });

    T.$("#preview-next")?.addEventListener("click", () => {
      if (T.currentPreviewId && T.currentFiles) {
        const idx = T.currentFiles.findIndex((f) => f.id === T.currentPreviewId);
        if (idx >= 0 && idx < T.currentFiles.length - 1 && typeof T.previewFile === "function") {
          T.previewFile(T.currentFiles[idx + 1].id);
        }
      }
    });

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
      const filesView = ev.target.closest("[data-files-view]");
      if (filesView && typeof T.setFilesViewMode === "function") {
        T.setFilesViewMode(filesView.dataset.filesView);
        return;
      }
      const uploadTrigger = ev.target.closest("[data-upload-trigger]");
      if (uploadTrigger) {
        T.$("#upload-input")?.click();
        return;
      }
      const viewJump = ev.target.closest("[data-view-jump]");
      if (viewJump && typeof T.setView === "function") {
        T.setView(viewJump.dataset.viewJump);
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
      const shareFile = ev.target.closest("[data-share-file]");
      if (shareFile && typeof T.showFileShare === "function") {
        T.showFileShare(shareFile.dataset.shareFile);
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
      const duplicateNote = ev.target.closest("[data-duplicate-note]");
      if (duplicateNote && typeof T.duplicateNote === "function") {
        T.duplicateNote(duplicateNote.dataset.duplicateNote);
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
      const openEditorWs = ev.target.closest("[data-ws-open-editor]");
      if (openEditorWs) {
        const wsId = openEditorWs.dataset.wsOpenEditor;
        if (typeof T.openEditorWorkspace === "function") {
          T.setView("editor");
          T.openEditorWorkspace(wsId);
        } else {
          T.setView("editor");
        }
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
      // Note card keyboard activation
      if (ev.key === "Enter" || ev.key === " ") {
        const card = ev.target.closest(".note-card[data-edit-note]");
        if (card && ev.target === card && typeof T.openNote === "function") {
          ev.preventDefault();
          T.openNote(card.dataset.editNote);
          return;
        }
      }

      // Ctrl+K / Cmd+K — focus global search
      if ((ev.ctrlKey || ev.metaKey) && ev.key === "k") {
        ev.preventDefault();
        T.openCommandPalette?.(T.$("#global-search")?.value || "");
        return;
      }

      // Ctrl+N — new note (when on notes view and no dialog open)
      if ((ev.ctrlKey || ev.metaKey) && ev.key === "n") {
        const dialog = document.querySelector("dialog[open]");
        if (dialog) return;
        const notesVisible = !T.$("#view-notes")?.classList.contains("hidden");
        if (notesVisible && typeof T.openNoteDialog === "function") {
          ev.preventDefault();
          T.openNoteDialog(null);
        }
        return;
      }

      // Arrow keys — navigate preview dialog
      if (ev.key === "ArrowLeft" || ev.key === "ArrowRight") {
        const dialog = T.$("#preview-dialog");
        if (dialog?.open) {
          ev.preventDefault();
          const btn = ev.key === "ArrowLeft" ? T.$("#preview-prev") : T.$("#preview-next");
          btn?.click();
          return;
        }
      }

      // Escape — close open dialogs or go back from editor views
      if (ev.key === "Escape") {
        const openDialog = document.querySelector("dialog[open]");
        if (openDialog) { openDialog.close(); return; }
        const view = document.querySelector(".view:not(.hidden)");
        if (view?.id === "view-note-editor" && typeof T.closeNoteEditor === "function") {
          T.closeNoteEditor();
        }
      }
    });

    if (typeof T.bindUpload === "function") T.bindUpload();
    if (typeof T.bindEditorEvents === "function") T.bindEditorEvents();
    if (typeof T.bindCommandPalette === "function") T.bindCommandPalette();
    T.setAdminTab("overview");
  }

  try {
    bindEvents();
    T.probeAuth().catch((err) => {
      console.error("Auth probe failed:", err);
      T.showApp();
    });
  } catch (err) {
    console.error("Initialization failed:", err);
    T.showBootError(
      "Dashboard failed to load",
      "A JavaScript initialisation error prevented the app from starting.",
      err?.stack || err?.message || String(err),
    );
  }
})();
