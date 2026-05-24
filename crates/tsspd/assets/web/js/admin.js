window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  function sysBar(label, detail, pct) {
    const color = pct > 85 ? "var(--danger)" : pct > 65 ? "var(--warning)" : "var(--brand)";
    return `<div class="sys-bar-row">
      <div class="sys-bar-head">
        <span class="sys-bar-label">${T.escapeHtml(label)}</span>
        <span class="sys-bar-detail">${T.escapeHtml(detail)}</span>
      </div>
      <div class="sys-bar-track">
        <div class="sys-bar-fill" style="width:${pct}%;background:${color}"></div>
      </div>
    </div>`;
  }

  function renderAdminUsers(users) {
    if (!users.length) return '<div class="empty-state compact">No users configured.</div>';
    return `<table class="data-table compact-table">
      <thead><tr><th>Name</th><th>Role</th><th>Created</th><th class="col-actions">Actions</th></tr></thead>
      <tbody>${users
        .map((user) => {
          const nextRole = user.role === "admin" ? "user" : "admin";
          return `<tr>
            <td><strong>${T.escapeHtml(user.name)}</strong><div class="row-meta mono">${T.escapeHtml(user.id)}</div></td>
            <td><span class="state-badge ${user.role === "admin" ? "public" : "private"}">${T.escapeHtml(user.role)}</span></td>
            <td>${T.escapeHtml(T.formatDate(user.created_at))}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-admin-role="${T.escapeHtml(user.id)}" data-role="${nextRole}">Make ${nextRole}</button>
              <button type="button" class="btn btn-text btn-sm" data-admin-reset-code="${T.escapeHtml(user.id)}">Reset code</button>
              <button type="button" class="btn btn-text btn-sm" data-admin-revoke-user-sessions="${T.escapeHtml(user.id)}">Revoke sessions</button>
              <button type="button" class="btn btn-text btn-sm" data-admin-revoke-user-devices="${T.escapeHtml(user.id)}">Revoke devices</button>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-admin-delete-user="${T.escapeHtml(user.id)}">Delete</button>
            </td>
          </tr>`;
        })
        .join("")}</tbody>
    </table>`;
  }

  function renderAdminDevices(devices) {
    if (!devices.length) return '<div class="empty-state compact">No trusted devices.</div>';
    return `<table class="data-table compact-table">
      <thead><tr><th>Device</th><th>User</th><th>Last seen</th><th>IP</th><th class="col-actions">Actions</th></tr></thead>
      <tbody>${devices
        .map(
          (device) => `<tr>
            <td><strong>${T.escapeHtml(device.device_name || "Unnamed device")}</strong><div class="row-meta mono">${T.escapeHtml(device.device_token.slice(0, 12))}</div></td>
            <td>${T.escapeHtml(device.user_id)} <span class="tag">${T.escapeHtml(device.role)}</span></td>
            <td>${T.escapeHtml(T.formatDate(device.last_seen_at))}</td>
            <td class="mono">${T.escapeHtml(device.last_ip || "—")}</td>
            <td class="col-actions"><button type="button" class="btn btn-text btn-sm btn-danger" data-admin-revoke-device="${T.escapeHtml(device.device_token)}">Revoke</button></td>
          </tr>`
        )
        .join("")}</tbody>
    </table>`;
  }

  function renderAdminSessions(sessions) {
    if (!sessions.length) return '<div class="empty-state compact">No active sessions.</div>';
    return `<table class="data-table compact-table">
      <thead><tr><th>User</th><th>Kind</th><th>Created</th><th>Expires</th><th class="col-actions">Actions</th></tr></thead>
      <tbody>${sessions
        .map((session) => {
          const userName = session.user_name || session.user_id || "Legacy";
          const role = session.role ? `<span class="tag">${T.escapeHtml(session.role)}</span>` : "";
          const current = session.current ? '<span class="state-badge public">Current</span>' : "";
          return `<tr>
            <td><strong>${T.escapeHtml(userName)}</strong><div class="row-meta mono">${T.escapeHtml(session.user_id || session.token_preview)} ${role}${current}</div></td>
            <td><span class="type-pill">${T.escapeHtml(session.kind)}</span></td>
            <td>${T.escapeHtml(T.formatDate(session.created_at))}</td>
            <td>${T.escapeHtml(T.formatDate(session.expires_at))}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm btn-danger" data-admin-revoke-session="${T.escapeHtml(session.token)}">Revoke</button>
            </td>
          </tr>`;
        })
        .join("")}</tbody>
    </table>`;
  }

  let adminSelectedIds = new Set();

  function updateBulkBar() {
    const bar = T.$("#admin-bulk-bar");
    if (!bar) return;
    const count = adminSelectedIds.size;
    if (count > 0) {
      bar.classList.remove("hidden");
      const label = bar.querySelector(".bulk-bar-label");
      if (label) label.textContent = `${count} file${count !== 1 ? "s" : ""} selected`;
    } else {
      bar.classList.add("hidden");
    }
  }

  function renderAdminFiles(files) {
    adminSelectedIds.clear();
    updateBulkBar();
    if (!files.length) return '<div class="empty-state compact">No files found.</div>';
    return `<table class="data-table compact-table">
      <thead><tr>
        <th style="width:32px"><input type="checkbox" id="admin-select-all" title="Select all"></th>
        <th>Name</th><th>Size</th><th>Folder</th><th>Visibility</th><th class="col-actions">Actions</th>
      </tr></thead>
      <tbody>${files
        .map((file) => {
          const nextVis = file.visibility === "public" ? "private" : "public";
          return `<tr>
            <td><input type="checkbox" class="admin-file-check" data-file-id="${T.escapeHtml(file.id)}"></td>
            <td><strong>${T.escapeHtml(file.name)}</strong><div class="row-meta mono">${T.escapeHtml(file.id)}</div></td>
            <td class="mono">${T.escapeHtml(T.formatBytes(file.size_bytes))}</td>
            <td>${T.escapeHtml(file.folder_path || "Bucket root")}</td>
            <td>${T.stateBadge(file.visibility)}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-preview-file="${T.escapeHtml(file.id)}">Preview</button>
              <button type="button" class="btn btn-text btn-sm" data-vis="${T.escapeHtml(file.id)}" data-v="${nextVis}">${nextVis === "public" ? "Public" : "Private"}</button>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-admin-delete-file="${T.escapeHtml(file.id)}">Delete</button>
            </td>
          </tr>`;
        })
        .join("")}</tbody>
    </table>`;
  }

  function bindAdminFileCheckboxes() {
    const container = T.$("#admin-files");
    if (!container || container.dataset.checkBound) return;
    container.dataset.checkBound = "1";
    container.addEventListener("change", (e) => {
      const box = e.target.closest(".admin-file-check");
      const all = e.target.closest("#admin-select-all");
      if (box) {
        const id = box.dataset.fileId;
        if (box.checked) adminSelectedIds.add(id); else adminSelectedIds.delete(id);
        updateBulkBar();
      } else if (all) {
        container.querySelectorAll(".admin-file-check").forEach((cb) => {
          cb.checked = all.checked;
          if (all.checked) adminSelectedIds.add(cb.dataset.fileId);
          else adminSelectedIds.delete(cb.dataset.fileId);
        });
        updateBulkBar();
      }
    });
  }

  let adminAllFiles = [];

  function renderFilteredAdminFiles() {
    const filesEl = T.$("#admin-files");
    if (!filesEl) return;
    const q = (T.$("#admin-files-search")?.value || "").toLowerCase().trim();
    const visible = q ? adminAllFiles.filter((f) =>
      (f.name || "").toLowerCase().includes(q) ||
      (f.id || "").toLowerCase().includes(q) ||
      (f.folder_path || "").toLowerCase().includes(q)
    ) : adminAllFiles;
    filesEl.innerHTML = renderAdminFiles(visible);
    delete filesEl.dataset.checkBound;
    bindAdminFileCheckboxes();
  }

  T.loadAdminFiles = async function loadAdminFiles() {
    const filesEl = T.$("#admin-files");
    filesEl.innerHTML = "Loading files…";
    try {
      const data = await T.api("/admin/files?limit=200");
      adminAllFiles = data.files || [];
      renderFilteredAdminFiles();
      const searchEl = T.$("#admin-files-search");
      if (searchEl && !searchEl.dataset.bound) {
        searchEl.dataset.bound = "1";
        searchEl.addEventListener("input", renderFilteredAdminFiles);
      }
    } catch (error) {
      filesEl.innerHTML = `<div class="empty-state error">${T.escapeHtml(error.message)}</div>`;
    }
  };

  T.bindAdminBulkBar = function bindAdminBulkBar() {
    const deleteBtn = T.$("#admin-bulk-delete");
    const clearBtn = T.$("#admin-bulk-clear");
    if (deleteBtn && !deleteBtn.dataset.bound) {
      deleteBtn.dataset.bound = "1";
      deleteBtn.addEventListener("click", async () => {
        const ids = [...adminSelectedIds];
        if (!ids.length) return;
        if (!confirm(`Delete ${ids.length} file${ids.length !== 1 ? "s" : ""}? This cannot be undone.`)) return;
        let ok = 0, failed = 0;
        for (const id of ids) {
          try {
            await T.api("/files/" + encodeURIComponent(id), { method: "DELETE" });
            ok++;
          } catch (_) { failed++; }
        }
        if (failed === 0) T.showBanner(`Deleted ${ok} file${ok !== 1 ? "s" : ""}`, "success");
        else T.showBanner(`${ok} deleted, ${failed} failed`, "error");
        await T.loadAdminFiles();
      });
    }
    if (clearBtn && !clearBtn.dataset.bound) {
      clearBtn.dataset.bound = "1";
      clearBtn.addEventListener("click", () => {
        adminSelectedIds.clear();
        T.$("#admin-files")?.querySelectorAll(".admin-file-check, #admin-select-all").forEach((cb) => { cb.checked = false; });
        updateBulkBar();
      });
    }
  };

  T.loadAdmin = async function loadAdmin() {
    const overview = T.$("#admin-overview");
    const system = T.$("#admin-system");
    const usersEl = T.$("#admin-users");
    const devicesEl = T.$("#admin-devices");
    const sessionsEl = T.$("#admin-sessions");
    overview.innerHTML = "Loading…";
    system.innerHTML = "Loading…";
    usersEl.innerHTML = "Loading…";
    devicesEl.innerHTML = "Loading…";
    sessionsEl.innerHTML = "Loading…";
    try {
      const [ov, sys, users, devices, sessions] = await Promise.all([
        T.api("/admin/overview"),
        T.api("/admin/system"),
        T.api("/admin/users"),
        T.api("/admin/devices"),
        T.api("/admin/sessions?limit=100"),
      ]);
      function adminMetric(icon, label, value, color, alert) {
        return `<div class="admin-metric-card ${alert ? "admin-metric-alert" : ""}">
          <div class="admin-metric-icon" style="color:${color}">${icon}</div>
          <div class="admin-metric-body">
            <div class="admin-metric-value" ${alert ? `style="color:var(--danger)"` : ""}>${value}</div>
            <div class="admin-metric-label">${label}</div>
          </div>
        </div>`;
      }
      const corruptAlert = ov.corrupt_file_count > 0;
      overview.innerHTML = `<div class="admin-metrics-grid">
        ${adminMetric("📁", "Files", ov.file_count, "var(--blue)", false)}
        ${adminMetric("📝", "Notes", ov.note_count, "var(--yellow)", false)}
        ${adminMetric("⌨️", "Workspaces", ov.workspace_count ?? "—", "var(--cyan)", false)}
        ${adminMetric("📌", "Pinned", ov.pinned_count, "var(--violet)", false)}
        ${adminMetric("🏷️", "Tags", ov.tag_count, "var(--green)", false)}
        ${adminMetric("💾", "Storage used", T.escapeHtml(T.formatBytes(ov.storage_bytes_used)), "var(--brand)", false)}
        ${adminMetric("⚠️", "Corrupt files", ov.corrupt_file_count, "var(--danger)", corruptAlert)}
      </div>
      <div class="admin-version-bar">Server version <strong>v${T.escapeHtml(ov.version || "—")}</strong></div>`;
      const memUsed = (sys.total_memory_bytes || 0) - (sys.available_memory_bytes || 0);
      const memPct = sys.total_memory_bytes > 0 ? Math.round(memUsed / sys.total_memory_bytes * 100) : 0;
      const diskUsed = (sys.data_dir_total_bytes || 0) - (sys.data_dir_free_bytes || 0);
      const diskPct = sys.data_dir_total_bytes > 0 ? Math.round(diskUsed / sys.data_dir_total_bytes * 100) : 0;
      const loadPct = Math.min(100, Math.round((sys.load_average_1m || 0) * 50));
      system.innerHTML = `<div class="admin-sys-layout">
        <dl class="admin-dl">
          <dt>Host</dt><dd>${T.escapeHtml(sys.hostname)}</dd>
          <dt>OS</dt><dd>${T.escapeHtml(sys.os)}</dd>
          <dt>Arch</dt><dd>${T.escapeHtml(sys.arch)}</dd>
          <dt>Uptime</dt><dd>${sys.uptime_seconds ? T.formatUptime(sys.uptime_seconds) : "—"}</dd>
        </dl>
        <div class="sys-bars">
          ${sysBar("CPU load", `${Number(sys.load_average_1m || 0).toFixed(2)} avg`, loadPct)}
          ${sysBar("Memory", `${T.formatBytes(memUsed)} / ${T.formatBytes(sys.total_memory_bytes)}`, memPct)}
          ${sysBar("Disk", `${T.formatBytes(diskUsed)} / ${T.formatBytes(sys.data_dir_total_bytes)}`, diskPct)}
        </div>
      </div>`;
      usersEl.innerHTML = renderAdminUsers(users.users || []);
      devicesEl.innerHTML = renderAdminDevices(devices.devices || []);
      sessionsEl.innerHTML = renderAdminSessions(sessions.sessions || []);
      T.loadAdminFiles();
      T.bindAdminBulkBar();
      T.loadConsoleCommands();
    } catch (error) {
      overview.innerHTML = `<div class="empty-state error">${T.escapeHtml(error.message)}</div>`;
      system.innerHTML = "";
      usersEl.innerHTML = "";
      devicesEl.innerHTML = "";
      sessionsEl.innerHTML = "";
      T.$("#admin-files").innerHTML = "";
    }
  };

  T.createAdminUser = async function createAdminUser() {
    const name = T.$("#admin-user-name").value.trim();
    const code = T.$("#admin-user-code").value;
    const role = T.$("#admin-user-role").value;
    if (!name || code.length < 4) {
      T.showBanner("Name and a 4+ character access code are required", "error");
      return;
    }
    try {
      await T.api("/admin/users", {
        method: "POST",
        body: JSON.stringify({ name, code, role }),
      });
      T.$("#admin-create-user-form").reset();
      T.showBanner("User created", "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminSetUserRole = async function adminSetUserRole(id, role) {
    if (!confirm(`Change this user role to ${role}?`)) return;
    try {
      await T.api("/admin/users/" + encodeURIComponent(id) + "/role", {
        method: "PUT",
        body: JSON.stringify({ role }),
      });
      T.showBanner("Role updated", "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminResetCode = async function adminResetCode(id) {
    const code = prompt("New access code (minimum 4 characters)");
    if (!code) return;
    try {
      await T.api("/admin/users/" + encodeURIComponent(id) + "/reset-code", {
        method: "POST",
        body: JSON.stringify({ code }),
      });
      T.showBanner("Access code reset", "success");
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminDeleteUser = async function adminDeleteUser(id) {
    if (!confirm("Delete this user? The last admin cannot be deleted.")) return;
    try {
      await T.api("/admin/users/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("User deleted", "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminRevokeDevice = async function adminRevokeDevice(token) {
    if (!confirm("Revoke this trusted device?")) return;
    try {
      await T.api("/admin/devices/" + encodeURIComponent(token), { method: "DELETE" });
      T.showBanner("Device revoked", "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminRevokeUserDevices = async function adminRevokeUserDevices(id) {
    if (!confirm("Revoke all trusted devices for this user?")) return;
    try {
      const result = await T.api("/admin/users/" + encodeURIComponent(id) + "/devices", {
        method: "DELETE",
      });
      T.showBanner(`Revoked ${result.removed || 0} device(s)`, "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminRevokeSession = async function adminRevokeSession(token) {
    if (!confirm("Revoke this active session?")) return;
    try {
      await T.api("/admin/sessions/" + encodeURIComponent(token), { method: "DELETE" });
      T.showBanner("Session revoked", "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminRevokeUserSessions = async function adminRevokeUserSessions(id) {
    if (!confirm("Revoke all active sessions for this user?")) return;
    try {
      const result = await T.api("/admin/users/" + encodeURIComponent(id) + "/sessions", {
        method: "DELETE",
      });
      T.showBanner(`Revoked ${result.removed || 0} session(s)`, "success");
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminDeleteFile = async function adminDeleteFile(id) {
    if (!confirm("Delete this file as admin?")) return;
    try {
      await T.api("/admin/files/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("File deleted", "success");
      T.loadAdminFiles();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  T.adminCleanup = async function adminCleanup(kind) {
    const path = kind === "temp" ? "/admin/cleanup/temp" : "/admin/cleanup/sessions";
    try {
      const result = await T.api(path, { method: "POST" });
      T.showBanner(
        kind === "temp"
          ? `Removed ${result.removed ?? 0} temp file(s)`
          : result.message || "Expired sessions cleaned up",
        "success"
      );
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

  // Admin console

  const consoleHistory = [];

  T.loadConsoleCommands = async function loadConsoleCommands() {
    const el = T.$("#console-commands");
    if (!el) return;
    try {
      const data = await T.api("/admin/console/commands");
      const commands = data.commands || [];
      const byCategory = new Map();
      for (const cmd of commands) {
        if (!byCategory.has(cmd.category)) byCategory.set(cmd.category, []);
        byCategory.get(cmd.category).push(cmd);
      }
      let html = "";
      for (const [cat, cmds] of byCategory) {
        html += `<div class="console-category">${T.escapeHtml(cat)}</div>`;
        for (const cmd of cmds) {
          html += `<button type="button" class="console-cmd-btn" data-console-cmd="${T.escapeHtml(cmd.name)}">
            <span class="console-cmd-name">${T.escapeHtml(cmd.name)}</span>
            <span class="console-cmd-desc">${T.escapeHtml(cmd.description)}</span>
          </button>`;
        }
      }
      el.innerHTML = html || "No commands available.";
    } catch (error) {
      el.innerHTML = `<span class="muted">${T.escapeHtml(error.message)}</span>`;
    }
  };

  function highlightJson(json) {
    return T.escapeHtml(json).replace(
      /("(?:[^"\\]|\\.)*")(\s*:)|("(?:[^"\\]|\\.)*")|(true|false|null)|(-?\d+\.?\d*(?:[eE][+-]?\d+)?)/g,
      (_, key, colon, str, kw, num) => {
        if (key && colon) return `<span class="json-key">${key}</span>${colon}`;
        if (str) return `<span class="json-str">${str}</span>`;
        if (kw) return `<span class="json-kw">${kw}</span>`;
        if (num) return `<span class="json-num">${num}</span>`;
        return _;
      }
    );
  }

  T.runConsoleCommand = async function runConsoleCommand(command) {
    const outputEl = T.$("#console-output");
    if (!outputEl) return;
    outputEl.innerHTML = `<span class="console-hint">Running <strong>${T.escapeHtml(command)}</strong>…</span>`;
    try {
      const result = await T.api("/admin/console/run", {
        method: "POST",
        body: JSON.stringify({ command }),
      });
      const ts = new Date(result.ran_at_ms).toLocaleTimeString();
      const json = JSON.stringify(result.output, null, 2);
      const statusClass = result.success ? "console-ok" : "console-err";
      outputEl.innerHTML = `<div class="console-result-header ${statusClass}">
        <strong>${T.escapeHtml(command)}</strong>
        <span>${ts}</span>
        <span class="${statusClass}">${result.success ? "✓ ok" : "✗ failed"}</span>
      </div>
      <pre class="console-json">${highlightJson(json)}</pre>`;
      consoleHistory.unshift({ command, ts, success: result.success });
      renderConsoleHistory();
    } catch (error) {
      outputEl.innerHTML = `<div class="console-result-header console-err"><strong>${T.escapeHtml(command)}</strong><span class="console-err">${T.escapeHtml(error.message)}</span></div>`;
    }
  };

  function renderConsoleHistory() {
    const el = T.$("#console-history");
    if (!el) return;
    if (!consoleHistory.length) {
      el.innerHTML = '<span class="console-hint">No commands run yet</span>';
      return;
    }
    el.innerHTML = consoleHistory
      .slice(0, 20)
      .map(
        (item) =>
          `<button type="button" class="console-history-item ${item.success ? "" : "console-err"}" data-console-cmd="${T.escapeHtml(item.command)}">
            <span>${T.escapeHtml(item.command)}</span>
            <span class="muted">${T.escapeHtml(item.ts)}</span>
          </button>`
      )
      .join("");
  }
})(window.Tssp);
