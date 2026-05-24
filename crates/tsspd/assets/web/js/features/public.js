window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

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

})(window.Tssp);
