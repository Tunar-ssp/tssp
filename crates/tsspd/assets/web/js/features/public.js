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
      if (!files.length) {
        body.innerHTML =
          '<div id="public-body" class="empty-state"><strong>No public links</strong><span>Make a file public from Cloud Drive to share it here.</span><div class="empty-actions"><button type="button" class="btn btn-secondary" data-view-jump="objects">Open Cloud Drive</button></div></div>';
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
      body.innerHTML = `<div id="public-body" class="empty-state error">${T.escapeHtml(error.message)}</div>`;
    }
  };

})(window.Tssp);
