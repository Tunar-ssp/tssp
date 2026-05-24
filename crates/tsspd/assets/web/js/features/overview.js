window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.loadOverview = async function loadOverview() {
    const grid = T.$("#metric-grid");
    const details = T.$("#overview-details");
    if (!grid) return;

    grid.innerHTML = `<div class="metric-card skeleton-card" style="grid-column:1/-1;height:64px"></div>`;
    if (details) details.innerHTML = "";

    try {
      const [statusRes, publicRes, recentRes] = await Promise.allSettled([
        T.api("/status"),
        T.api("/public/files"),
        T.api("/files?limit=10"),
      ]);

      if (statusRes.status !== "fulfilled") throw statusRes.reason;
      const s = statusRes.value;
      const publicFiles =
        publicRes.status === "fulfilled" ? publicRes.value.files || [] : [];
      const recentFiles =
        recentRes.status === "fulfilled" ? recentRes.value.files || [] : [];

      const isHealthy = s.status === "ok";
      const corruptWarn = s.corrupt_file_count > 0;

      /* ── Metric cards ─────────────────────────────────────────────── */
      grid.innerHTML = [
        metricCard("Files", s.file_count ?? "—", "objects stored", "📁", "blue"),
        metricCard("Notes", s.note_count ?? "—", "markdown pages", "📝", "yellow"),
        metricCard("Storage", T.formatBytes(s.storage_bytes_used ?? 0), "disk used", "💾", "cyan"),
        metricCard("Public", publicFiles.length, "shared links", "🔗", "green"),
        metricCard("Pinned", s.pinned_count ?? "—", "pinned items", "📌", "orange"),
        metricCard("Tags", s.tag_count ?? "—", "unique tags", "🏷️", "violet"),
        metricCard("Recent (24h)", s.recent_upload_count_24h ?? "—", "uploads today", "⬆️", "blue"),
        metricCard("Uptime", T.formatUptime(s.uptime_seconds ?? 0), "since restart", "⏱", isHealthy ? "green" : "red"),
      ].join("");

      /* ── Details section ──────────────────────────────────────────── */
      if (!details) return;

      const storageTotal = s.storage_total_bytes ?? 0;
      const storageUsed = s.storage_bytes_used ?? 0;
      const storagePct = storageTotal > 0 ? Math.round((storageUsed / storageTotal) * 100) : 0;
      const storageBar = storageTotal > 0
        ? `<div class="health-bar-track"><div class="health-bar-fill" style="width:${Math.min(storagePct, 100)}%;background:${storagePct > 85 ? "var(--red)" : storagePct > 65 ? "var(--yellow)" : "var(--blue)"}"></div></div><span class="health-bar-label">${T.escapeHtml(T.formatBytes(storageUsed))} / ${T.escapeHtml(T.formatBytes(storageTotal))} (${storagePct}%)</span>`
        : `<span class="health-bar-label">${T.escapeHtml(T.formatBytes(storageUsed))} used</span>`;

      const cpuLoad = s.cpu_load_1m ?? 0;
      const memUsed = s.memory_used_bytes ?? 0;
      const memTotal = s.memory_total_bytes ?? 0;
      const memPct = memTotal > 0 ? Math.round((memUsed / memTotal) * 100) : 0;

      const systemHtml = `
        <div class="overview-sys-grid">
          <div class="overview-sys-item">
            <div class="overview-sys-label">CPU load (1m)</div>
            <div class="health-bar-track"><div class="health-bar-fill" style="width:${Math.min(cpuLoad * 10, 100)}%;background:${cpuLoad > 7 ? "var(--red)" : cpuLoad > 4 ? "var(--yellow)" : "var(--green)"}"></div></div>
            <span class="health-bar-label">${cpuLoad.toFixed(2)}</span>
          </div>
          <div class="overview-sys-item">
            <div class="overview-sys-label">Memory</div>
            <div class="health-bar-track"><div class="health-bar-fill" style="width:${memPct}%;background:${memPct > 85 ? "var(--red)" : memPct > 65 ? "var(--yellow)" : "var(--blue)"}"></div></div>
            <span class="health-bar-label">${T.escapeHtml(T.formatBytes(memUsed))} / ${T.escapeHtml(T.formatBytes(memTotal))}</span>
          </div>
          <div class="overview-sys-item">
            <div class="overview-sys-label">Disk storage</div>
            ${storageBar}
          </div>
        </div>`;

      const healthBadge = isHealthy
        ? `<span class="overview-health-badge ok">● Healthy</span>`
        : `<span class="overview-health-badge warn">● ${T.escapeHtml(s.status || "Unknown")}</span>`;
      const corruptBadge = corruptWarn
        ? `<span class="overview-health-badge error">⚠ ${s.corrupt_file_count} corrupt files</span>`
        : "";
      const hostLine = s.host
        ? `<span class="overview-host">${T.escapeHtml(s.host)}${s.os ? ` · ${T.escapeHtml(s.os)}` : ""}${s.version ? ` · v${T.escapeHtml(s.version)}` : ""}</span>`
        : "";

      const recentHtml = recentFiles.length
        ? `<div class="overview-section">
            <div class="overview-section-head">
              <h3>Recent uploads</h3>
              <button type="button" class="btn btn-text btn-sm" onclick="window.Tssp.setView('objects')">View all →</button>
            </div>
            <div class="recent-uploads">${recentFiles.slice(0, 8).map((f) =>
              `<div class="recent-upload-row">
                <span class="file-kind-icon file-kind-${T.escapeHtml(T.fileKind(f))}">${T.escapeHtml(T.fileKind(f).slice(0, 3).toUpperCase())}</span>
                <span class="recent-upload-name">${T.escapeHtml(f.name || f.id)}</span>
                <span class="recent-upload-meta">${T.escapeHtml(T.formatBytes(f.size_bytes))} · ${T.escapeHtml(T.formatDate(f.uploaded_at))}</span>
                <button class="btn btn-text btn-sm" data-preview-file="${T.escapeHtml(f.id)}">Preview</button>
              </div>`
            ).join("")}</div>
          </div>`
        : "";

      const quickActions = `
        <div class="overview-section">
          <h3>Quick actions</h3>
          <div class="overview-quick-actions">
            <button class="overview-qa-btn" onclick="window.Tssp.setView('objects')">
              <span class="overview-qa-icon" style="background:rgba(37,99,235,0.15);color:var(--blue)">📁</span>
              <span>Browse files</span>
            </button>
            <button class="overview-qa-btn" onclick="window.Tssp.openNoteDialog && window.Tssp.openNoteDialog(null);window.Tssp.setView('notes')">
              <span class="overview-qa-icon" style="background:rgba(251,191,36,0.15);color:var(--yellow)">📝</span>
              <span>New note</span>
            </button>
            <button class="overview-qa-btn" onclick="window.Tssp.setView('workspaces')">
              <span class="overview-qa-icon" style="background:rgba(129,140,248,0.15);color:var(--indigo)">⌨</span>
              <span>Workspaces</span>
            </button>
            <button class="overview-qa-btn" onclick="window.Tssp.setView('admin')">
              <span class="overview-qa-icon" style="background:rgba(248,113,113,0.15);color:var(--red)">⚙</span>
              <span>Admin console</span>
            </button>
          </div>
        </div>`;

      details.innerHTML = `
        <div class="overview-details-grid">
          <div>
            <div class="overview-section">
              <div class="overview-section-head">
                <h3>System health</h3>
                <div style="display:flex;gap:8px;flex-wrap:wrap">${healthBadge}${corruptBadge}${hostLine}</div>
              </div>
              ${systemHtml}
            </div>
            ${recentHtml}
          </div>
          <div>
            ${quickActions}
          </div>
        </div>`;
    } catch (error) {
      grid.innerHTML = `<div class="metric-card" style="grid-column:1/-1"><div class="label">Error loading overview</div><div class="value" style="font-size:1rem;color:var(--red)">${T.escapeHtml(error.message)}</div></div>`;
    }
  };

  function metricCard(label, value, sub, icon, color) {
    const colors = {
      blue:   "rgba(37,99,235,0.12)",  violet: "rgba(129,140,248,0.12)",
      cyan:   "rgba(34,211,238,0.12)", green:  "rgba(74,222,128,0.12)",
      yellow: "rgba(251,191,36,0.12)", orange: "rgba(251,146,60,0.12)",
      red:    "rgba(248,113,113,0.12)",
    };
    const bg = colors[color] || colors.blue;
    return `<div class="metric-card">
      <div class="metric-card-head">
        <span class="metric-icon" style="background:${bg}">${icon}</span>
        <span class="metric-label">${T.escapeHtml(label)}</span>
      </div>
      <div class="metric-value">${T.escapeHtml(String(value ?? "—"))}</div>
      ${sub ? `<div class="metric-sub">${T.escapeHtml(sub)}</div>` : ""}
    </div>`;
  }

})(window.Tssp);
