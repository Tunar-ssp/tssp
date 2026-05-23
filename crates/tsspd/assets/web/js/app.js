(function () {
  "use strict";
  const T = window.Tssp;

  function bindEvents() {
    T.$("#login-form").addEventListener("submit", async (ev) => {
      ev.preventDefault();
      const err = T.$("#login-error");
      err.classList.add("hidden");
      const password = T.$("#login-password").value;
      try {
        await fetch(T.API + "/auth/login", {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ password }),
        }).then(async (res) => {
          if (!res.ok) {
            const b = await res.json().catch(() => ({}));
            throw new Error(b.error?.message || b.error || "Invalid password");
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
    T.$("#note-cancel").addEventListener("click", () => T.$("#note-dialog").close());
    T.$("#note-close").addEventListener("click", () => T.$("#note-dialog").close());

    T.$("#admin-cleanup-temp")?.addEventListener("click", () => T.adminCleanup("temp"));
    T.$("#admin-cleanup-sessions")?.addEventListener("click", () =>
      T.adminCleanup("sessions")
    );

    document.addEventListener("click", (ev) => {
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
      }
    });

    T.bindUpload();
  }

  bindEvents();
  T.probeAuth().then(() => T.loadFolderTree());
})();
