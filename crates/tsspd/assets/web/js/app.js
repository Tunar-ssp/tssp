(function () {
  "use strict";
  const T = window.Tssp;

  function bindEvents() {
    T.$("#login-form").addEventListener("submit", async (ev) => {
      ev.preventDefault();
      const err = T.$("#login-error");
      err.classList.add("hidden");
      const name = T.$("#login-name").value.trim();
      const code = T.$("#login-code").value;
      const remember_device = T.$("#login-remember").checked;
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
        err.textContent = e.message;
        err.classList.remove("hidden");
      }
    });

    T.$$(".nav-item").forEach((a) => {
      a.addEventListener("click", (ev) => {
        ev.preventDefault();
        T.setView(a.dataset.view);
        T.$("#side-nav").classList.remove("open");
      });
    });

    T.$("#nav-toggle").addEventListener("click", () => {
      T.$("#side-nav").classList.toggle("open");
    });

    T.$("#logout-btn")?.addEventListener("click", async () => {
      try {
        await fetch(T.API + "/auth/logout", { method: "POST", credentials: "same-origin" });
      } catch {
        /* ignore */
      }
      T.showLogin();
    });

    T.$("#refresh-btn").addEventListener("click", () => {
      T.showBanner("");
      T.refreshCurrentView();
      T.loadFolderTree();
    });

    T.$("#pinned-only")?.addEventListener("change", () => T.loadFiles());

    T.$("#global-search").addEventListener("input", (ev) => {
      const q = ev.target.value.trim();
      clearTimeout(T.searchTimer);
      T.searchTimer = setTimeout(() => T.runSearch(q), T.SEARCH_DEBOUNCE_MS);
    });

    T.$("#new-note-btn")?.addEventListener("click", () => T.openNoteDialog(null));
    T.$("#note-form").addEventListener("submit", (ev) => {
      ev.preventDefault();
      T.saveNote();
    });
    T.$("#note-cancel")?.addEventListener("click", () => T.closeNoteEditor());
    T.$("#note-back")?.addEventListener("click", () => T.closeNoteEditor());
    T.$("#search-clear-filters")?.addEventListener("click", () => {
      T.$("#search-kind").value = "all";
      T.$("#search-tag").value = "";
      T.$("#search-type").value = "";
      T.$("#search-visibility").value = "";
      T.$("#search-pinned").checked = false;
      const q = T.$("#global-search").value.trim();
      if (q) T.runSearch(q);
    });
    ["search-kind", "search-tag", "search-type", "search-visibility", "search-pinned"].forEach(
      (id) => {
        const el = T.$(`#${id}`);
        if (!el) return;
        el.addEventListener("change", () => {
          const q = T.$("#global-search").value.trim();
          if (q) T.runSearch(q);
        });
        if (el.tagName === "INPUT" && el.type === "text") {
          el.addEventListener(
            "input",
            () => {
              clearTimeout(T.searchFilterTimer);
              T.searchFilterTimer = setTimeout(() => {
                const query = T.$("#global-search").value.trim();
                if (query) T.runSearch(query);
              }, T.SEARCH_DEBOUNCE_MS);
            }
          );
        }
      }
    );
    T.$("#note-preview-refresh")?.addEventListener("click", () => T.refreshNotePreview());
    T.$("#note-body-input")?.addEventListener("input", () => T.refreshNotePreview());

    T.$("#new-workspace-btn")?.addEventListener("click", () =>
      T.openWorkspaceDialog(null)
    );
    T.$("#workspace-form")?.addEventListener("submit", (ev) => {
      ev.preventDefault();
      T.saveWorkspace();
    });
    T.$("#workspace-cancel")?.addEventListener("click", () =>
      T.$("#workspace-dialog").close()
    );
    T.$("#workspace-close")?.addEventListener("click", () =>
      T.$("#workspace-dialog").close()
    );

    T.$("#admin-cleanup-temp")?.addEventListener("click", () => T.adminCleanup("temp"));
    T.$("#admin-cleanup-sessions")?.addEventListener("click", () =>
      T.adminCleanup("sessions")
    );
    T.$("#admin-refresh-files")?.addEventListener("click", () => T.loadAdminFiles());
    T.$("#admin-create-user-form")?.addEventListener("submit", (ev) => {
      ev.preventDefault();
      T.createAdminUser();
    });
    T.$("#select-all-files")?.addEventListener("change", (ev) => {
      T.setAllVisibleFilesSelected(ev.target.checked);
    });
    T.$("#preview-close")?.addEventListener("click", () => T.$("#preview-dialog").close());

    document.addEventListener("change", (ev) => {
      const fileSelect = ev.target.closest("[data-file-select]");
      if (fileSelect) {
        T.setSelectedFile(fileSelect.dataset.fileSelect, fileSelect.checked);
      }
    });

    document.addEventListener("click", (ev) => {
      const bulkAction = ev.target.closest("[data-bulk-action]");
      if (bulkAction) {
        T.bulkFileAction(bulkAction.dataset.bulkAction);
        return;
      }
      const previewFile = ev.target.closest("[data-preview-file]");
      if (previewFile) {
        T.previewFile(previewFile.dataset.previewFile);
        return;
      }
      const renameFile = ev.target.closest("[data-rename-file]");
      if (renameFile) {
        T.renameFile(renameFile.dataset.renameFile);
        return;
      }
      const copyLink = ev.target.closest("[data-copy-link]");
      if (copyLink) {
        T.copyText(copyLink.dataset.copyLink)
          .then(() => T.showBanner("Link copied", "success"))
          .catch((e) => T.showBanner(e.message, "error"));
        return;
      }
      const adminRole = ev.target.closest("[data-admin-role]");
      if (adminRole) {
        T.adminSetUserRole(adminRole.dataset.adminRole, adminRole.dataset.role);
        return;
      }
      const adminResetCode = ev.target.closest("[data-admin-reset-code]");
      if (adminResetCode) {
        T.adminResetCode(adminResetCode.dataset.adminResetCode);
        return;
      }
      const adminDeleteUser = ev.target.closest("[data-admin-delete-user]");
      if (adminDeleteUser) {
        T.adminDeleteUser(adminDeleteUser.dataset.adminDeleteUser);
        return;
      }
      const adminRevokeUserDevices = ev.target.closest("[data-admin-revoke-user-devices]");
      if (adminRevokeUserDevices) {
        T.adminRevokeUserDevices(adminRevokeUserDevices.dataset.adminRevokeUserDevices);
        return;
      }
      const adminRevokeDevice = ev.target.closest("[data-admin-revoke-device]");
      if (adminRevokeDevice) {
        T.adminRevokeDevice(adminRevokeDevice.dataset.adminRevokeDevice);
        return;
      }
      const adminDeleteFile = ev.target.closest("[data-admin-delete-file]");
      if (adminDeleteFile) {
        T.adminDeleteFile(adminDeleteFile.dataset.adminDeleteFile);
        return;
      }
      const delFile = ev.target.closest("[data-delete-file]");
      if (delFile) {
        T.deleteFile(delFile.dataset.deleteFile);
        return;
      }
      const pinFile = ev.target.closest("[data-pin-file]");
      if (pinFile) {
        T.toggleFilePin(
          pinFile.dataset.pinFile,
          pinFile.dataset.pinned === "1"
        );
        return;
      }
      const delNote = ev.target.closest("[data-delete-note]");
      if (delNote) {
        T.deleteNote(delNote.dataset.deleteNote);
        return;
      }
      const editNote = ev.target.closest("[data-edit-note]");
      if (editNote) {
        T.openNote(editNote.dataset.editNote);
        return;
      }
      const pinNote = ev.target.closest("[data-pin-note]");
      if (pinNote) {
        T.toggleNotePin(pinNote.dataset.pinNote, pinNote.dataset.pinned === "1");
        return;
      }
      const editWorkspace = ev.target.closest("[data-ws-edit]");
      if (editWorkspace) {
        T.openWorkspace(editWorkspace.dataset.wsEdit);
        return;
      }
      const deleteWorkspace = ev.target.closest("[data-ws-del]");
      if (deleteWorkspace) {
        T.deleteWorkspace(deleteWorkspace.dataset.wsDel);
        return;
      }
      const vis = ev.target.closest("[data-vis]");
      if (vis) {
        T.setFileVisibility(vis.dataset.vis, vis.dataset.v);
      }
    });

    T.bindUpload();
  }

  bindEvents();
  T.probeAuth().then(() => T.loadFolderTree());
})();
