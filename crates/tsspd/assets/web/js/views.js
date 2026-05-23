window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.loadOverview = async function loadOverview() {
    const grid = T.$("#metric-grid");
    grid.innerHTML = '<div class="metric-card"><span class="label">Loading…</span></div>';
    try {
      const s = await T.api("/status");
      const metrics = [
        ["Files", s.file_count],
        ["Notes", s.note_count],
        ["Tags", s.tag_count],
        ["Pinned", s.pinned_count],
        ["Uploads (24h)", s.recent_upload_count_24h],
        ["Storage", T.formatBytes(s.storage_bytes_used)],
        ["Uptime", T.formatUptime(s.uptime_seconds)],
        ["Status", s.status],
      ];
      grid.innerHTML = metrics
        .map(
          ([label, value]) =>
            `<div class="metric-card"><div class="label">${T.escapeHtml(label)}</div><div class="value">${T.escapeHtml(String(value))}</div></div>`
        )
        .join("");
    } catch (e) {
      grid.innerHTML = `<div class="metric-card"><span class="label">Error</span><div class="value">${T.escapeHtml(e.message)}</div></div>`;
    }
  };

  T.loadFolderTree = async function loadFolderTree() {
    const tree = T.$("#folder-tree");
    if (!tree) return;
    try {
      const data = await T.api("/admin/folders");
      const folders = data.folders || [];
      const items = [
        { path: "", label: "Bucket root", count: folders.find((f) => !f.path)?.file_count },
      ];
      for (const f of folders) {
        if (f.path) items.push({ path: f.path, label: f.path, count: f.file_count });
      }
      tree.innerHTML = items
        .map(
          (f) =>
            `<button type="button" data-folder="${T.escapeHtml(f.path)}" class="${f.path === T.currentFolder ? "active" : ""}">${T.escapeHtml(f.label || "root")} <span class="mono">(${f.count ?? 0})</span></button>`
        )
        .join("");
      tree.querySelectorAll("button").forEach((btn) => {
        btn.addEventListener("click", () => {
          T.currentFolder = btn.dataset.folder || "";
          T.$("#breadcrumb-folder").textContent = T.currentFolder || "default";
          tree.querySelectorAll("button").forEach((b) => b.classList.remove("active"));
          btn.classList.add("active");
          T.loadFiles();
        });
      });
    } catch {
      tree.innerHTML = '<button type="button" data-folder="" class="active">Bucket root</button>';
    }
  };

  T.loadFiles = async function loadFiles() {
    const body = T.$("#files-body");
    body.innerHTML = '<tr><td colspan="6" class="table-empty">Loading…</td></tr>';
    const pinned = T.$("#pinned-only")?.checked;
    const params = new URLSearchParams({ limit: "200" });
    if (pinned) params.set("pinned", "true");
    if (T.currentFolder) params.set("folder", T.currentFolder);
    try {
      const data = await T.api("/files?" + params.toString());
      const files = data.files || [];
      if (!files.length) {
        body.innerHTML =
          '<tr><td colspan="6" class="table-empty">No objects in this folder. Drop files above or click Upload.</td></tr>';
        return;
      }
      body.innerHTML = files
        .map((f) => {
          const pin = f.pinned
            ? '<span class="pin" title="Pinned">★</span>'
            : "";
          const name = T.escapeHtml(f.name || f.id);
          const id = T.escapeHtml(f.id);
          const folder = f.folder_path
            ? `<span class="tag">${T.escapeHtml(f.folder_path)}</span>`
            : "";
          const tags = (f.tags || [])
            .map((t) => `<span class="tag">${T.escapeHtml(t)}</span>`)
            .join("");
          const download = `/api/v1/files/${id}/content?disposition=attachment`;
          return `<tr>
            <td><div class="file-name">${pin}<span>${name}</span></div>${folder}${tags}</td>
            <td class="mono">${T.escapeHtml(T.formatBytes(f.size_bytes))}</td>
            <td class="mono">${T.escapeHtml(f.mime_type || "—")}</td>
            <td>${T.escapeHtml(T.formatDate(f.uploaded_at))}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-pin-file="${id}" data-pinned="${f.pinned ? "1" : "0"}">${f.pinned ? "Unpin" : "Pin"}</button>
              <button type="button" class="btn btn-text btn-sm" data-vis="${id}" data-v="${f.visibility === "public" ? "private" : "public"}">${f.visibility === "public" ? "Private" : "Public"}</button>
              <a class="btn btn-text btn-sm" href="${download}" download>Download</a>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-delete-file="${id}">Delete</button>
            </td>
          </tr>`;
        })
        .join("");
    } catch (e) {
      body.innerHTML = `<tr><td colspan="6" class="table-empty">${T.escapeHtml(e.message)}</td></tr>`;
    }
  };

  T.loadImages = async function loadImages() {
    const grid = T.$("#image-grid");
    grid.innerHTML = '<p class="table-empty">Loading…</p>';
    try {
      const data = await T.api("/files?type=image&limit=200");
      const files = (data.files || []).filter((f) =>
        (f.mime_type || "").startsWith("image/")
      );
      if (!files.length) {
        grid.innerHTML = '<p class="table-empty">No images yet. Upload to the photos folder or any path.</p>';
        return;
      }
      grid.innerHTML = files
        .map((f) => {
          const id = T.escapeHtml(f.id);
          const thumb = `/api/v1/files/${id}/thumbnail`;
          const name = T.escapeHtml(f.name || f.id);
          return `<a class="image-card" href="/api/v1/files/${id}/content" target="_blank" rel="noopener">
            <img src="${thumb}" alt="${name}" loading="lazy" onerror="this.src='/api/v1/files/${id}/content'">
            <span class="caption">${name}</span>
          </a>`;
        })
        .join("");
    } catch (e) {
      grid.innerHTML = `<p class="table-empty">${T.escapeHtml(e.message)}</p>`;
    }
  };

  T.loadNotes = async function loadNotes() {
    const body = T.$("#notes-body");
    body.innerHTML = '<tr><td colspan="4" class="table-empty">Loading…</td></tr>';
    try {
      const data = await T.api("/notes?limit=200");
      const notes = data.notes || [];
      if (!notes.length) {
        body.innerHTML = '<tr><td colspan="4" class="table-empty">No notes yet.</td></tr>';
        return;
      }
      body.innerHTML = notes
        .map((n) => {
          const id = T.escapeHtml(n.id);
          const tags = (n.tags || [])
            .map((t) => `<span class="tag">${T.escapeHtml(t)}</span>`)
            .join("");
          return `<tr>
            <td><strong>${T.escapeHtml(n.title || "Untitled")}</strong></td>
            <td>${T.escapeHtml(T.formatDate(n.updated_at))}</td>
            <td>${tags || "—"}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Edit</button>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-delete-note="${id}">Delete</button>
            </td>
          </tr>`;
        })
        .join("");
    } catch (e) {
      body.innerHTML = `<tr><td colspan="4" class="table-empty">${T.escapeHtml(e.message)}</td></tr>`;
    }
  };

  T.runSearch = async function runSearch(q) {
    const body = T.$("#search-body");
    const sub = T.$("#search-subtitle");
    if (!q || q.length < 1) {
      sub.textContent = "Type in the search bar above";
      body.innerHTML =
        '<tr><td colspan="4" class="table-empty">Enter a query to search</td></tr>';
      return;
    }
    sub.textContent = `Results for “${q}”`;
    body.innerHTML = '<tr><td colspan="4" class="table-empty">Searching…</td></tr>';
    T.setView("search");
    try {
      const data = await T.api("/search?q=" + encodeURIComponent(q));
      const results = data.results || [];
      if (!results.length) {
        body.innerHTML = '<tr><td colspan="4" class="table-empty">No matches</td></tr>';
        return;
      }
      body.innerHTML = results
        .map((r) => {
          const type = T.escapeHtml(r.type || "item");
          const name = T.escapeHtml(r.title || r.name || r.id);
          const id = T.escapeHtml(r.id);
          let actions = "";
          if (r.type === "file") {
            actions = `<a class="btn btn-text btn-sm" href="/api/v1/files/${id}/content" download>Download</a>`;
          } else if (r.type === "note") {
            actions = `<button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Open</button>`;
          }
          const detail =
            r.type === "file"
              ? T.escapeHtml(T.formatBytes(r.size_bytes))
              : T.escapeHtml((r.snippet || r.body || "").slice(0, 80));
          return `<tr>
            <td><span class="tag">${type}</span></td>
            <td><strong>${name}</strong></td>
            <td class="mono">${detail || "—"}</td>
            <td class="col-actions">${actions}</td>
          </tr>`;
        })
        .join("");
    } catch (e) {
      body.innerHTML = `<tr><td colspan="4" class="table-empty">${T.escapeHtml(e.message)}</td></tr>`;
    }
  };

  T.setFileVisibility = async function setFileVisibility(id, visibility) {
    try {
      await T.api("/files/" + encodeURIComponent(id) + "/visibility", {
        method: "PATCH",
        body: JSON.stringify({ visibility }),
      });
      T.showBanner(visibility === "public" ? "Object is now public" : "Object is now private", "success");
      T.refreshCurrentView();
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  };

  T.loadPublic = async function loadPublic() {
    const body = T.$("#public-body");
    body.innerHTML = '<tr><td colspan="4" class="table-empty">Loading…</td></tr>';
    try {
      const data = await fetch(T.API + "/public/files").then((r) => r.json());
      const files = data.files || [];
      if (!files.length) {
        body.innerHTML = '<tr><td colspan="4" class="table-empty">No public files yet.</td></tr>';
        return;
      }
      body.innerHTML = files
        .map((f) => {
          const id = T.escapeHtml(f.id);
          const link = f.public_token
            ? `<a class="mono" href="/p/${T.escapeHtml(f.public_token)}">/p/${T.escapeHtml(f.public_token)}</a>`
            : "—";
          return `<tr>
            <td>${T.escapeHtml(f.name)}</td>
            <td class="mono">${T.formatBytes(f.size_bytes)}</td>
            <td>${link}</td>
            <td class="col-actions">
              <a class="btn btn-text btn-sm" href="/api/v1/files/${id}/content" download>Download</a>
              <button type="button" class="btn btn-text btn-sm" data-vis="${id}" data-v="private">Make private</button>
            </td>
          </tr>`;
        })
        .join("");
      body.querySelectorAll("[data-vis]").forEach((btn) => {
        btn.addEventListener("click", () => T.setFileVisibility(btn.dataset.vis, btn.dataset.v));
      });
    } catch (e) {
      body.innerHTML = `<tr><td colspan="4" class="table-empty">${T.escapeHtml(e.message)}</td></tr>`;
    }
  };

  T.loadTypedFiles = async function loadTypedFiles(mimePrefix, bodyId) {
    const body = T.$(`#${bodyId}`);
    body.innerHTML = '<tr><td colspan="4" class="table-empty">Loading…</td></tr>';
    const params = new URLSearchParams({ limit: "200", type: mimePrefix + "/" });
    try {
      const data = await T.api("/files?" + params.toString());
      const files = data.files || [];
      if (!files.length) {
        body.innerHTML = '<tr><td colspan="4" class="table-empty">No matching objects.</td></tr>';
        return;
      }
      body.innerHTML = files
        .map((f) => {
          const id = T.escapeHtml(f.id);
          const inline = `/api/v1/files/${id}/content?disposition=inline`;
          return `<tr>
            <td>${T.escapeHtml(f.name)}</td>
            <td class="mono">${T.formatBytes(f.size_bytes)}</td>
            <td>${T.formatDate(f.uploaded_at)}</td>
            <td class="col-actions">
              <a class="btn btn-text btn-sm" href="${inline}" target="_blank" rel="noopener">Open</a>
              <button type="button" class="btn btn-text btn-sm" data-vis="${id}" data-v="${f.visibility === "public" ? "private" : "public"}">${f.visibility === "public" ? "Private" : "Public"}</button>
            </td>
          </tr>`;
        })
        .join("");
      body.querySelectorAll("[data-vis]").forEach((btn) => {
        btn.addEventListener("click", () => T.setFileVisibility(btn.dataset.vis, btn.dataset.v));
      });
    } catch (e) {
      body.innerHTML = `<tr><td colspan="4" class="table-empty">${T.escapeHtml(e.message)}</td></tr>`;
    }
  };

  T.loadDocuments = async function loadDocuments() {
    const body = T.$("#documents-body");
    body.innerHTML = '<tr><td colspan="5" class="table-empty">Loading…</td></tr>';
    try {
      const data = await T.api("/files?limit=200");
      const files = (data.files || []).filter(
        (f) =>
          f.mime_type &&
          (f.mime_type.startsWith("application/") ||
            f.mime_type.startsWith("text/")) &&
          !f.mime_type.startsWith("image/") &&
          !f.mime_type.startsWith("video/")
      );
      if (!files.length) {
        body.innerHTML = '<tr><td colspan="5" class="table-empty">No documents.</td></tr>';
        return;
      }
      body.innerHTML = files
        .map(
          (f) => `<tr>
            <td>${T.escapeHtml(f.name)}</td>
            <td class="mono">${T.escapeHtml(f.mime_type)}</td>
            <td class="mono">${T.formatBytes(f.size_bytes)}</td>
            <td>${T.escapeHtml(f.folder_path || "—")}</td>
            <td>${T.formatDate(f.uploaded_at)}</td>
          </tr>`
        )
        .join("");
    } catch (e) {
      body.innerHTML = `<tr><td colspan="5" class="table-empty">${T.escapeHtml(e.message)}</td></tr>`;
    }
  };

  T.loadWorkspaces = async function loadWorkspaces() {
    const body = T.$("#workspaces-body");
    body.innerHTML = '<tr><td colspan="4" class="table-empty">Loading…</td></tr>';
    try {
      const data = await T.api("/workspaces");
      const items = data.workspaces || [];
      if (!items.length) {
        body.innerHTML = '<tr><td colspan="4" class="table-empty">No workspaces yet.</td></tr>';
        return;
      }
      body.innerHTML = items
        .map(
          (w) => `<tr>
            <td>${T.escapeHtml(w.name)}</td>
            <td class="mono">${T.escapeHtml(w.language)}</td>
            <td>${T.formatDate(w.updated_at)}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-ws-edit="${T.escapeHtml(w.id)}">Edit</button>
              <button type="button" class="btn btn-text btn-sm" data-ws-del="${T.escapeHtml(w.id)}">Delete</button>
            </td>
          </tr>`
        )
        .join("");
    } catch (e) {
      body.innerHTML = `<tr><td colspan="4" class="table-empty">${T.escapeHtml(e.message)}</td></tr>`;
    }
  };

  T.openWorkspaceDialog = function openWorkspaceDialog(workspace) {
    T.editingWorkspaceId = workspace ? workspace.id : null;
    T.$("#workspace-dialog-title").textContent = workspace
      ? "Edit workspace"
      : "New workspace";
    T.$("#workspace-name-input").value = workspace ? workspace.name || "" : "";
    T.$("#workspace-language-input").value = workspace
      ? workspace.language || "text"
      : "text";
    T.$("#workspace-body-input").value = workspace ? workspace.body || "" : "";
    T.$("#workspace-dialog").showModal();
  };

  T.saveWorkspace = async function saveWorkspace() {
    const payload = {
      name: T.$("#workspace-name-input").value.trim(),
      language: T.$("#workspace-language-input").value.trim() || "text",
      body: T.$("#workspace-body-input").value,
    };
    if (!payload.name) {
      T.showBanner("Workspace name is required", "error");
      return;
    }
    try {
      if (T.editingWorkspaceId) {
        await T.api("/workspaces/" + encodeURIComponent(T.editingWorkspaceId), {
          method: "PUT",
          body: JSON.stringify(payload),
        });
      } else {
        await T.api("/workspaces", {
          method: "POST",
          body: JSON.stringify(payload),
        });
      }
      T.$("#workspace-dialog").close();
      T.showBanner("Workspace saved", "success");
      T.loadWorkspaces();
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  };

  T.openWorkspace = async function openWorkspace(id) {
    try {
      const workspace = await T.api("/workspaces/" + encodeURIComponent(id));
      T.openWorkspaceDialog(workspace);
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  };

  T.deleteWorkspace = async function deleteWorkspace(id) {
    if (!confirm("Delete this workspace?")) return;
    try {
      await T.api("/workspaces/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("Workspace deleted", "success");
      T.loadWorkspaces();
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  };

  T.loadAdmin = async function loadAdmin() {
    const overview = T.$("#admin-overview");
    const system = T.$("#admin-system");
    const usersEl = T.$("#admin-users");
    const devicesEl = T.$("#admin-devices");
    overview.innerHTML = "Loading…";
    system.innerHTML = "Loading…";
    usersEl.innerHTML = "Loading…";
    devicesEl.innerHTML = "Loading…";
    try {
      const [ov, sys, users, devices] = await Promise.all([
        T.api("/admin/overview"),
        T.api("/admin/system"),
        T.api("/admin/users").catch(() => ({ users: [] })),
        T.api("/admin/devices").catch(() => ({ devices: [] })),
      ]);
      overview.innerHTML = `
        <dl class="admin-dl">
          <dt>Files</dt><dd>${ov.file_count}</dd>
          <dt>Notes</dt><dd>${ov.note_count}</dd>
          <dt>Tags</dt><dd>${ov.tag_count}</dd>
          <dt>Pinned</dt><dd>${ov.pinned_count}</dd>
          <dt>Corrupt</dt><dd>${ov.corrupt_file_count}</dd>
          <dt>Storage</dt><dd>${T.formatBytes(ov.storage_bytes_used)}</dd>
          <dt>Version</dt><dd>${T.escapeHtml(ov.version || "—")}</dd>
        </dl>`;
      system.innerHTML = `
        <dl class="admin-dl">
          <dt>Host</dt><dd>${T.escapeHtml(sys.hostname)}</dd>
          <dt>OS</dt><dd>${T.escapeHtml(sys.os)} / ${T.escapeHtml(sys.arch)}</dd>
          <dt>Load (1m)</dt><dd>${sys.load_average_1m.toFixed(2)}</dd>
          <dt>Memory</dt><dd>${T.formatBytes(sys.available_memory_bytes)} free / ${T.formatBytes(sys.total_memory_bytes)}</dd>
          <dt>Data disk</dt><dd>${T.formatBytes(sys.data_dir_free_bytes)} free / ${T.formatBytes(sys.data_dir_total_bytes)}</dd>
        </dl>`;
      usersEl.innerHTML =
        (users.users || [])
          .map(
            (u) =>
              `<div class="admin-row"><strong>${T.escapeHtml(u.name)}</strong> <span class="tag">${T.escapeHtml(u.role)}</span></div>`
          )
          .join("") || "<p>No users</p>";
      devicesEl.innerHTML =
        (devices.devices || [])
          .map(
            (d) =>
              `<div class="admin-row"><strong>${T.escapeHtml(d.device_name || d.device_token?.slice(0, 8) || "device")}</strong> <span class="mono">${T.escapeHtml(d.user_name || "")}</span></div>`
          )
          .join("") || "<p>No trusted devices</p>";
    } catch (e) {
      overview.textContent = e.message;
    }
  };

  T.adminCleanup = async function adminCleanup(kind) {
    const path = kind === "temp" ? "/admin/cleanup/temp" : "/admin/cleanup/sessions";
    try {
      const res = await T.api(path, { method: "POST" });
      T.showBanner(
        kind === "temp"
          ? `Removed ${res.removed ?? 0} temp file(s)`
          : res.message || "Cleanup requested",
        "success"
      );
      T.loadAdmin();
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  };

  T.deleteFile = async function deleteFile(id) {
    if (!confirm("Delete this object permanently?")) return;
    try {
      await T.api("/files/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("Object deleted", "success");
      T.loadFiles();
      T.loadFolderTree();
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  };

  T.deleteNote = async function deleteNote(id) {
    if (!confirm("Delete this note?")) return;
    try {
      await T.api("/notes/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("Note deleted", "success");
      T.loadNotes();
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  };

  T.openNoteDialog = function openNoteDialog(note) {
    T.editingNoteId = note ? note.id : null;
    T.$("#note-dialog-title").textContent = note ? "Edit note" : "New note";
    T.$("#note-title-input").value = note ? note.title || "" : "";
    T.$("#note-body-input").value = note ? note.body || "" : "";
    T.$("#note-dialog").showModal();
  };

  T.saveNote = async function saveNote() {
    const title = T.$("#note-title-input").value.trim();
    const body = T.$("#note-body-input").value;
    const payload = { body };
    if (title) payload.title = title;
    try {
      if (T.editingNoteId) {
        await T.api("/notes/" + encodeURIComponent(T.editingNoteId), {
          method: "PUT",
          body: JSON.stringify(payload),
        });
      } else {
        await T.api("/notes", { method: "POST", body: JSON.stringify(payload) });
      }
      T.$("#note-dialog").close();
      T.showBanner("Note saved", "success");
      T.loadNotes();
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  };

  T.openNote = async function openNote(id) {
    try {
      const note = await T.api("/notes/" + encodeURIComponent(id));
      T.openNoteDialog(note);
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  };
})(window.Tssp);
