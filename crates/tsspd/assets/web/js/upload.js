window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.uploadFiles = async function uploadFiles(fileList) {
    if (!fileList || !fileList.length) return;
    const progress = T.$("#upload-progress");
    const folder = (T.$("#upload-folder")?.value || "").trim();
    const pinned = T.$("#upload-pin")?.checked || false;
    const tagInput = T.$("#upload-tags")?.value || "";
    const tags = tagInput
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);

    T.showBanner(`Uploading ${fileList.length} file(s)…`, "info");
    if (progress) {
      progress.classList.remove("hidden");
      progress.textContent = `Preparing ${fileList.length} file(s)…`;
    }
    let ok = 0;
    const files = Array.from(fileList);
    for (const file of files) {
      if (progress) {
        progress.textContent = `Uploading ${ok + 1} of ${files.length}: ${file.name}`;
      }
      const fd = new FormData();
      fd.append("file", file);
      if (folder) fd.append("folder", folder);
      if (pinned) fd.append("pin", "1");
      for (const tag of tags) fd.append("tag", tag);
      try {
        await T.api("/files", { method: "POST", body: fd });
        ok++;
      } catch (e) {
        T.showBanner(`Upload failed: ${e.message}`, "error");
        if (progress) {
          progress.textContent = `Upload failed: ${e.message}`;
        }
        return;
      }
    }
    T.showBanner(`Uploaded ${ok} file(s)`, "success");
    if (progress) {
      progress.textContent = `Uploaded ${ok} file(s)`;
      setTimeout(() => progress.classList.add("hidden"), 3500);
    }
    setTimeout(() => T.showBanner(""), 3000);
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
