window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  // Shared rendering helpers used across modules.

  T.tagsHtml = function tagsHtml(tags) {
    return (tags || [])
      .map((tag) => `<span class="tag">${T.escapeHtml(tag)}</span>`)
      .join("");
  };

  T.stateBadge = function stateBadge(value) {
    const isPublic = value === "public";
    return `<span class="state-badge ${isPublic ? "public" : "private"}">${isPublic ? "Public" : "Private"}</span>`;
  };

  T.publicLink = function publicLink(file) {
    return file.public_token ? `${window.location.origin}/p/${file.public_token}` : "";
  };

  T.tableMessage = function tableMessage(columns, message) {
    return `<tr><td colspan="${columns}" class="table-empty">${T.escapeHtml(message)}</td></tr>`;
  };

  // Overview

  T.loadOverview = async function loadOverview() {
    const grid = T.$("#metric-grid");
    grid.innerHTML = '<div class="metric-card"><span class="label">Loading…</span></div>';
    try {
      const [status, publicFiles] = await Promise.allSettled([
        T.api("/status"),
        T.api("/public/files"),
      ]);
      if (status.status !== "fulfilled") throw status.reason;
      const s = status.value;
      const publicCount =
        publicFiles.status === "fulfilled" ? (publicFiles.value.files || []).length : 0;
      const metrics = [
        ["Files", s.file_count],
        ["Notes", s.note_count],
        ["Tags", s.tag_count],
        ["Pinned", s.pinned_count],
        ["Public files", publicCount],
        ["Uploads 24h", s.recent_upload_count_24h],
        ["Storage", T.formatBytes(s.storage_bytes_used)],
        ["Uptime", T.formatUptime(s.uptime_seconds)],
        ["Health", s.status],
      ];
      grid.innerHTML = metrics
        .map(
          ([label, value]) =>
            `<div class="metric-card"><div class="label">${T.escapeHtml(label)}</div><div class="value">${T.escapeHtml(String(value ?? "—"))}</div></div>`
        )
        .join("");
    } catch (error) {
      grid.innerHTML = `<div class="metric-card"><span class="label">Error</span><div class="value">${T.escapeHtml(error.message)}</div></div>`;
    }
  };

  // Search

  T.searchQueryString = function searchQueryString(q) {
    const params = new URLSearchParams({ q, limit: "50" });
    const kind = T.$("#search-kind")?.value;
    if (kind && kind !== "all") params.set("kind", kind);
    const tag = T.$("#search-tag")?.value.trim();
    if (tag) params.set("tag", tag);
    const mime = T.$("#search-type")?.value.trim();
    if (mime) params.set("type", mime);
    const visibility = T.$("#search-visibility")?.value;
    if (visibility) params.set("visibility", visibility);
    if (T.$("#search-pinned")?.checked) params.set("pinned", "true");
    return params.toString();
  };

  T.runSearch = async function runSearch(q) {
    const body = T.$("#search-body");
    const sub = T.$("#search-subtitle");
    if (!q || q.length < 1) {
      sub.textContent = "Type in the search bar above";
      body.innerHTML = T.tableMessage(4, "Enter a query to search");
      return;
    }
    const filterParts = [];
    const kind = T.$("#search-kind")?.value;
    if (kind && kind !== "all") filterParts.push(kind);
    if (T.$("#search-tag")?.value.trim()) filterParts.push("tag");
    if (T.$("#search-type")?.value.trim()) filterParts.push("type");
    if (T.$("#search-visibility")?.value) filterParts.push(T.$("#search-visibility").value);
    if (T.$("#search-pinned")?.checked) filterParts.push("pinned");
    sub.textContent =
      filterParts.length > 0
        ? `Results for "${q}" (${filterParts.join(", ")})`
        : `Results for "${q}"`;
    body.innerHTML = T.tableMessage(4, "Searching…");
    T.setView("search");
    try {
      const searchData = await T.api("/search?" + T.searchQueryString(q));
      const results = searchData.results || [];
      if (!results.length) {
        body.innerHTML = T.tableMessage(4, "No matches");
        return;
      }
      body.innerHTML = results
        .map((result) => {
          const type = T.escapeHtml(result.type || "item");
          const name = T.escapeHtml(result.title || result.name || result.id);
          const id = T.escapeHtml(result.id);
          const vis = result.visibility != null ? T.stateBadge(result.visibility) : "";
          const tags = T.tagsHtml(result.tags);
          let actions = "";
          let detail = "";
          if (result.type === "file") {
            detail = `${T.formatBytes(result.size_bytes)} · ${T.escapeHtml(result.folder_path || "Bucket root")} ${vis} ${tags}`;
            actions = `<button type="button" class="btn btn-text btn-sm" data-preview-file="${id}">Preview</button><a class="btn btn-text btn-sm" href="${T.fileDownloadUrl(result.id)}" download>Download</a>`;
          } else if (result.type === "note") {
            detail = (result.body || "").slice(0, 100);
            actions = `<button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Open</button>`;
          } else if (result.type === "workspace") {
            detail = `${result.language || "text"} · ${(result.body || "").slice(0, 100)}`;
            actions = `<button type="button" class="btn btn-text btn-sm" data-ws-edit="${id}">Open</button>`;
          }
          return `<tr>
            <td><span class="tag">${type}</span></td>
            <td><strong>${name}</strong></td>
            <td>${T.escapeHtml(detail || "—")}</td>
            <td class="col-actions">${actions}</td>
          </tr>`;
        })
        .join("");
    } catch (error) {
      body.innerHTML = T.tableMessage(4, error.message);
    }
  };

  // Images

  T.loadImages = async function loadImages() {
    const grid = T.$("#image-grid");
    grid.innerHTML = '<div class="empty-state compact">Loading images…</div>';
    try {
      const data = await T.api("/files?type=image/&limit=200");
      const files = data.files || [];
      if (!files.length) {
        grid.innerHTML =
          '<div class="empty-state"><strong>No images yet</strong><p>Upload photos or screenshots and they will appear here as a gallery.</p></div>';
        return;
      }
      grid.innerHTML = files
        .map((file) => {
          const id = T.escapeHtml(file.id);
          const nextVis = file.visibility === "public" ? "private" : "public";
          return `<article class="image-card">
            <button type="button" class="media-open" data-preview-file="${id}" aria-label="Preview ${T.escapeHtml(file.name)}">
              <img src="${T.fileThumbnailUrl(file.id)}" alt="${T.escapeHtml(file.name)}" loading="lazy" onerror="this.src='${T.fileInlineUrl(file.id)}'">
            </button>
            <div class="media-card-footer">
              <strong>${T.escapeHtml(file.name)}</strong>
              <span>${T.escapeHtml(T.formatBytes(file.size_bytes))} · ${T.stateBadge(file.visibility)}</span>
              <div>
                <button type="button" class="btn btn-text btn-sm" data-vis="${id}" data-v="${nextVis}">${nextVis === "public" ? "Public" : "Private"}</button>
                <a class="btn btn-text btn-sm" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
              </div>
            </div>
          </article>`;
        })
        .join("");
    } catch (error) {
      grid.innerHTML = `<div class="empty-state error">${T.escapeHtml(error.message)}</div>`;
    }
  };

  // Typed file lists (audio, video, etc.)

  T.loadTypedFiles = async function loadTypedFiles(mimePrefix, bodyId) {
    const body = T.$(`#${bodyId}`);
    body.innerHTML = T.tableMessage(4, "Loading objects…");
    const params = new URLSearchParams({ limit: "200", type: `${mimePrefix}/` });
    try {
      const data = await T.api("/files?" + params.toString());
      const files = data.files || [];
      if (!files.length) {
        body.innerHTML = T.tableMessage(4, `No ${mimePrefix} objects yet.`);
        return;
      }
      body.innerHTML = files
        .map((file) => {
          const id = T.escapeHtml(file.id);
          const nextVis = file.visibility === "public" ? "private" : "public";
          return `<tr>
            <td><strong>${T.escapeHtml(file.name)}</strong><div class="row-meta">${T.stateBadge(file.visibility)}${T.tagsHtml(file.tags)}</div></td>
            <td class="mono">${T.escapeHtml(T.formatBytes(file.size_bytes))}</td>
            <td>${T.escapeHtml(T.formatDate(file.uploaded_at))}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-preview-file="${id}">Preview</button>
              <button type="button" class="btn btn-text btn-sm" data-vis="${id}" data-v="${nextVis}">${nextVis === "public" ? "Public" : "Private"}</button>
              <a class="btn btn-text btn-sm" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
            </td>
          </tr>`;
        })
        .join("");
    } catch (error) {
      body.innerHTML = T.tableMessage(4, error.message);
    }
  };

  // Documents

  T.loadDocuments = async function loadDocuments() {
    const body = T.$("#documents-body");
    body.innerHTML = T.tableMessage(6, "Loading documents…");
    try {
      const data = await T.api("/files?limit=200");
      const files = (data.files || []).filter((file) => {
        const mime = file.mime_type || "";
        return (
          (mime.startsWith("application/") || mime.startsWith("text/")) &&
          !mime.startsWith("image/") &&
          !mime.startsWith("video/")
        );
      });
      if (!files.length) {
        body.innerHTML = T.tableMessage(6, "No documents yet.");
        return;
      }
      body.innerHTML = files
        .map((file) => {
          const id = T.escapeHtml(file.id);
          return `<tr>
            <td><strong>${T.escapeHtml(file.name)}</strong><div class="row-meta">${T.tagsHtml(file.tags)}${T.stateBadge(file.visibility)}</div></td>
            <td><span class="type-pill">${T.escapeHtml(T.fileKind(file))}</span><span class="mono muted">${T.escapeHtml(file.mime_type)}</span></td>
            <td class="mono">${T.escapeHtml(T.formatBytes(file.size_bytes))}</td>
            <td>${T.escapeHtml(file.folder_path || "Bucket root")}</td>
            <td>${T.escapeHtml(T.formatDate(file.uploaded_at))}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-preview-file="${id}">Preview</button>
              <a class="btn btn-text btn-sm" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
            </td>
          </tr>`;
        })
        .join("");
    } catch (error) {
      body.innerHTML = T.tableMessage(6, error.message);
    }
  };

  // Public files

  T.loadPublic = async function loadPublic() {
    const body = T.$("#public-body");
    body.innerHTML = T.tableMessage(4, "Loading public files…");
    try {
      const data = await T.api("/public/files");
      const files = data.files || [];
      if (!files.length) {
        body.innerHTML = T.tableMessage(4, "No public files. Make an object public to share it here.");
        return;
      }
      body.innerHTML = files
        .map((file) => {
          const id = T.escapeHtml(file.id);
          const link = T.publicLink(file);
          return `<tr>
            <td><strong>${T.escapeHtml(file.name)}</strong><div class="row-meta">${T.escapeHtml(file.folder_path || "Bucket root")}</div></td>
            <td class="mono">${T.escapeHtml(T.formatBytes(file.size_bytes))}</td>
            <td><button type="button" class="link-button mono" data-copy-link="${T.escapeHtml(link)}">${T.escapeHtml(link.replace(window.location.origin, ""))}</button></td>
            <td class="col-actions">
              <a class="btn btn-text btn-sm" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
              <button type="button" class="btn btn-text btn-sm" data-vis="${id}" data-v="private">Make private</button>
            </td>
          </tr>`;
        })
        .join("");
    } catch (error) {
      body.innerHTML = T.tableMessage(4, error.message);
    }
  };

  // Workspaces

  T.loadWorkspaces = async function loadWorkspaces() {
    const body = T.$("#workspaces-body");
    body.innerHTML = T.tableMessage(4, "Loading workspaces…");
    try {
      const data = await T.api("/workspaces");
      const items = data.workspaces || [];
      if (!items.length) {
        body.innerHTML = T.tableMessage(4, "No workspaces yet.");
        return;
      }
      body.innerHTML = items
        .map(
          (workspace) => `<tr>
            <td><strong>${T.escapeHtml(workspace.name)}</strong><div class="row-meta mono">${T.escapeHtml(workspace.id)}</div></td>
            <td><span class="type-pill">${T.escapeHtml(workspace.language)}</span></td>
            <td>${T.escapeHtml(T.formatDate(workspace.updated_at))}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-ws-edit="${T.escapeHtml(workspace.id)}">Edit</button>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-ws-del="${T.escapeHtml(workspace.id)}">Delete</button>
            </td>
          </tr>`
        )
        .join("");
    } catch (error) {
      body.innerHTML = T.tableMessage(4, error.message);
    }
  };

  T.openWorkspaceDialog = function openWorkspaceDialog(workspace, options) {
    T.workspaceDialogSource = options?.source || "workspaces";
    T.editingWorkspaceId = workspace ? workspace.id : null;
    T.$("#workspace-dialog-title").textContent = workspace ? "Edit workspace" : "New workspace";
    T.$("#workspace-name-input").value = workspace ? workspace.name || "" : "";
    T.$("#workspace-language-input").value = workspace ? workspace.language || "text" : "text";
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
