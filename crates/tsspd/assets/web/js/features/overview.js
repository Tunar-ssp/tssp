window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

T.loadOverview = async function loadOverview() {
    const grid = T.$("#metric-grid");
    const details = T.$("#overview-details");
    grid.innerHTML = '<div class="metric-card"><span class="label">Loading…</span></div>';
    if (details) details.innerHTML = "";
    try {
      const [status, publicFiles, recentFiles] = await Promise.allSettled([
        T.api("/status"),
        T.api("/public/files"),
        T.api("/files?limit=8"),
      ]);
      if (status.status !== "fulfilled") throw status.reason;
      const s = status.value;
      const publicCount =
        publicFiles.status === "fulfilled" ? (publicFiles.value.files || []).length : 0;
      const recentUploads =
        recentFiles.status === "fulfilled" ? (recentFiles.value.files || []) : [];

      const health = s.status === "ok" ? "Healthy" : T.escapeHtml(s.status || "—");
      const healthClass = s.status === "ok" ? "metric-health-ok" : "metric-health-warn";

      grid.innerHTML = [
        metricCard("Files", s.file_count, "total objects stored"),
        metricCard("Notes", s.note_count, "markdown notes"),
        metricCard("Storage", T.formatBytes(s.storage_bytes_used), "used"),
        metricCard("Public", publicCount, "publicly shared"),
        metricCard("Pinned", s.pinned_count, "files + notes"),
        metricCard("Tags", s.tag_count, "unique tags"),
        metricCard("Uploaded (24h)", s.recent_upload_count_24h ?? "—", "recent"),
        metricCard("Uptime", T.formatUptime(s.uptime_seconds), "since last restart"),
        `<div class="metric-card metric-health ${healthClass}"><div class="label">Health</div><div class="value">${health}</div></div>`,
      ].join("");

      if (details && recentUploads.length) {
        details.innerHTML = `<div class="overview-section"><h3>Recent uploads</h3><div class="recent-uploads">${
          recentUploads.slice(0, 8).map((f) =>
            `<div class="recent-upload-row">
              <span class="recent-upload-name">${T.escapeHtml(f.name || f.id)}</span>
              <span class="recent-upload-meta">${T.escapeHtml(T.formatBytes(f.size_bytes))} · ${T.escapeHtml(T.formatDate(f.uploaded_at))}</span>
            </div>`
          ).join("")
        }</div></div>`;
      }
    } catch (error) {
      grid.innerHTML = `<div class="metric-card"><span class="label">Error</span><div class="value">${T.escapeHtml(error.message)}</div></div>`;
    }
  };

  function metricCard(label, value, sub) {
    return `<div class="metric-card">
      <div class="label">${T.escapeHtml(label)}</div>
      <div class="value">${T.escapeHtml(String(value ?? "—"))}</div>
      ${sub ? `<div class="metric-sub">${T.escapeHtml(sub)}</div>` : ""}
    </div>`;
  }

})(window.Tssp);
