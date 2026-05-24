window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.loadPublic = async function loadPublic() {
    const body = T.$("#public-grid") || T.$("#public-body");
    if (!body) return;
    body.innerHTML = '<div id="public-body" class="empty-state compact">Loading public links...</div>';
	    try {
	      const data = await T.api("/public/files");
	      const files = data.files || [];
	      const total = files.reduce((sum, file) => sum + Number(file.size_bytes || 0), 0);
	      const countEl = T.$("#sharing-count");
	      const sizeEl = T.$("#sharing-size");
	      if (countEl) countEl.textContent = String(files.length);
	      if (sizeEl) sizeEl.textContent = T.formatBytes(total);
	      if (!files.length) {
        body.innerHTML = `<div class="notes-empty-hero">
          <div class="notes-empty-icon">🔗</div>
          <div class="notes-empty-title">No public links yet</div>
          <div class="notes-empty-sub">Open a file in Cloud Drive and set visibility to Public to create a shareable link.</div>
          <button type="button" class="btn btn-primary" data-view-jump="objects">Open Cloud Drive</button>
        </div>`;
        return;
      }
      body.innerHTML = files
        .map((file) => {
          const id = T.escapeHtml(file.id);
          const link = T.publicLink(file);
          return `<article class="public-card">
            <div class="public-card-head">
              <span class="file-kind-icon ${T.escapeHtml(T.fileKindClass(file))}" aria-hidden="true">${T.escapeHtml(T.fileKindIcon(file))}</span>
              <div>
                <strong>${T.escapeHtml(file.name)}</strong>
                <div class="row-meta">${T.escapeHtml(file.folder_path || "Bucket root")} · ${T.escapeHtml(T.formatBytes(file.size_bytes))}</div>
              </div>
            </div>
            <button type="button" class="public-link mono" data-copy-link="${T.escapeHtml(link)}">${T.escapeHtml(link.replace(window.location.origin, ""))}</button>
            <div class="public-card-actions">
              <button type="button" class="btn btn-secondary btn-sm" data-preview-file="${id}">Preview</button>
              <button type="button" class="btn btn-secondary btn-sm" data-share-file="${id}">QR</button>
              <a class="btn btn-secondary btn-sm" href="${T.fileDownloadUrl(file.id)}" download>Download</a>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-vis="${id}" data-v="private">Make private</button>
            </div>
          </article>`;
        })
        .join("");
	    } catch (error) {
	      const countEl = T.$("#sharing-count");
	      const sizeEl = T.$("#sharing-size");
	      if (countEl) countEl.textContent = "—";
	      if (sizeEl) sizeEl.textContent = "—";
	      body.innerHTML = `<div id="public-body" class="empty-state error">${T.escapeHtml(error.message)}</div>`;
	    }
  };

})(window.Tssp);
