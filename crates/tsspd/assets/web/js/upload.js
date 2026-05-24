window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.uploadFiles = async function uploadFiles(fileList) {
    if (!fileList || !fileList.length) return;
    const queue = T.$("#upload-queue");
    const folder = (T.$("#upload-folder")?.value || "").trim();
    const pinned = T.$("#upload-pin")?.checked || false;
    const tagInput = T.$("#upload-tags")?.value || "";
    const tags = tagInput.split(",").map((t) => t.trim()).filter(Boolean);

    const files = Array.from(fileList);

    if (queue) {
      queue.classList.remove("hidden");
      queue.innerHTML = files
        .map((f, i) =>
          `<div class="upload-queue-row" id="uq-${i}">
            <span class="upload-queue-name" title="${T.escapeHtml(f.name)}">${T.escapeHtml(f.name)}</span>
            <span class="upload-queue-size">${T.escapeHtml(T.formatBytes(f.size))}</span>
            <span class="upload-queue-status pending" id="uqs-${i}">Queued</span>
          </div>`
        )
        .join("");
    }

    let ok = 0;
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      const statusEl = T.$(`#uqs-${i}`);
      if (statusEl) { statusEl.textContent = "Uploading…"; statusEl.className = "upload-queue-status uploading"; }
      const fd = new FormData();
      fd.append("file", file);
      if (folder) fd.append("folder", folder);
      if (pinned) fd.append("pin", "1");
      for (const tag of tags) fd.append("tag", tag);
      try {
        await T.api("/files", { method: "POST", body: fd });
        ok++;
        if (statusEl) { statusEl.textContent = "Done"; statusEl.className = "upload-queue-status done"; }
      } catch (e) {
        if (statusEl) { statusEl.textContent = `Failed: ${e.message}`; statusEl.className = "upload-queue-status failed"; }
      }
    }

    const failed = files.length - ok;
    if (failed === 0) {
      T.showBanner(`Uploaded ${ok} file${ok !== 1 ? "s" : ""}`, "success");
    } else {
      T.showBanner(`${ok} uploaded, ${failed} failed`, "error");
    }
    if (queue) setTimeout(() => { queue.classList.add("hidden"); queue.innerHTML = ""; }, 4000);

    T.loadFiles();
    T.loadFolderTree();
    if (T.$("#view-images") && !T.$("#view-images").classList.contains("hidden")) {
      T.loadImages();
    }
  };

  T.toggleFilePin = async function toggleFilePin(id, pinned) {
    try {
      if (pinned) {
        await T.api("/files/" + encodeURIComponent(id) + "/pin", { method: "DELETE" });
      } else {
        await T.api("/files/" + encodeURIComponent(id) + "/pin", { method: "PUT" });
      }
      T.loadFiles();
    } catch (e) {
      T.showBanner(e.message, "error");
    }
  };

  T.bindUpload = function bindUpload() {
    T.$("#upload-btn")?.addEventListener("click", () => T.$("#upload-input").click());
    T.$("#upload-input")?.addEventListener("change", (ev) => {
      T.uploadFiles(ev.target.files);
      ev.target.value = "";
    });

    const drop = T.$("#drop-zone");
    if (!drop) return;
    ["dragenter", "dragover"].forEach((ev) => {
      drop.addEventListener(ev, (e) => {
        e.preventDefault();
        drop.classList.add("dragover");
      });
    });
    ["dragleave", "drop"].forEach((ev) => {
      drop.addEventListener(ev, (e) => {
        e.preventDefault();
        drop.classList.remove("dragover");
      });
    });
    drop.addEventListener("drop", (e) => T.uploadFiles(e.dataTransfer.files));
    drop.addEventListener("click", () => T.$("#upload-input")?.click());
  };
})(window.Tssp);
