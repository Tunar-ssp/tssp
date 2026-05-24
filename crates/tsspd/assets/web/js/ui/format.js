window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.formatBytes = function formatBytes(n) {
    if (n == null) return "—";
    if (Number(n) === 0) return "0 B";
    const u = ["B", "KB", "MB", "GB", "TB"];
    let i = 0;
    let v = Number(n);
    while (v >= 1024 && i < u.length - 1) {
      v /= 1024;
      i++;
    }
    return `${v.toFixed(i > 0 ? 1 : 0)} ${u[i]}`;
  };

  T.formatDate = function formatDate(value) {
    if (value == null || value === "") return "—";
    try {
      const ms =
        typeof value === "number" ? value * 1000 : Date.parse(String(value));
      if (Number.isNaN(ms)) return String(value);
      return new Date(ms).toLocaleString(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
      });
    } catch {
      return String(value);
    }
  };

  T.formatUptime = function formatUptime(sec) {
    const h = Math.floor(sec / 3600);
    const m = Math.floor((sec % 3600) / 60);
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  };

  T.fileKind = function fileKind(file) {
    const mime = file?.mime_type || "";
    if (mime.startsWith("image/")) return "Image";
    if (mime.startsWith("video/")) return "Video";
    if (mime.startsWith("audio/")) return "Audio";
    if (mime.startsWith("text/")) return "Text";
    if (mime.includes("pdf")) return "PDF";
    if (mime.startsWith("application/")) return "Document";
    return "Object";
  };

  T.tagsFromInput = function tagsFromInput(value) {
    return String(value || "")
      .split(",")
      .map((tag) => tag.trim())
      .filter(Boolean);
  };
})(window.Tssp);
