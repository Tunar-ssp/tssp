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
    const details = T.$("#overview-details");
    grid.innerHTML = '<div class="metric-card"><span class="label">Loading…</span></div>';
    if (details) details.innerHTML = "";
    try {
      const [status, publicFiles, recentFiles] = await Promise.allSettled([
        T.api("/status"),
        T.api("/public/files"),
        T.api("/files?limit=8"),
      ]);
      if (status.status !== "fulfilled") throw status.reason;
      const s = status.value;
      const publicCount =
        publicFiles.status === "fulfilled" ? (publicFiles.value.files || []).length : 0;
      const recentUploads =
        recentFiles.status === "fulfilled" ? (recentFiles.value.files || []) : [];

      const health = s.status === "ok" ? "Healthy" : T.escapeHtml(s.status || "—");
      const healthClass = s.status === "ok" ? "metric-health-ok" : "metric-health-warn";

      grid.innerHTML = [
        metricCard("Files", s.file_count, "total objects stored"),
        metricCard("Notes", s.note_count, "markdown notes"),
        metricCard("Storage", T.formatBytes(s.storage_bytes_used), "used"),
        metricCard("Public", publicCount, "publicly shared"),
        metricCard("Pinned", s.pinned_count, "files + notes"),
        metricCard("Tags", s.tag_count, "unique tags"),
        metricCard("Uploaded (24h)", s.recent_upload_count_24h ?? "—", "recent"),
        metricCard("Uptime", T.formatUptime(s.uptime_seconds), "since last restart"),
        `<div class="metric-card metric-health ${healthClass}"><div class="label">Health</div><div class="value">${health}</div></div>`,
      ].join("");

      if (details && recentUploads.length) {
        details.innerHTML = `<div class="overview-section"><h3>Recent uploads</h3><div class="recent-uploads">${
          recentUploads.slice(0, 8).map((f) =>
            `<div class="recent-upload-row">
              <span class="recent-upload-name">${T.escapeHtml(f.name || f.id)}</span>
              <span class="recent-upload-meta">${T.escapeHtml(T.formatBytes(f.size_bytes))} · ${T.escapeHtml(T.formatDate(f.uploaded_at))}</span>
            </div>`
          ).join("")
        }</div></div>`;
      }
    } catch (error) {
      grid.innerHTML = `<div class="metric-card"><span class="label">Error</span><div class="value">${T.escapeHtml(error.message)}</div></div>`;
    }
  };

  function metricCard(label, value, sub) {
    return `<div class="metric-card">
      <div class="label">${T.escapeHtml(label)}</div>
      <div class="value">${T.escapeHtml(String(value ?? "—"))}</div>
      ${sub ? `<div class="metric-sub">${T.escapeHtml(sub)}</div>` : ""}
    </div>`;
  }

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
          const type = result.type || "item";
          const name = T.escapeHtml(result.title || result.name || result.id);
          const id = T.escapeHtml(result.id);
          const vis = result.visibility != null ? T.stateBadge(result.visibility) : "";
          const tags = T.tagsHtml(result.tags);
          let actions = "";
          let detail = "";
          let extra = "";
          if (type === "file") {
            detail = T.escapeHtml((result.folder_path || "Bucket root") + " · " + T.formatBytes(result.size_bytes));
            extra = `${vis}${tags}`;
            actions = `<button type="button" class="btn btn-text btn-sm" data-preview-file="${id}">Preview</button><a class="btn btn-text btn-sm" href="${T.fileDownloadUrl(result.id)}" download>Download</a>`;
          } else if (type === "note") {
            detail = T.escapeHtml((result.body || "").trim().replace(/^#+\s+/gm, "").slice(0, 120));
            extra = tags;
            actions = `<button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Open</button>`;
          } else if (type === "workspace") {
            detail = T.escapeHtml((result.body || "").slice(0, 120));
            extra = `<span class="type-pill">${T.escapeHtml(result.language || "text")}</span>`;
            actions = `<button type="button" class="btn btn-text btn-sm" data-ws-edit="${id}">Open</button>`;
          }
          const typeLabel = { file: "File", note: "Note", workspace: "Workspace" }[type] || type;
          return `<tr>
            <td><span class="search-result-type search-type-${T.escapeHtml(type)}">${T.escapeHtml(typeLabel)}</span></td>
            <td><div class="search-result-name"><strong>${name}</strong></div>${detail ? `<div class="row-meta">${detail}</div>` : ""}</td>
            <td>${extra}</td>
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
              <div class="image-card-overlay">
                <span class="image-card-overlay-icon">⤢</span>
              </div>
            </button>
            <div class="media-card-footer">
              <strong class="media-card-name" title="${T.escapeHtml(file.name)}">${T.escapeHtml(file.name)}</strong>
              <div class="media-card-meta">
                <span>${T.escapeHtml(T.formatBytes(file.size_bytes))}</span>
                ${T.stateBadge(file.visibility)}
              </div>
              <div class="media-card-actions">
                <button type="button" class="btn btn-text btn-sm" data-vis="${id}" data-v="${nextVis}">${nextVis === "public" ? "Make public" : "Make private"}</button>
                <a class="btn btn-text btn-sm" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
                <button type="button" class="btn btn-text btn-sm btn-danger" data-delete-file="${id}">Delete</button>
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
    const container = T.$("#workspaces-grid");
    if (!container) return;
    container.innerHTML = '<div class="notes-loading">Loading workspaces…</div>';
    try {
      const data = await T.api("/workspaces");
      const items = data.workspaces || [];
      if (!items.length) {
        container.innerHTML = '<div class="notes-empty-state">No workspaces yet. Create one to store scripts and text files.</div>';
        return;
      }
      container.innerHTML = `<div class="workspace-cards">${items
        .map((workspace) => {
          const id = T.escapeHtml(workspace.id);
          const lineCount = (workspace.body || "").split("\n").length;
          const preview = (workspace.body || "").trim().slice(0, 160);
          return `<article class="workspace-card">
            <div class="workspace-card-head">
              <div class="workspace-card-title-row">
                <strong class="workspace-card-name">${T.escapeHtml(workspace.name)}</strong>
                <span class="type-pill">${T.escapeHtml(workspace.language)}</span>
              </div>
              <div class="workspace-card-meta">${lineCount} lines · Updated ${T.escapeHtml(T.formatDate(workspace.updated_at))}</div>
            </div>
            ${preview ? `<pre class="workspace-card-preview">${T.escapeHtml(preview)}</pre>` : ""}
            <div class="workspace-card-actions">
              <button type="button" class="btn btn-secondary btn-sm" data-ws-edit="${id}">Edit</button>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-ws-del="${id}">Delete</button>
            </div>
          </article>`;
        })
        .join("")}</div>`;
    } catch (error) {
      container.innerHTML = `<div class="notes-empty-state">${T.escapeHtml(error.message)}</div>`;
    }
  };

  T.openWorkspaceDialog = function openWorkspaceDialog(workspace, options) {
    T.workspaceDialogSource = options?.source || "workspaces";
    T.editingWorkspaceId = workspace ? workspace.id : null;
    T.$("#workspace-dialog-title").textContent = workspace ? "Edit workspace" : "New workspace";
    T.$("#workspace-name-input").value = workspace ? workspace.name || "" : "";
    const langEl = T.$("#workspace-language-input");
    if (langEl) langEl.value = workspace ? workspace.language || "text" : "text";
    T.$("#workspace-body-input").value = workspace ? workspace.body || "" : "";
    T.$("#workspace-dialog").showModal();
  };

  T.saveWorkspace = async function saveWorkspace() {
    const payload = {
      name: T.$("#workspace-name-input").value.trim(),
      language: (T.$("#workspace-language-input").value || "text").trim(),
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
