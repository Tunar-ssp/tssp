window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.authRequired = false;
  T.searchTimer = null;
  T.searchFilterTimer = null;
  T.editingNoteId = null;
  T.editingNoteTags = [];
  T.editingNotePinned = false;
  T.editingWorkspaceId = null;
  T.currentFolder = "";
  T.currentFiles = [];
  T.selectedFileIds = new Set();

  T.showLogin = function showLogin() {
    T.$("#login-screen").classList.remove("hidden");
    T.$("#login-screen").setAttribute("aria-hidden", "false");
    T.$("#app").classList.add("hidden");
  };

  T.showApp = function showApp() {
    T.$("#login-screen").classList.add("hidden");
    T.$("#login-screen").setAttribute("aria-hidden", "true");
    T.$("#app").classList.remove("hidden");
    T.refreshCurrentView();
  };

  T.probeAuth = async function probeAuth() {
    try {
      const res = await fetch(T.API + "/auth/required", {
        credentials: "same-origin",
      });
      if (!res.ok) {
        T.showApp();
        return;
      }
      const data = await res.json();
      T.authRequired = Boolean(data.required);
      if (T.authRequired) {
        const me = await fetch(T.API + "/auth/me", { credentials: "same-origin" });
        if (me.status === 401) {
          T.showLogin();
          return;
        }
        T.$("#logout-btn")?.removeAttribute("hidden");
      }
      T.showApp();
    } catch {
      T.showApp();
    }
  };

  T.setView = function setView(name) {
    T.$$(".nav-item").forEach((a) => {
      a.classList.toggle("active", a.dataset.view === name);
    });
    T.$$(".view").forEach((v) => v.classList.add("hidden"));
    const section = T.$(`#view-${name}`);
    if (section) section.classList.remove("hidden");
    if (name === "search" || name === "note-editor") return;
    T.refreshView(name);
  };

  T.refreshCurrentView = function refreshCurrentView() {
    const active = T.$(".nav-item.active");
    const view = active ? active.dataset.view : "objects";
    T.refreshView(view);
  };

  T.refreshView = function refreshView(view) {
    if (view === "objects") {
      T.loadFolderTree();
      T.loadFiles();
    } else if (view === "notes") T.loadNotes();
    else if (view === "overview") T.loadOverview();
    else if (view === "images") T.loadImages();
    else if (view === "videos") T.loadTypedFiles("video", "videos-body");
    else if (view === "documents") T.loadDocuments();
    else if (view === "public") T.loadPublic();
    else if (view === "workspaces") T.loadWorkspaces();
    else if (view === "admin") T.loadAdmin();
  };
})(window.Tssp);
