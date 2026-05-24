window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.confirmAction = function confirmAction(message) {
    return window.confirm(message);
  };

  T.promptText = function promptText(message, defaultValue = "") {
    return window.prompt(message, defaultValue);
  };

  T.showShareDialog = function showShareDialog(payload) {
    const overlay = document.createElement("div");
    overlay.className = "modal-overlay";
    const path = payload.public_url.replace(window.location.origin, "");
    overlay.innerHTML = `<div class="modal-card share-modal" role="dialog" aria-labelledby="share-dialog-title">
      <header class="modal-header">
        <h2 id="share-dialog-title" style="margin:0;font-size:16px">Share link</h2>
        <button type="button" class="icon-btn modal-close" aria-label="Close">×</button>
      </header>
      <div class="modal-body">
        <div class="share-link-row">
          <input type="text" class="share-url-input" readonly value="${T.escapeHtml(payload.public_url)}" aria-label="Public URL">
          <button type="button" class="btn btn-secondary btn-sm" data-copy-share title="Copy link">Copy</button>
        </div>
        <div class="share-path-hint">${T.escapeHtml(path)}</div>
        ${payload.qr_terminal ? `<div class="share-qr-section">
          <div class="share-qr-label">QR code (scan to open)</div>
          <pre class="share-qr" aria-label="QR code">${T.escapeHtml(payload.qr_terminal)}</pre>
        </div>` : ""}
        <div class="share-actions-row">
          <a class="btn btn-secondary btn-sm" href="${T.escapeHtml(payload.public_url)}" target="_blank" rel="noopener">Open link ↗</a>
          <a class="btn btn-secondary btn-sm" href="${T.escapeHtml(payload.public_url)}" download>Download</a>
        </div>
      </div>
    </div>`;

    const close = () => overlay.remove();
    overlay.querySelectorAll(".modal-close").forEach((btn) => btn.addEventListener("click", close));
    overlay.addEventListener("click", (ev) => { if (ev.target === overlay) close(); });
    overlay.querySelector("[data-copy-share]")?.addEventListener("click", () => {
      const input = overlay.querySelector(".share-url-input");
      if (input) { input.select(); input.setSelectionRange(0, 99999); }
      T.copyText(payload.public_url)
        .then(() => T.showBanner("Link copied to clipboard", "success"))
        .catch((error) => T.showBanner(error.message, "error"));
    });
    document.body.appendChild(overlay);
    overlay.querySelector(".share-url-input")?.select();
  };
})(window.Tssp);
