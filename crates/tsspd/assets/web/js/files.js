window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

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
    const link = T.publicLink(file);
    const nextVisibility = file.visibility === "public" ? "private" : "public";
    return `<tr data-file-row="${id}">
      <td class="col-select"><input type="checkbox" data-file-select="${id}" ${checked} aria-label="Select ${T.escapeHtml(file.name || file.id)}"></td>
      <td>
        <div class="file-name">${pin}<strong>${T.escapeHtml(file.name || file.id)}</strong></div>
        <div class="row-meta">${folder}${T.tagsHtml(file.tags)}${T.stateBadge(file.visibility)}</div>
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
    const selected = T.currentFiles.filter((file) => T.selectedFileIds.has(file.id));
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
    const link = T.publicLink(file);
    details.innerHTML = `<h3>${T.escapeHtml(file.name)}</h3>
      <dl class="admin-dl">
        <dt>ID</dt><dd class="mono">${T.escapeHtml(file.id)}</dd>
        <dt>Type</dt><dd>${T.escapeHtml(file.mime_type || "—")}</dd>
        <dt>Size</dt><dd>${T.escapeHtml(T.formatBytes(file.size_bytes))}</dd>
        <dt>Folder</dt><dd>${T.escapeHtml(file.folder_path || "Bucket root")}</dd>
        <dt>Uploaded</dt><dd>${T.escapeHtml(T.formatDate(file.uploaded_at))}</dd>
        <dt>Visibility</dt><dd>${T.stateBadge(file.visibility)}</dd>
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

  T.loadFolderTree = async function loadFolderTree() {
    const tree = T.$("#folder-tree");
    if (!tree) return;

    function renderTree(items) {
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
    }

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
      renderTree(items);
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
        renderTree(items);
      } catch {
        tree.innerHTML =
          '<button type="button" data-folder="" class="active"><span>Bucket root</span><span class="mono">—</span></button>';
      }
    }
  };

  T.loadFiles = async function loadFiles() {
    const body = T.$("#files-body");
    body.innerHTML = T.tableMessage(6, "Loading objects…");
    const params = new URLSearchParams({ limit: "200" });
    if (T.$("#pinned-only")?.checked) params.set("pinned", "true");
    if (T.currentFolder) params.set("folder", T.currentFolder);
    try {
      const data = await T.api("/files?" + params.toString());
      T.currentFiles = data.files || [];
      T.selectedFileIds.clear();
      updateObjectSummary(T.currentFiles);
      if (!T.currentFiles.length) {
        body.innerHTML = T.tableMessage(
          6,
          "No objects in this folder. Drop files above or click Upload."
        );
        T.updateFileSelection();
        return;
      }
      body.innerHTML = T.currentFiles.map(fileRow).join("");
      T.updateFileSelection();
    } catch (error) {
      body.innerHTML = T.tableMessage(6, error.message);
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
    const meta = T.$("#preview-meta");
    const dlBtn = T.$("#preview-download-btn");
    title.textContent = file.name || file.id;
    if (meta) {
      meta.innerHTML = [
        T.stateBadge(file.visibility),
        `<span class="preview-meta-item">${T.escapeHtml(T.formatBytes(file.size_bytes))}</span>`,
        file.mime_type ? `<span class="preview-meta-item mono">${T.escapeHtml(file.mime_type)}</span>` : "",
        file.folder_path ? `<span class="preview-meta-item">${T.escapeHtml(file.folder_path)}</span>` : "",
      ].join("");
    }
    if (dlBtn) {
      dlBtn.href = T.fileDownloadUrl(file.id);
      dlBtn.removeAttribute("hidden");
    }
    const inline = T.fileInlineUrl(file.id);
    const mime = file.mime_type || "";
    const name = (file.name || "").toLowerCase();

    if (mime.startsWith("image/")) {
      content.innerHTML = `<img class="preview-media" src="${inline}" alt="${T.escapeHtml(file.name)}">`;
    } else if (mime.startsWith("video/")) {
      content.innerHTML = `<video class="preview-media" src="${inline}" controls preload="metadata"></video>`;
    } else if (mime.startsWith("audio/")) {
      content.innerHTML = `<div class="preview-audio-wrap"><audio controls src="${inline}" style="width:100%;margin-top:24px"></audio><p class="muted" style="text-align:center;margin-top:12px">${T.escapeHtml(file.name)}</p></div>`;
    } else if (mime === "application/pdf" || name.endsWith(".pdf")) {
      content.innerHTML = `<iframe class="preview-iframe" src="${inline}" title="${T.escapeHtml(file.name)}"></iframe>`;
    } else if (mime.startsWith("text/") || isTextLike(name, mime)) {
      content.innerHTML = `<div class="preview-text-loading">Loading text…</div>`;
      T.$("#preview-dialog").showModal();
      try {
        const resp = await fetch(inline, { credentials: "same-origin" });
        const text = await resp.text();
        const isMarkdown = name.endsWith(".md") || name.endsWith(".markdown");
        if (isMarkdown) {
          content.innerHTML = `<article class="markdown-preview preview-md">${T.simpleMarkdown(text)}</article>`;
        } else {
          content.innerHTML = `<pre class="preview-code">${T.escapeHtml(text.slice(0, 60000))}</pre>`;
        }
      } catch {
        content.innerHTML = buildFallback(T.fileKind(file), inline, T.fileDownloadUrl(file.id), file.name);
      }
      return;
    } else {
      content.innerHTML = buildFallback(T.fileKind(file), inline, T.fileDownloadUrl(file.id), file.name);
    }
    T.$("#preview-dialog").showModal();
  };

  function isTextLike(name, mime) {
    const textExts = [".rs", ".py", ".js", ".ts", ".jsx", ".tsx", ".html", ".css", ".json",
      ".toml", ".yaml", ".yml", ".xml", ".sh", ".bash", ".txt", ".csv", ".log", ".md",
      ".markdown", ".ini", ".env", ".gitignore", ".dockerfile"];
    return textExts.some((ext) => name.endsWith(ext)) || mime === "application/json";
  }

  function buildFallback(kind, inlineUrl, downloadUrl, name) {
    return `<div class="empty-state compact">
      <strong>${T.escapeHtml(kind)} preview unavailable</strong>
      <p>Open or download the object to view its contents.</p>
      <div class="empty-actions">
        <a class="btn btn-secondary" href="${inlineUrl}" target="_blank" rel="noopener">Open</a>
        <a class="btn btn-primary" href="${downloadUrl}" download>Download</a>
      </div>
    </div>`;
  }
})(window.Tssp);
