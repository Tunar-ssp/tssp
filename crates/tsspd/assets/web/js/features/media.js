window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.loadImages = async function loadImages() {
    const grid = T.$("#image-grid");
    if (!grid) return;
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
                <span>${T.escapeHtml(file.folder_path || "Bucket root")}</span>
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

  T.loadVideos = async function loadVideos() {
    const grid = T.$("#video-grid");
    if (!grid) return;
    grid.innerHTML = '<div class="empty-state compact">Loading videos…</div>';
    try {
      const data = await T.api("/files?type=video/&limit=200");
      const files = data.files || [];
      if (!files.length) {
        grid.innerHTML =
          '<div class="empty-state"><strong>No videos yet</strong><p>Upload video files and they will appear here.</p></div>';
        return;
      }
      grid.innerHTML = files
        .map((file) => {
          const id = T.escapeHtml(file.id);
          const nextVis = file.visibility === "public" ? "private" : "public";
          const src = T.escapeHtml(T.fileInlineUrl(file.id));
          return `<article class="image-card video-card">
            <button type="button" class="media-open" data-preview-file="${id}" aria-label="Preview ${T.escapeHtml(file.name)}">
              <video class="video-thumb" src="${src}" preload="metadata" muted playsinline tabindex="-1"></video>
              <div class="image-card-overlay video-play-icon">
                <span class="image-card-overlay-icon">▶</span>
              </div>
            </button>
            <div class="media-card-footer">
              <strong class="media-card-name" title="${T.escapeHtml(file.name)}">${T.escapeHtml(file.name)}</strong>
              <div class="media-card-meta">
                <span>${T.escapeHtml(file.folder_path || "Bucket root")}</span>
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
    if (!body) return;
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
    const grid = T.$("#documents-grid");
    if (body) body.innerHTML = T.tableMessage(6, "Loading documents...");
    if (grid) grid.innerHTML = '<div class="empty-state compact">Loading documents...</div>';
    try {
      // Fetch application/ and text/ separately; backend MIME filter is prefix-based
      const [appData, textData] = await Promise.all([
        T.api("/files?limit=200&type=application/"),
        T.api("/files?limit=200&type=text/"),
      ]);
      const seen = new Set();
      const files = [...(appData.files || []), ...(textData.files || [])].filter((file) => {
        if (seen.has(file.id)) return false;
        seen.add(file.id);
        const mime = file.mime_type || "";
        return !mime.startsWith("image/") && !mime.startsWith("video/") && !mime.startsWith("audio/");
      });
      if (!files.length) {
        if (body) body.innerHTML = T.tableMessage(6, "No documents yet.");
        if (grid) {
          grid.innerHTML =
            '<div class="empty-state"><strong>No documents yet</strong><span>Upload PDFs, text files, or source documents from Cloud Drive.</span><div class="empty-actions"><button type="button" class="btn btn-secondary" data-view-jump="objects">Upload document</button></div></div>';
        }
        return;
      }
      if (grid) {
        grid.innerHTML = files
          .map((file) => {
            const id = T.escapeHtml(file.id);
            return `<article class="document-card">
              <span class="file-kind-icon ${T.escapeHtml(T.fileKindClass(file))}" aria-hidden="true">${T.escapeHtml(T.fileKindIcon(file))}</span>
              <div class="document-card-main">
                <strong>${T.escapeHtml(file.name)}</strong>
                <div class="row-meta">${T.escapeHtml(file.folder_path || "Bucket root")} · ${T.escapeHtml(file.mime_type || "unknown")} · ${T.escapeHtml(T.formatBytes(file.size_bytes))}</div>
                <div class="document-card-tags">${T.tagsHtml(file.tags)}${T.stateBadge(file.visibility)}</div>
              </div>
              <div class="document-card-actions">
                <button type="button" class="btn btn-secondary btn-sm" data-preview-file="${id}">Preview</button>
                <a class="btn btn-secondary btn-sm" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
              </div>
            </article>`;
          })
          .join("");
      }
      if (body) body.innerHTML = files
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
      if (body) body.innerHTML = T.tableMessage(6, error.message);
      if (grid) grid.innerHTML = `<div class="empty-state error">${T.escapeHtml(error.message)}</div>`;
    }
  };

})(window.Tssp);
