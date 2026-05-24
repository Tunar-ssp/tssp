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
    overlay.innerHTML = `<div class="modal-card" role="dialog" aria-labelledby="share-dialog-title">
      <header class="modal-header">
        <h2 id="share-dialog-title">Public link</h2>
        <button type="button" class="btn btn-text modal-close" aria-label="Close">×</button>
      </header>
      <div class="modal-body">
        <p class="mono share-url">${T.escapeHtml(payload.public_url)}</p>
        <pre class="share-qr" aria-label="QR code">${T.escapeHtml(payload.qr_terminal)}</pre>
      </div>
      <footer class="modal-footer">
        <button type="button" class="btn btn-secondary" data-copy-share>Copy link</button>
        <button type="button" class="btn btn-primary modal-close">Done</button>
      </footer>
    </div>`;
    const close = () => overlay.remove();
    overlay.querySelectorAll(".modal-close").forEach((btn) => {
      btn.addEventListener("click", close);
    });
    overlay.addEventListener("click", (ev) => {
      if (ev.target === overlay) close();
    });
    overlay.querySelector("[data-copy-share]")?.addEventListener("click", () => {
      T.copyText(payload.public_url)
        .then(() => T.showBanner("Link copied", "success"))
        .catch((error) => T.showBanner(error.message, "error"));
    });
    document.body.appendChild(overlay);
  };
})(window.Tssp);
