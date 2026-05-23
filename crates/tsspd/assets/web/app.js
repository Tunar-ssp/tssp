(function () {
  "use strict";

  const API = "/api/v1";
  const SEARCH_DEBOUNCE_MS = 350;

  const $ = (sel, root = document) => root.querySelector(sel);
  const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];

  let authRequired = false;
  let searchTimer = null;
  let editingNoteId = null;

  function escapeHtml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function formatBytes(n) {
    if (n == null || n === 0) return "—";
    const u = ["B", "KB", "MB", "GB", "TB"];
    let i = 0;
    let v = Number(n);
    while (v >= 1024 && i < u.length - 1) {
      v /= 1024;
      i++;
    }
    return `${v.toFixed(i > 0 ? 1 : 0)} ${u[i]}`;
  }

  function formatDate(value) {
    if (value == null || value === "") return "—";
    try {
      const ms =
        typeof value === "number"
          ? value * 1000
          : Date.parse(String(value));
      if (Number.isNaN(ms)) return String(value);
      return new Date(ms).toLocaleString(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
      });
    } catch {
      return String(value);
    }
  }

  function showBanner(msg, kind = "info") {
    const el = $("#banner");
    if (!msg) {
      el.classList.add("hidden");
      return;
    }
    el.textContent = msg;
    el.className = `banner ${kind}`;
    el.classList.remove("hidden");
  }

  async function api(path, options = {}) {
    const res = await fetch(API + path, {
      credentials: "same-origin",
      headers: options.body && !(options.body instanceof FormData)
        ? { "Content-Type": "application/json", ...options.headers }
        : options.headers,
      ...options,
    });
    if (res.status === 401) {
      authRequired = true;
      showLogin();
      throw new Error("Unauthorized");
    }
    const ct = res.headers.get("content-type") || "";
    const body = ct.includes("application/json") ? await res.json() : await res.text();
    if (!res.ok) {
      const err = typeof body === "object" && body && body.error ? body.error : res.statusText;
      throw new Error(err);
    }
    return body;
  }

  async function probeAuth() {
    try {
      const res = await fetch(API + "/auth/required", { credentials: "same-origin" });
      if (!res.ok) return;
      const data = await res.json();
      authRequired = Boolean(data.required);
      if (authRequired) {
        const me = await fetch(API + "/auth/me", { credentials: "same-origin" });
        if (me.status === 401) {
          showLogin();
          return;
        }
      }
      showApp();
    } catch {
      showApp();
    }
  }

  function showLogin() {
    $("#login-screen").classList.remove("hidden");
    $("#login-screen").setAttribute("aria-hidden", "false");
    $("#app").classList.add("hidden");
  }

  function showApp() {
    $("#login-screen").classList.add("hidden");
    $("#login-screen").setAttribute("aria-hidden", "true");
    $("#app").classList.remove("hidden");
    refreshCurrentView();
  }

  function setView(name) {
    $$(".nav-item").forEach((a) => {
      a.classList.toggle("active", a.dataset.view === name);
    });
    $$(".view").forEach((v) => v.classList.add("hidden"));
    const section = $(`#view-${name}`);
    if (section) section.classList.remove("hidden");
    if (name === "search") return;
    refreshView(name);
  }

  function refreshCurrentView() {
    const active = $(".nav-item.active");
    const view = active ? active.dataset.view : "objects";
    refreshView(view);
  }

  function refreshView(view) {
    if (view === "objects") loadFiles();
    else if (view === "notes") loadNotes();
    else if (view === "overview") loadOverview();
  }

  async function loadOverview() {
    const grid = $("#metric-grid");
    grid.innerHTML = '<div class="metric-card"><span class="label">Loading…</span></div>';
    try {
      const s = await api("/status");
      const metrics = [
        ["Files", s.file_count],
        ["Notes", s.note_count],
        ["Tags", s.tag_count],
        ["Pinned", s.pinned_count],
        ["Uploads (24h)", s.recent_upload_count_24h],
        ["Storage", formatBytes(s.storage_bytes_used)],
        ["Uptime", formatUptime(s.uptime_seconds)],
        ["Status", s.status],
      ];
      grid.innerHTML = metrics
        .map(
          ([label, value]) =>
            `<div class="metric-card"><div class="label">${escapeHtml(label)}</div><div class="value">${escapeHtml(String(value))}</div></div>`
        )
        .join("");
    } catch (e) {
      grid.innerHTML = `<div class="metric-card"><span class="label">Error</span><div class="value">${escapeHtml(e.message)}</div></div>`;
    }
  }

  function formatUptime(sec) {
    const h = Math.floor(sec / 3600);
    const m = Math.floor((sec % 3600) / 60);
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  }

  async function loadFiles() {
    const body = $("#files-body");
    body.innerHTML = '<tr><td colspan="5" class="table-empty">Loading…</td></tr>';
    const pinned = $("#pinned-only").checked;
    const qs = pinned ? "?pinned=true&limit=200" : "?limit=200";
    try {
      const data = await api("/files" + qs);
      const files = data.files || [];
      if (!files.length) {
        body.innerHTML = '<tr><td colspan="5" class="table-empty">No objects yet. Upload a file to get started.</td></tr>';
        return;
      }
      body.innerHTML = files
        .map((f) => {
          const pin = f.pinned ? '<span class="pin" title="Pinned">★</span>' : "";
          const name = escapeHtml(f.name || f.id);
          const id = escapeHtml(f.id);
          const download = `/api/v1/files/${id}/content?disposition=attachment`;
          return `<tr>
            <td><div class="file-name">${pin}<span>${name}</span></div></td>
            <td class="mono">${escapeHtml(formatBytes(f.size_bytes))}</td>
            <td class="mono">${escapeHtml(f.mime_type || "—")}</td>
            <td>${escapeHtml(formatDate(f.uploaded_at))}</td>
            <td class="col-actions">
              <a class="btn btn-text btn-sm" href="${download}" download>Download</a>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-delete-file="${id}">Delete</button>
            </td>
          </tr>`;
        })
        .join("");
    } catch (e) {
      body.innerHTML = `<tr><td colspan="5" class="table-empty">${escapeHtml(e.message)}</td></tr>`;
    }
  }

  async function loadNotes() {
    const body = $("#notes-body");
    body.innerHTML = '<tr><td colspan="4" class="table-empty">Loading…</td></tr>';
    try {
      const data = await api("/notes?limit=200");
      const notes = data.notes || [];
      if (!notes.length) {
        body.innerHTML = '<tr><td colspan="4" class="table-empty">No notes yet.</td></tr>';
        return;
      }
      body.innerHTML = notes
        .map((n) => {
          const id = escapeHtml(n.id);
          const tags = (n.tags || [])
            .map((t) => `<span class="tag">${escapeHtml(t)}</span>`)
            .join("");
          return `<tr>
            <td><strong>${escapeHtml(n.title || "Untitled")}</strong></td>
            <td>${escapeHtml(formatDate(n.updated_at))}</td>
            <td>${tags || "—"}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Edit</button>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-delete-note="${id}">Delete</button>
            </td>
          </tr>`;
        })
        .join("");
    } catch (e) {
      body.innerHTML = `<tr><td colspan="4" class="table-empty">${escapeHtml(e.message)}</td></tr>`;
    }
  }

  async function runSearch(q) {
    const body = $("#search-body");
    const sub = $("#search-subtitle");
    if (!q || q.length < 1) {
      sub.textContent = "Type in the search bar above";
      body.innerHTML = '<tr><td colspan="4" class="table-empty">Enter a query to search</td></tr>';
      return;
    }
    sub.textContent = `Results for “${q}”`;
    body.innerHTML = '<tr><td colspan="4" class="table-empty">Searching…</td></tr>';
    setView("search");
    try {
      const data = await api("/search?q=" + encodeURIComponent(q) + "&limit=50");
      const results = data.results || [];
      if (!results.length) {
        body.innerHTML = '<tr><td colspan="4" class="table-empty">No matches</td></tr>';
        return;
      }
      body.innerHTML = results
        .map((r) => {
          const type = escapeHtml(r.type);
          const name = escapeHtml(r.title || r.name || r.id);
          const id = escapeHtml(r.id);
          let actions = "";
          if (r.type === "file") {
            const download = `/api/v1/files/${id}/content?disposition=attachment`;
            actions = `<a class="btn btn-text btn-sm" href="${download}" download>Download</a>`;
          } else if (r.type === "note") {
            actions = `<button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Open</button>`;
          }
          const detail =
            r.type === "file"
              ? escapeHtml(formatBytes(r.size_bytes))
              : escapeHtml((r.snippet || "").slice(0, 80));
          return `<tr>
            <td><span class="tag">${type}</span></td>
            <td><strong>${name}</strong></td>
            <td class="mono">${detail || "—"}</td>
            <td class="col-actions">${actions}</td>
          </tr>`;
        })
        .join("");
    } catch (e) {
      body.innerHTML = `<tr><td colspan="4" class="table-empty">${escapeHtml(e.message)}</td></tr>`;
    }
  }

  async function uploadFiles(fileList) {
    if (!fileList || !fileList.length) return;
    showBanner(`Uploading ${fileList.length} file(s)…`, "info");
    let ok = 0;
    for (const file of fileList) {
      const fd = new FormData();
      fd.append("file", file);
      try {
        await api("/files", { method: "POST", body: fd });
        ok++;
      } catch (e) {
        showBanner(`Upload failed: ${e.message}`, "error");
        return;
      }
    }
    showBanner(`Uploaded ${ok} file(s)`, "success");
    setTimeout(() => showBanner(""), 3000);
    loadFiles();
  }

  async function deleteFile(id) {
    if (!confirm("Delete this object permanently?")) return;
    try {
      await api("/files/" + encodeURIComponent(id), { method: "DELETE" });
      showBanner("Object deleted", "success");
      loadFiles();
    } catch (e) {
      showBanner(e.message, "error");
    }
  }

  async function deleteNote(id) {
    if (!confirm("Delete this note?")) return;
    try {
      await api("/notes/" + encodeURIComponent(id), { method: "DELETE" });
      showBanner("Note deleted", "success");
      loadNotes();
    } catch (e) {
      showBanner(e.message, "error");
    }
  }

  function openNoteDialog(note) {
    editingNoteId = note ? note.id : null;
    $("#note-dialog-title").textContent = note ? "Edit note" : "New note";
    $("#note-title-input").value = note ? note.title || "" : "";
    $("#note-body-input").value = note ? note.body || "" : "";
    $("#note-dialog").showModal();
  }

  async function saveNote() {
    const title = $("#note-title-input").value.trim();
    const body = $("#note-body-input").value;
    const payload = { body };
    if (title) payload.title = title;
    try {
      if (editingNoteId) {
        await api("/notes/" + encodeURIComponent(editingNoteId), {
          method: "PUT",
          body: JSON.stringify(payload),
        });
      } else {
        await api("/notes", { method: "POST", body: JSON.stringify(payload) });
      }
      $("#note-dialog").close();
      showBanner("Note saved", "success");
      loadNotes();
    } catch (e) {
      showBanner(e.message, "error");
    }
  }

  async function openNote(id) {
    try {
      const note = await api("/notes/" + encodeURIComponent(id));
      openNoteDialog(note);
    } catch (e) {
      showBanner(e.message, "error");
    }
  }

  function bindEvents() {
    $("#login-form").addEventListener("submit", async (ev) => {
      ev.preventDefault();
      const err = $("#login-error");
      err.classList.add("hidden");
      const password = $("#login-password").value;
      try {
        await fetch(API + "/auth/login", {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ password }),
        }).then(async (res) => {
          if (!res.ok) {
            const b = await res.json().catch(() => ({}));
            throw new Error(b.error || "Invalid password");
          }
        });
        showApp();
      } catch (e) {
        err.textContent = e.message;
        err.classList.remove("hidden");
      }
    });

    $$(".nav-item").forEach((a) => {
      a.addEventListener("click", (ev) => {
        ev.preventDefault();
        setView(a.dataset.view);
        $("#side-nav").classList.remove("open");
      });
    });

    $("#nav-toggle").addEventListener("click", () => {
      $("#side-nav").classList.toggle("open");
    });

    $("#refresh-btn").addEventListener("click", () => {
      showBanner("");
      refreshCurrentView();
    });

    $("#pinned-only").addEventListener("change", () => loadFiles());

    $("#upload-btn").addEventListener("click", () => $("#upload-input").click());
    $("#upload-input").addEventListener("change", (ev) => {
      uploadFiles(ev.target.files);
      ev.target.value = "";
    });

    const drop = $("#drop-zone");
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
    drop.addEventListener("drop", (e) => uploadFiles(e.dataTransfer.files));

    $("#global-search").addEventListener("input", (ev) => {
      const q = ev.target.value.trim();
      clearTimeout(searchTimer);
      searchTimer = setTimeout(() => runSearch(q), SEARCH_DEBOUNCE_MS);
    });

    $("#new-note-btn").addEventListener("click", () => openNoteDialog(null));
    $("#note-form").addEventListener("submit", (ev) => {
      ev.preventDefault();
      saveNote();
    });
    $("#note-cancel").addEventListener("click", () => $("#note-dialog").close());
    $("#note-close").addEventListener("click", () => $("#note-dialog").close());

    document.addEventListener("click", (ev) => {
      const delFile = ev.target.closest("[data-delete-file]");
      if (delFile) {
        deleteFile(delFile.dataset.deleteFile);
        return;
      }
      const delNote = ev.target.closest("[data-delete-note]");
      if (delNote) {
        deleteNote(delNote.dataset.deleteNote);
        return;
      }
      const editNote = ev.target.closest("[data-edit-note]");
      if (editNote) {
        openNote(editNote.dataset.editNote);
      }
    });
  }

  bindEvents();
  probeAuth();
})();
