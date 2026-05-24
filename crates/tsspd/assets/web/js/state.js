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
  T.workspaceDialogSource = "workspaces";
  T.currentFolder = "";
  T.currentFiles = [];
  T.selectedFileIds = new Set();
  T.editorCurrentWorkspaceId = null;
  T.editorCurrentWorkspace = null;
	  T.editorCurrentDocumentId = null;
	  T.editorCurrentDocument = null;
	  T.editorDocuments = [];
	  T.editorWorkspaces = [];
	  T.editorOpenTabs = [];
  T.editorDirty = false;
  T.editorSaving = false;
  T.editorDocumentDialogMode = "create";

  T.showLogin = function showLogin() {
    T.$("#login-screen")?.classList.remove("hidden");
    T.$("#login-screen")?.setAttribute("aria-hidden", "false");
    T.$("#app")?.classList.add("hidden");
    T.markBootReady();
  };

  T.showApp = function showApp() {
    T.$("#login-screen")?.classList.add("hidden");
    T.$("#login-screen")?.setAttribute("aria-hidden", "true");
    T.$("#app")?.classList.remove("hidden");
    try {
      if (typeof T.refreshCurrentView === "function") {
        T.refreshCurrentView();
      }
    } catch (error) {
      T.showBootError(
        "Dashboard failed to load",
        "The main dashboard view could not be rendered.",
        error instanceof Error ? `${error.name}: ${error.message}` : String(error),
      );
      return;
    }
    T.markBootReady();
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
        const meData = me.ok ? await me.json().catch(() => null) : null;
        T.currentUserRole = meData?.role || null;
        T.currentUserName = meData?.name || null;
        const isAdmin = T.currentUserRole === "admin";
        document.querySelectorAll(".admin-only").forEach((el) => {
          if (isAdmin) el.classList.remove("hidden");
          else el.classList.add("hidden");
        });
        const authStatus = T.$("#auth-status");
        if (authStatus) authStatus.textContent = meData?.name ? `${meData.name}${isAdmin ? " · admin" : ""}` : "Signed in";
        T.$("#logout-btn")?.removeAttribute("hidden");
      } else {
        const authStatus = T.$("#auth-status");
        if (authStatus) authStatus.textContent = "Open local";
      }
      T.showApp();
    } catch {
      T.showApp();
    }
  };

  T.setView = function setView(name) {
    if (!T.$(`#view-${name}`)) {
      name = "objects";
    }
    T.$$(".nav-item").forEach((a) => {
      a.classList.toggle("active", a.dataset.view === name);
    });
    T.$$(".view").forEach((v) => v.classList.add("hidden"));
    const section = T.$(`#view-${name}`);
    if (section) section.classList.remove("hidden");
    if (name !== "note-editor" && location.hash !== `#${name}`) {
      history.replaceState(null, "", `#${name}`);
    }
    if (name === "search" || name === "note-editor") return;
    if (typeof T.refreshView === "function") {
      T.refreshView(name);
    }
  };

  T.refreshCurrentView = function refreshCurrentView() {
    const hashView = location.hash ? location.hash.slice(1) : "";
    const active = T.$(".nav-item.active");
    const view = T.$(`#view-${hashView}`) && hashView !== "note-editor"
      ? hashView
      : active?.dataset.view || "objects";
    if (view !== active?.dataset.view) {
      T.setView(view);
      return;
    }
    if (typeof T.refreshView === "function") {
      T.refreshView(view);
    }
  };

  T.refreshView = function refreshView(view) {
    if (view === "objects") {
      T.loadFolderTree?.();
      T.loadFiles?.();
    } else if (view === "notes") T.loadNotes?.();
    else if (view === "overview") T.loadOverview?.();
    else if (view === "images") T.loadImages?.();
    else if (view === "videos") T.loadVideos?.();
    else if (view === "documents") T.loadDocuments?.();
    else if (view === "public") T.loadPublic?.();
    else if (view === "workspaces") T.loadWorkspaces?.();
    else if (view === "admin") T.loadAdmin?.();
    else if (view === "editor") T.loadEditor?.();
  };
})(window.Tssp);
