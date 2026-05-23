window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  function tagsHtml(tags) {
    return (tags || [])
      .map((tag) => `<span class="tag">${T.escapeHtml(tag)}</span>`)
      .join("");
  }

  function stateBadge(value) {
    const publicState = value === "public";
    return `<span class="state-badge ${publicState ? "public" : "private"}">${publicState ? "Public" : "Private"}</span>`;
  }

  function publicLink(file) {
    return file.public_token ? `${window.location.origin}/p/${file.public_token}` : "";
  }

  function tableMessage(columns, message) {
    return `<tr><td colspan="${columns}" class="table-empty">${T.escapeHtml(message)}</td></tr>`;
  }

  function selectedFiles() {
    return T.currentFiles.filter((file) => T.selectedFileIds.has(file.id));
  }

  function updateObjectSummary(files) {
    const total = files.reduce((sum, file) => sum + Number(file.size_bytes || 0), 0);
    const publicCount = files.filter((file) => file.visibility === "public").length;
    T.$("#object-folder-label").textContent = T.currentFolder || "Bucket root";
    T.$("#object-count").textContent = String(files.length);
    T.$("#object-storage").textContent = T.formatBytes(total);
    T.$("#object-public").textContent = String(publicCount);
  }

  function fileRow(file) {
    const id = T.escapeHtml(file.id);
    const checked = T.selectedFileIds.has(file.id) ? "checked" : "";
    const pin = file.pinned ? '<span class="pin" title="Pinned">★</span>' : "";
    const folder = file.folder_path
      ? `<span class="tag">${T.escapeHtml(file.folder_path)}</span>`
      : "";
    const link = publicLink(file);
    const nextVisibility = file.visibility === "public" ? "private" : "public";
    return `<tr data-file-row="${id}">
      <td class="col-select"><input type="checkbox" data-file-select="${id}" ${checked} aria-label="Select ${T.escapeHtml(file.name || file.id)}"></td>
      <td>
        <div class="file-name">${pin}<strong>${T.escapeHtml(file.name || file.id)}</strong></div>
        <div class="row-meta">${folder}${tagsHtml(file.tags)}${stateBadge(file.visibility)}</div>
      </td>
      <td class="mono">${T.escapeHtml(T.formatBytes(file.size_bytes))}</td>
      <td><span class="type-pill">${T.escapeHtml(T.fileKind(file))}</span><span class="mono muted">${T.escapeHtml(file.mime_type || "—")}</span></td>
      <td>${T.escapeHtml(T.formatDate(file.uploaded_at))}</td>
      <td class="col-actions">
        <button type="button" class="btn btn-text btn-sm" data-preview-file="${id}">Preview</button>
        <button type="button" class="btn btn-text btn-sm" data-rename-file="${id}">Rename</button>
        <button type="button" class="btn btn-text btn-sm" data-pin-file="${id}" data-pinned="${file.pinned ? "1" : "0"}">${file.pinned ? "Unpin" : "Pin"}</button>
        <button type="button" class="btn btn-text btn-sm" data-vis="${id}" data-v="${nextVisibility}">${nextVisibility === "public" ? "Public" : "Private"}</button>
        ${link ? `<button type="button" class="btn btn-text btn-sm" data-copy-link="${T.escapeHtml(link)}">Copy link</button>` : ""}
        <a class="btn btn-text btn-sm" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
        <button type="button" class="btn btn-text btn-sm btn-danger" data-delete-file="${id}">Delete</button>
      </td>
    </tr>`;
  }

  T.updateFileSelection = function updateFileSelection() {
    const selected = selectedFiles();
    const toolbar = T.$("#bulk-toolbar");
    const count = T.$("#bulk-count");
    if (toolbar && count) {
      toolbar.classList.toggle("hidden", selected.length === 0);
      count.textContent = `${selected.length} selected`;
    }

    const selectAll = T.$("#select-all-files");
    if (selectAll) {
      selectAll.checked =
        T.currentFiles.length > 0 &&
        T.currentFiles.every((file) => T.selectedFileIds.has(file.id));
      selectAll.indeterminate =
        selected.length > 0 && selected.length < T.currentFiles.length;
    }

    const details = T.$("#details-panel");
    if (!details) return;
    if (selected.length === 0) {
      details.innerHTML =
        '<h3>Details</h3><p class="muted">Select an object to inspect metadata and actions.</p>';
      return;
    }
    if (selected.length > 1) {
      const total = selected.reduce((sum, file) => sum + Number(file.size_bytes || 0), 0);
      details.innerHTML = `<h3>${selected.length} objects</h3>
        <dl class="admin-dl">
          <dt>Total size</dt><dd>${T.escapeHtml(T.formatBytes(total))}</dd>
          <dt>Public</dt><dd>${selected.filter((file) => file.visibility === "public").length}</dd>
          <dt>Pinned</dt><dd>${selected.filter((file) => file.pinned).length}</dd>
        </dl>`;
      return;
    }

    const file = selected[0];
    const link = publicLink(file);
    details.innerHTML = `<h3>${T.escapeHtml(file.name)}</h3>
      <dl class="admin-dl">
        <dt>ID</dt><dd class="mono">${T.escapeHtml(file.id)}</dd>
        <dt>Type</dt><dd>${T.escapeHtml(file.mime_type || "—")}</dd>
        <dt>Size</dt><dd>${T.escapeHtml(T.formatBytes(file.size_bytes))}</dd>
        <dt>Folder</dt><dd>${T.escapeHtml(file.folder_path || "Bucket root")}</dd>
        <dt>Uploaded</dt><dd>${T.escapeHtml(T.formatDate(file.uploaded_at))}</dd>
        <dt>Visibility</dt><dd>${stateBadge(file.visibility)}</dd>
        <dt>Hash</dt><dd class="mono hash">${T.escapeHtml(file.content_hash || "—")}</dd>
      </dl>
      <div class="details-actions">
        <button type="button" class="btn btn-secondary btn-sm" data-preview-file="${T.escapeHtml(file.id)}">Preview</button>
        <a class="btn btn-secondary btn-sm" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
        ${link ? `<button type="button" class="btn btn-secondary btn-sm" data-copy-link="${T.escapeHtml(link)}">Copy public link</button>` : ""}
      </div>`;
  };

  T.setSelectedFile = function setSelectedFile(id, checked) {
    if (checked) T.selectedFileIds.add(id);
    else T.selectedFileIds.delete(id);
    T.updateFileSelection();
  };

  T.setAllVisibleFilesSelected = function setAllVisibleFilesSelected(checked) {
    for (const file of T.currentFiles) {
      if (checked) T.selectedFileIds.add(file.id);
      else T.selectedFileIds.delete(file.id);
    }
    T.$$("#files-body [data-file-select]").forEach((input) => {
      input.checked = checked;
    });
    T.updateFileSelection();
  };

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

  T.loadFolderTree = async function loadFolderTree() {
    const tree = T.$("#folder-tree");
    if (!tree) return;
    try {
      const data = await T.api("/admin/folders");
      const folders = data.folders || [];
      const root = folders.find((folder) => !folder.path);
      const items = [{ path: "", label: "Bucket root", count: root?.file_count || 0 }];
      for (const folder of folders) {
        if (folder.path) {
          items.push({ path: folder.path, label: folder.path, count: folder.file_count });
        }
      }
      tree.innerHTML = items
        .map(
          (folder) =>
            `<button type="button" data-folder="${T.escapeHtml(folder.path)}" class="${folder.path === T.currentFolder ? "active" : ""}">
              <span>${T.escapeHtml(folder.label)}</span><span class="mono">${folder.count}</span>
            </button>`
        )
        .join("");
      tree.querySelectorAll("button").forEach((button) => {
        button.addEventListener("click", () => {
          T.currentFolder = button.dataset.folder || "";
          T.$("#breadcrumb-folder").textContent = T.currentFolder || "default";
          T.$("#upload-folder").value = T.currentFolder;
          tree.querySelectorAll("button").forEach((item) => item.classList.remove("active"));
          button.classList.add("active");
          T.loadFiles();
        });
      });
    } catch {
      try {
        const data = await T.api("/files?limit=500");
        const counts = new Map([["", 0]]);
        for (const file of data.files || []) {
          const folder = file.folder_path || "";
          counts.set(folder, (counts.get(folder) || 0) + 1);
          counts.set("", (counts.get("") || 0) + 1);
        }
        const items = [...counts.entries()].map(([path, count]) => ({
          path,
          label: path || "Bucket root",
          count,
        }));
        tree.innerHTML = items
          .map(
            (folder) =>
              `<button type="button" data-folder="${T.escapeHtml(folder.path)}" class="${folder.path === T.currentFolder ? "active" : ""}">
                <span>${T.escapeHtml(folder.label)}</span><span class="mono">${folder.count}</span>
              </button>`
          )
          .join("");
        tree.querySelectorAll("button").forEach((button) => {
          button.addEventListener("click", () => {
            T.currentFolder = button.dataset.folder || "";
            T.$("#breadcrumb-folder").textContent = T.currentFolder || "default";
            T.$("#upload-folder").value = T.currentFolder;
            tree.querySelectorAll("button").forEach((item) => item.classList.remove("active"));
            button.classList.add("active");
            T.loadFiles();
          });
        });
      } catch {
        tree.innerHTML =
          '<button type="button" data-folder="" class="active"><span>Bucket root</span><span class="mono">—</span></button>';
      }
    }
  };

  T.loadFiles = async function loadFiles() {
    const body = T.$("#files-body");
    body.innerHTML = tableMessage(6, "Loading objects…");
    const params = new URLSearchParams({ limit: "200" });
    if (T.$("#pinned-only")?.checked) params.set("pinned", "true");
    if (T.currentFolder) params.set("folder", T.currentFolder);
    try {
      const data = await T.api("/files?" + params.toString());
      T.currentFiles = data.files || [];
      T.selectedFileIds.clear();
      updateObjectSummary(T.currentFiles);
      if (!T.currentFiles.length) {
        body.innerHTML = tableMessage(
          6,
          "No objects in this folder. Drop files above or click Upload."
        );
        T.updateFileSelection();
        return;
      }
      body.innerHTML = T.currentFiles.map(fileRow).join("");
      T.updateFileSelection();
    } catch (error) {
      body.innerHTML = tableMessage(6, error.message);
      T.currentFiles = [];
      updateObjectSummary([]);
      T.updateFileSelection();
    }
  };

  T.renameFile = async function renameFile(id) {
    const file = T.currentFiles.find((item) => item.id === id);
    const next = prompt("New object name", file?.name || "");
    if (!next || next.trim() === file?.name) return;
    try {
      await T.api("/files/" + encodeURIComponent(id), {
        method: "PATCH",
        body: JSON.stringify({ name: next.trim() }),
      });
      T.showBanner("Object renamed", "success");
      T.loadFiles();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.bulkFileAction = async function bulkFileAction(action) {
    const ids = [...T.selectedFileIds];
    if (!ids.length) return;
    try {
      if (action === "delete") {
        if (!confirm(`Delete ${ids.length} selected object(s)?`)) return;
        for (const id of ids) await T.api("/files/" + encodeURIComponent(id), { method: "DELETE" });
      } else if (action === "public" || action === "private") {
        await T.api("/files/visibility/bulk", {
          method: "POST",
          body: JSON.stringify({ ids, visibility: action }),
        });
      } else if (action === "pin" || action === "unpin") {
        for (const id of ids) {
          await T.api("/files/" + encodeURIComponent(id) + "/pin", {
            method: action === "pin" ? "PUT" : "DELETE",
          });
        }
      } else if (action === "tag") {
        const tags = T.tagsFromInput(prompt("Tags to add, comma-separated", ""));
        if (!tags.length) return;
        for (const id of ids) {
          await T.api("/files/" + encodeURIComponent(id) + "/tags", {
            method: "POST",
            body: JSON.stringify(tags),
          });
        }
      }
      T.showBanner("Bulk operation completed", "success");
      T.loadFiles();
      T.loadFolderTree();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.setFileVisibility = async function setFileVisibility(id, visibility) {
    try {
      await T.api("/files/" + encodeURIComponent(id) + "/visibility", {
        method: "PATCH",
        body: JSON.stringify({ visibility }),
      });
      T.showBanner(
        visibility === "public" ? "Object is now public" : "Object is now private",
        "success"
      );
      T.refreshCurrentView();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.toggleFilePin = async function toggleFilePin(id, pinned) {
    try {
      await T.api("/files/" + encodeURIComponent(id) + "/pin", {
        method: pinned ? "DELETE" : "PUT",
      });
      T.loadFiles();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.deleteFile = async function deleteFile(id) {
    if (!confirm("Delete this object permanently?")) return;
    try {
      await T.api("/files/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("Object deleted", "success");
      T.loadFiles();
      T.loadFolderTree();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.previewFile = async function previewFile(id) {
    const file =
      T.currentFiles.find((item) => item.id === id) ||
      (await T.api("/files/" + encodeURIComponent(id)).catch(() => null));
    if (!file) {
      T.showBanner("Object metadata could not be loaded", "error");
      return;
    }
    const title = T.$("#preview-title");
    const content = T.$("#preview-content");
    title.textContent = file.name || file.id;
    const inline = T.fileInlineUrl(file.id);
    if ((file.mime_type || "").startsWith("image/")) {
      content.innerHTML = `<img class="preview-media" src="${inline}" alt="${T.escapeHtml(file.name)}">`;
    } else if ((file.mime_type || "").startsWith("video/")) {
      content.innerHTML = `<video class="preview-media" src="${inline}" controls preload="metadata"></video>`;
    } else {
      content.innerHTML = `<div class="empty-state compact">
        <strong>${T.escapeHtml(T.fileKind(file))} preview unavailable</strong>
        <p>Open or download the object using the actions below.</p>
        <div class="empty-actions">
          <a class="btn btn-secondary" href="${inline}" target="_blank" rel="noopener">Open</a>
          <a class="btn btn-primary" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
        </div>
      </div>`;
    }
    T.$("#preview-dialog").showModal();
  };

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
          const visibility = file.visibility === "public" ? "private" : "public";
          return `<article class="image-card">
            <button type="button" class="media-open" data-preview-file="${id}" aria-label="Preview ${T.escapeHtml(file.name)}">
              <img src="${T.fileThumbnailUrl(file.id)}" alt="${T.escapeHtml(file.name)}" loading="lazy" onerror="this.src='${T.fileInlineUrl(file.id)}'">
            </button>
            <div class="media-card-footer">
              <strong>${T.escapeHtml(file.name)}</strong>
              <span>${T.escapeHtml(T.formatBytes(file.size_bytes))} · ${stateBadge(file.visibility)}</span>
              <div>
                <button type="button" class="btn btn-text btn-sm" data-vis="${id}" data-v="${visibility}">${visibility === "public" ? "Public" : "Private"}</button>
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

  T.loadTypedFiles = async function loadTypedFiles(mimePrefix, bodyId) {
    const body = T.$(`#${bodyId}`);
    body.innerHTML = tableMessage(4, "Loading objects…");
    const params = new URLSearchParams({ limit: "200", type: `${mimePrefix}/` });
    try {
      const data = await T.api("/files?" + params.toString());
      const files = data.files || [];
      if (!files.length) {
        body.innerHTML = tableMessage(4, `No ${mimePrefix} objects yet.`);
        return;
      }
      body.innerHTML = files
        .map((file) => {
          const id = T.escapeHtml(file.id);
          const visibility = file.visibility === "public" ? "private" : "public";
          return `<tr>
            <td><strong>${T.escapeHtml(file.name)}</strong><div class="row-meta">${stateBadge(file.visibility)}${tagsHtml(file.tags)}</div></td>
            <td class="mono">${T.escapeHtml(T.formatBytes(file.size_bytes))}</td>
            <td>${T.escapeHtml(T.formatDate(file.uploaded_at))}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-preview-file="${id}">Preview</button>
              <button type="button" class="btn btn-text btn-sm" data-vis="${id}" data-v="${visibility}">${visibility === "public" ? "Public" : "Private"}</button>
              <a class="btn btn-text btn-sm" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
            </td>
          </tr>`;
        })
        .join("");
    } catch (error) {
      body.innerHTML = tableMessage(4, error.message);
    }
  };

  T.loadDocuments = async function loadDocuments() {
    const body = T.$("#documents-body");
    body.innerHTML = tableMessage(6, "Loading documents…");
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
        body.innerHTML = tableMessage(6, "No documents yet.");
        return;
      }
      body.innerHTML = files
        .map((file) => {
          const id = T.escapeHtml(file.id);
          return `<tr>
            <td><strong>${T.escapeHtml(file.name)}</strong><div class="row-meta">${tagsHtml(file.tags)}${stateBadge(file.visibility)}</div></td>
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
      body.innerHTML = tableMessage(6, error.message);
    }
  };

  T.loadPublic = async function loadPublic() {
    const body = T.$("#public-body");
    body.innerHTML = tableMessage(4, "Loading public files…");
    try {
      const data = await T.api("/public/files");
      const files = data.files || [];
      if (!files.length) {
        body.innerHTML = tableMessage(4, "No public files. Make an object public to share it here.");
        return;
      }
      body.innerHTML = files
        .map((file) => {
          const id = T.escapeHtml(file.id);
          const link = publicLink(file);
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
      body.innerHTML = tableMessage(4, error.message);
    }
  };

  T.runSearch = async function runSearch(q) {
    const body = T.$("#search-body");
    const sub = T.$("#search-subtitle");
    if (!q || q.length < 1) {
      sub.textContent = "Type in the search bar above";
      body.innerHTML = tableMessage(4, "Enter a query to search");
      return;
    }
    sub.textContent = `Results for "${q}"`;
    body.innerHTML = tableMessage(4, "Searching…");
    T.setView("search");
    try {
      const searchData = await T.api("/search?q=" + encodeURIComponent(q) + "&limit=50");
      const results = searchData.results || [];
      if (!results.length) {
        body.innerHTML = tableMessage(4, "No matches");
        return;
      }
      body.innerHTML = results
        .map((result) => {
          const type = T.escapeHtml(result.type || "item");
          const name = T.escapeHtml(result.title || result.name || result.id);
          const id = T.escapeHtml(result.id);
          let actions = "";
          let detail = "";
          if (result.type === "file") {
            detail = `${T.formatBytes(result.size_bytes)} · ${result.folder_path || "Bucket root"}`;
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
      body.innerHTML = tableMessage(4, error.message);
    }
  };

  T.loadNotes = async function loadNotes() {
    const body = T.$("#notes-body");
    body.innerHTML = tableMessage(5, "Loading notes…");
    try {
      const data = await T.api("/notes?limit=200");
      const notes = data.notes || [];
      if (!notes.length) {
        body.innerHTML = tableMessage(5, "No notes yet. Create a markdown note to start.");
        return;
      }
      body.innerHTML = notes
        .map((note) => {
          const id = T.escapeHtml(note.id);
          const pinned = note.pinned_at != null ? '<span class="pin">★</span>' : "";
          return `<tr>
            <td><div class="file-name">${pinned}<strong>${T.escapeHtml(note.title || "Untitled")}</strong></div><div class="row-meta">${T.escapeHtml(T.formatDate(note.created_at))}</div></td>
            <td>${T.escapeHtml(T.formatDate(note.updated_at))}</td>
            <td>${tagsHtml(note.tags) || "—"}</td>
            <td class="muted">${T.escapeHtml((note.body || "").slice(0, 120))}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Edit</button>
              <button type="button" class="btn btn-text btn-sm" data-pin-note="${id}" data-pinned="${note.pinned_at != null ? "1" : "0"}">${note.pinned_at != null ? "Unpin" : "Pin"}</button>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-delete-note="${id}">Delete</button>
            </td>
          </tr>`;
        })
        .join("");
    } catch (error) {
      body.innerHTML = tableMessage(5, error.message);
    }
  };

  T.refreshNotePreview = function refreshNotePreview() {
    const preview = T.$("#note-preview");
    if (preview) preview.innerHTML = T.simpleMarkdown(T.$("#note-body-input").value);
  };

  T.openNoteDialog = function openNoteDialog(note) {
    T.editingNoteId = note ? note.id : null;
    T.editingNoteTags = note ? note.tags || [] : [];
    T.editingNotePinned = note ? note.pinned_at != null : false;
    T.$("#note-dialog-title").textContent = note ? "Edit note" : "New note";
    T.$("#note-title-input").value = note ? note.title || "" : "";
    T.$("#note-tags-input").value = T.editingNoteTags.join(", ");
    T.$("#note-pin-input").checked = T.editingNotePinned;
    T.$("#note-body-input").value = note ? note.body || "" : "";
    T.$("#note-save-status").textContent = "";
    T.refreshNotePreview();
    T.$("#note-dialog").showModal();
    T.$("#note-title-input").focus();
  };

  async function syncNoteTags(id, desiredTags) {
    const current = new Set(T.editingNoteTags.map((tag) => tag.toLocaleLowerCase()));
    const desired = new Set(desiredTags.map((tag) => tag.toLocaleLowerCase()));
    const toAdd = desiredTags.filter((tag) => !current.has(tag.toLocaleLowerCase()));
    const toRemove = T.editingNoteTags.filter((tag) => !desired.has(tag.toLocaleLowerCase()));
    if (toAdd.length) {
      await T.api("/notes/" + encodeURIComponent(id) + "/tags", {
        method: "POST",
        body: JSON.stringify(toAdd),
      });
    }
    for (const tag of toRemove) {
      await T.api(
        "/notes/" + encodeURIComponent(id) + "/tags/" + encodeURIComponent(tag),
        { method: "DELETE" }
      );
    }
  }

  async function syncNotePin(id, desired) {
    if (desired === T.editingNotePinned) return;
    await T.api("/notes/" + encodeURIComponent(id) + "/pin", {
      method: desired ? "PUT" : "DELETE",
    });
  }

  T.saveNote = async function saveNote() {
    const title = T.$("#note-title-input").value.trim();
    const body = T.$("#note-body-input").value;
    const tags = T.tagsFromInput(T.$("#note-tags-input").value);
    const pin = T.$("#note-pin-input").checked;
    const payload = { body };
    if (title) payload.title = title;
    T.$("#note-save-status").textContent = "Saving…";
    try {
      let saved;
      if (T.editingNoteId) {
        saved = await T.api("/notes/" + encodeURIComponent(T.editingNoteId), {
          method: "PUT",
          body: JSON.stringify(payload),
        });
        await syncNoteTags(T.editingNoteId, tags);
        await syncNotePin(T.editingNoteId, pin);
      } else {
        saved = await T.api("/notes", {
          method: "POST",
          body: JSON.stringify({ ...payload, tags, pin }),
        });
      }
      T.$("#note-save-status").textContent = "Saved";
      T.$("#note-dialog").close();
      T.showBanner("Note saved", "success");
      T.loadNotes();
      return saved;
    } catch (error) {
      T.$("#note-save-status").textContent = "";
      T.showBanner(error.message, "error");
      return null;
    }
  };

  T.openNote = async function openNote(id) {
    try {
      const note = await T.api("/notes/" + encodeURIComponent(id));
      T.openNoteDialog(note);
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.toggleNotePin = async function toggleNotePin(id, pinned) {
    try {
      await T.api("/notes/" + encodeURIComponent(id) + "/pin", {
        method: pinned ? "DELETE" : "PUT",
      });
      T.loadNotes();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.deleteNote = async function deleteNote(id) {
    if (!confirm("Delete this note?")) return;
    try {
      await T.api("/notes/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("Note deleted", "success");
      T.loadNotes();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.loadWorkspaces = async function loadWorkspaces() {
    const body = T.$("#workspaces-body");
    body.innerHTML = tableMessage(4, "Loading workspaces…");
    try {
      const data = await T.api("/workspaces");
      const items = data.workspaces || [];
      if (!items.length) {
        body.innerHTML = tableMessage(4, "No workspaces yet.");
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
      body.innerHTML = tableMessage(4, error.message);
    }
  };

  T.openWorkspaceDialog = function openWorkspaceDialog(workspace) {
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
      if (T.editingWorkspaceId) {
        await T.api("/workspaces/" + encodeURIComponent(T.editingWorkspaceId), {
          method: "PUT",
          body: JSON.stringify(payload),
        });
      } else {
        await T.api("/workspaces", { method: "POST", body: JSON.stringify(payload) });
      }
      T.$("#workspace-dialog").close();
      T.showBanner("Workspace saved", "success");
      T.loadWorkspaces();
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

  function renderAdminUsers(users) {
    if (!users.length) return '<div class="empty-state compact">No users configured.</div>';
    return `<table class="data-table compact-table">
      <thead><tr><th>Name</th><th>Role</th><th>Created</th><th class="col-actions">Actions</th></tr></thead>
      <tbody>${users
        .map((user) => {
          const nextRole = user.role === "admin" ? "user" : "admin";
          return `<tr>
            <td><strong>${T.escapeHtml(user.name)}</strong><div class="row-meta mono">${T.escapeHtml(user.id)}</div></td>
            <td><span class="state-badge ${user.role === "admin" ? "public" : "private"}">${T.escapeHtml(user.role)}</span></td>
            <td>${T.escapeHtml(T.formatDate(user.created_at))}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-admin-role="${T.escapeHtml(user.id)}" data-role="${nextRole}">Make ${nextRole}</button>
              <button type="button" class="btn btn-text btn-sm" data-admin-reset-code="${T.escapeHtml(user.id)}">Reset code</button>
              <button type="button" class="btn btn-text btn-sm" data-admin-revoke-user-devices="${T.escapeHtml(user.id)}">Revoke devices</button>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-admin-delete-user="${T.escapeHtml(user.id)}">Delete</button>
            </td>
          </tr>`;
        })
        .join("")}</tbody>
    </table>`;
  }

  function renderAdminDevices(devices) {
    if (!devices.length) return '<div class="empty-state compact">No trusted devices.</div>';
    return `<table class="data-table compact-table">
      <thead><tr><th>Device</th><th>User</th><th>Last seen</th><th>IP</th><th class="col-actions">Actions</th></tr></thead>
      <tbody>${devices
        .map(
          (device) => `<tr>
            <td><strong>${T.escapeHtml(device.device_name || "Unnamed device")}</strong><div class="row-meta mono">${T.escapeHtml(device.device_token.slice(0, 12))}</div></td>
            <td>${T.escapeHtml(device.user_id)} <span class="tag">${T.escapeHtml(device.role)}</span></td>
            <td>${T.escapeHtml(T.formatDate(device.last_seen_at))}</td>
            <td class="mono">${T.escapeHtml(device.last_ip || "—")}</td>
            <td class="col-actions"><button type="button" class="btn btn-text btn-sm btn-danger" data-admin-revoke-device="${T.escapeHtml(device.device_token)}">Revoke</button></td>
          </tr>`
        )
        .join("")}</tbody>
    </table>`;
  }

  function renderAdminFiles(files) {
    if (!files.length) return '<div class="empty-state compact">No files found.</div>';
    return `<table class="data-table compact-table">
      <thead><tr><th>Name</th><th>Size</th><th>Folder</th><th>Visibility</th><th class="col-actions">Actions</th></tr></thead>
      <tbody>${files
        .map((file) => {
          const visibility = file.visibility === "public" ? "private" : "public";
          return `<tr>
            <td><strong>${T.escapeHtml(file.name)}</strong><div class="row-meta mono">${T.escapeHtml(file.id)}</div></td>
            <td class="mono">${T.escapeHtml(T.formatBytes(file.size_bytes))}</td>
            <td>${T.escapeHtml(file.folder_path || "Bucket root")}</td>
            <td>${stateBadge(file.visibility)}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-preview-file="${T.escapeHtml(file.id)}">Preview</button>
              <button type="button" class="btn btn-text btn-sm" data-vis="${T.escapeHtml(file.id)}" data-v="${visibility}">${visibility === "public" ? "Public" : "Private"}</button>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-admin-delete-file="${T.escapeHtml(file.id)}">Delete</button>
            </td>
          </tr>`;
        })
        .join("")}</tbody>
    </table>`;
  }

  T.loadAdminFiles = async function loadAdminFiles() {
    const filesEl = T.$("#admin-files");
    filesEl.innerHTML = "Loading files…";
    try {
      const data = await T.api("/admin/files?limit=50");
      filesEl.innerHTML = renderAdminFiles(data.files || []);
    } catch (error) {
      filesEl.innerHTML = `<div class="empty-state error">${T.escapeHtml(error.message)}</div>`;
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
        T.api("/admin/users"),
        T.api("/admin/devices"),
      ]);
      overview.innerHTML = `<dl class="admin-dl">
        <dt>Files</dt><dd>${ov.file_count}</dd>
        <dt>Notes</dt><dd>${ov.note_count}</dd>
        <dt>Tags</dt><dd>${ov.tag_count}</dd>
        <dt>Pinned</dt><dd>${ov.pinned_count}</dd>
        <dt>Corrupt</dt><dd>${ov.corrupt_file_count}</dd>
        <dt>Storage</dt><dd>${T.escapeHtml(T.formatBytes(ov.storage_bytes_used))}</dd>
        <dt>Version</dt><dd>${T.escapeHtml(ov.version || "—")}</dd>
      </dl>`;
      system.innerHTML = `<dl class="admin-dl">
        <dt>Host</dt><dd>${T.escapeHtml(sys.hostname)}</dd>
        <dt>OS</dt><dd>${T.escapeHtml(sys.os)} / ${T.escapeHtml(sys.arch)}</dd>
        <dt>Load 1m</dt><dd>${Number(sys.load_average_1m || 0).toFixed(2)}</dd>
        <dt>Memory</dt><dd>${T.escapeHtml(T.formatBytes(sys.available_memory_bytes))} free / ${T.escapeHtml(T.formatBytes(sys.total_memory_bytes))}</dd>
        <dt>Data disk</dt><dd>${T.escapeHtml(T.formatBytes(sys.data_dir_free_bytes))} free / ${T.escapeHtml(T.formatBytes(sys.data_dir_total_bytes))}</dd>
      </dl>`;
      usersEl.innerHTML = renderAdminUsers(users.users || []);
      devicesEl.innerHTML = renderAdminDevices(devices.devices || []);
      T.loadAdminFiles();
    } catch (error) {
      overview.innerHTML = `<div class="empty-state error">${T.escapeHtml(error.message)}</div>`;
      system.innerHTML = "";
      usersEl.innerHTML = "";
      devicesEl.innerHTML = "";
      T.$("#admin-files").innerHTML = "";
    }
  };

  T.createAdminUser = async function createAdminUser() {
    const name = T.$("#admin-user-name").value.trim();
    const code = T.$("#admin-user-code").value;
    const role = T.$("#admin-user-role").value;
    if (!name || code.length < 4) {
      T.showBanner("Name and a 4+ character access code are required", "error");
      return;
    }
    try {
      await T.api("/admin/users", {
        method: "POST",
        body: JSON.stringify({ name, code, role }),
      });
      T.$("#admin-create-user-form").reset();
      T.showBanner("User created", "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminSetUserRole = async function adminSetUserRole(id, role) {
    if (!confirm(`Change this user role to ${role}?`)) return;
    try {
      await T.api("/admin/users/" + encodeURIComponent(id) + "/role", {
        method: "PUT",
        body: JSON.stringify({ role }),
      });
      T.showBanner("Role updated", "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminResetCode = async function adminResetCode(id) {
    const code = prompt("New access code (minimum 4 characters)");
    if (!code) return;
    try {
      await T.api("/admin/users/" + encodeURIComponent(id) + "/reset-code", {
        method: "POST",
        body: JSON.stringify({ code }),
      });
      T.showBanner("Access code reset", "success");
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminDeleteUser = async function adminDeleteUser(id) {
    if (!confirm("Delete this user? The last admin cannot be deleted.")) return;
    try {
      await T.api("/admin/users/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("User deleted", "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminRevokeDevice = async function adminRevokeDevice(token) {
    if (!confirm("Revoke this trusted device?")) return;
    try {
      await T.api("/admin/devices/" + encodeURIComponent(token), { method: "DELETE" });
      T.showBanner("Device revoked", "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminRevokeUserDevices = async function adminRevokeUserDevices(id) {
    if (!confirm("Revoke all trusted devices for this user?")) return;
    try {
      const result = await T.api("/admin/users/" + encodeURIComponent(id) + "/devices", {
        method: "DELETE",
      });
      T.showBanner(`Revoked ${result.removed || 0} device(s)`, "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminDeleteFile = async function adminDeleteFile(id) {
    if (!confirm("Delete this file as admin?")) return;
    try {
      await T.api("/admin/files/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("File deleted", "success");
      T.loadAdminFiles();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminCleanup = async function adminCleanup(kind) {
    const path = kind === "temp" ? "/admin/cleanup/temp" : "/admin/cleanup/sessions";
    try {
      const result = await T.api(path, { method: "POST" });
      T.showBanner(
        kind === "temp"
          ? `Removed ${result.removed ?? 0} temp file(s)`
          : result.message || "Cleanup requested",
        "success"
      );
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };
})(window.Tssp);
