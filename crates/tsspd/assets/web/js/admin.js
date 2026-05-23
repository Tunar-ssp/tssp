window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

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

  function renderAdminFiles(files) {
    if (!files.length) return '<div class="empty-state compact">No files found.</div>';
    return `<table class="data-table compact-table">
      <thead><tr><th>Name</th><th>Size</th><th>Folder</th><th>Visibility</th><th class="col-actions">Actions</th></tr></thead>
      <tbody>${files
        .map((file) => {
          const nextVis = file.visibility === "public" ? "private" : "public";
          return `<tr>
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

  T.loadAdminFiles = async function loadAdminFiles() {
    const filesEl = T.$("#admin-files");
    filesEl.innerHTML = "Loading files…";
    try {
      const data = await T.api("/admin/files?limit=50");
      filesEl.innerHTML = renderAdminFiles(data.files || []);
    } catch (error) {
      filesEl.innerHTML = `<div class="empty-state error">${T.escapeHtml(error.message)}</div>`;
    }
  };

  T.loadAdmin = async function loadAdmin() {
    const overview = T.$("#admin-overview");
    const system = T.$("#admin-system");
    const usersEl = T.$("#admin-users");
    const devicesEl = T.$("#admin-devices");
    overview.innerHTML = "Loading…";
    system.innerHTML = "Loading…";
    usersEl.innerHTML = "Loading…";
    devicesEl.innerHTML = "Loading…";
    try {
      const [ov, sys, users, devices] = await Promise.all([
        T.api("/admin/overview"),
        T.api("/admin/system"),
        T.api("/admin/users"),
        T.api("/admin/devices"),
      ]);
      overview.innerHTML = `<dl class="admin-dl">
        <dt>Files</dt><dd>${ov.file_count}</dd>
        <dt>Notes</dt><dd>${ov.note_count}</dd>
        <dt>Tags</dt><dd>${ov.tag_count}</dd>
        <dt>Pinned</dt><dd>${ov.pinned_count}</dd>
        <dt>Corrupt</dt><dd>${ov.corrupt_file_count}</dd>
        <dt>Storage</dt><dd>${T.escapeHtml(T.formatBytes(ov.storage_bytes_used))}</dd>
        <dt>Version</dt><dd>${T.escapeHtml(ov.version || "—")}</dd>
      </dl>`;
      system.innerHTML = `<dl class="admin-dl">
        <dt>Host</dt><dd>${T.escapeHtml(sys.hostname)}</dd>
        <dt>OS</dt><dd>${T.escapeHtml(sys.os)} / ${T.escapeHtml(sys.arch)}</dd>
        <dt>Load 1m</dt><dd>${Number(sys.load_average_1m || 0).toFixed(2)}</dd>
        <dt>Memory</dt><dd>${T.escapeHtml(T.formatBytes(sys.available_memory_bytes))} free / ${T.escapeHtml(T.formatBytes(sys.total_memory_bytes))}</dd>
        <dt>Data disk</dt><dd>${T.escapeHtml(T.formatBytes(sys.data_dir_free_bytes))} free / ${T.escapeHtml(T.formatBytes(sys.data_dir_total_bytes))}</dd>
      </dl>`;
      usersEl.innerHTML = renderAdminUsers(users.users || []);
      devicesEl.innerHTML = renderAdminDevices(devices.devices || []);
      T.loadAdminFiles();
    } catch (error) {
      overview.innerHTML = `<div class="empty-state error">${T.escapeHtml(error.message)}</div>`;
      system.innerHTML = "";
      usersEl.innerHTML = "";
      devicesEl.innerHTML = "";
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
          : result.message || "Cleanup requested",
        "success"
      );
      T.loadAdmin();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };
})(window.Tssp);
