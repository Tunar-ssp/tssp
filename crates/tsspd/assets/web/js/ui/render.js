window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.tagsHtml = function tagsHtml(tags) {
    return (tags || [])
      .map((tag) => `<span class="tag">${T.escapeHtml(tag)}</span>`)
      .join("");
  };

  T.stateBadge = function stateBadge(value) {
    const isPublic = value === "public";
    return `<span class="state-badge ${isPublic ? "public" : "private"}">${isPublic ? "Public" : "Private"}</span>`;
  };

  T.publicLink = function publicLink(file) {
    return file.public_token ? `${window.location.origin}/p/${file.public_token}` : "";
  };

  T.fileKindClass = function fileKindClass(file) {
    const mime = file?.mime_type || "";
    if (mime.startsWith("image/")) return "file-kind-image";
    if (mime.startsWith("video/")) return "file-kind-video";
    if (mime.startsWith("audio/")) return "file-kind-audio";
    if (mime.includes("pdf")) return "file-kind-pdf";
    if (mime.startsWith("text/")) return "file-kind-text";
    if (mime.includes("json") || mime.includes("javascript") || mime.includes("typescript")) {
      return "file-kind-code";
    }
    return "file-kind-file";
  };

  T.fileKindIcon = function fileKindIcon(file) {
    const kind = T.fileKindClass(file);
    if (kind === "file-kind-image") return "IMG";
    if (kind === "file-kind-video") return "VID";
    if (kind === "file-kind-audio") return "AUD";
    if (kind === "file-kind-pdf") return "PDF";
    if (kind === "file-kind-text") return "TXT";
    if (kind === "file-kind-code") return "DEV";
    return "OBJ";
  };

  T.tableMessage = function tableMessage(columns, message) {
    return `<tr><td colspan="${columns}" class="table-empty">${T.escapeHtml(message)}</td></tr>`;
  };
})(window.Tssp);
