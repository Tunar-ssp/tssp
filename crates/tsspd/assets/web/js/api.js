window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.API = "/api/v1";
  T.SEARCH_DEBOUNCE_MS = 350;

  T.$ = (sel, root = document) => root.querySelector(sel);
  T.$$ = (sel, root = document) => [...root.querySelectorAll(sel)];

  T.escapeHtml = function escapeHtml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  };

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

  T.showBanner = function showBanner(msg, kind = "info") {
    const el = T.$("#banner");
    if (!msg) {
      el.classList.add("hidden");
      return;
    }
    el.textContent = msg;
    el.className = `banner ${kind}`;
    el.classList.remove("hidden");
  };

  T.fileDownloadUrl = function fileDownloadUrl(id) {
    return `/api/v1/files/${encodeURIComponent(id)}/content?disposition=attachment`;
  };

  T.fileInlineUrl = function fileInlineUrl(id) {
    return `/api/v1/files/${encodeURIComponent(id)}/content?disposition=inline`;
  };

  T.fileThumbnailUrl = function fileThumbnailUrl(id) {
    return `/api/v1/files/${encodeURIComponent(id)}/thumbnail`;
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

  T.copyText = async function copyText(value) {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(value);
      return;
    }
    const input = document.createElement("textarea");
    input.value = value;
    input.setAttribute("readonly", "");
    input.style.position = "fixed";
    input.style.opacity = "0";
    document.body.appendChild(input);
    input.select();
    document.execCommand("copy");
    input.remove();
  };

  T.simpleMarkdown = function simpleMarkdown(markdown) {
    const lines = String(markdown || "").split(/\r?\n/);
    const html = [];
    let inList = false;
    let inOrderedList = false;
    let inCheckList = false;
    let inCodeBlock = false;
    let codeLang = "";
    let codeLines = [];
    let inTable = false;
    let tableRows = [];

    function closeList() {
      if (inCheckList) { html.push("</ul>"); inCheckList = false; }
      if (inList) { html.push("</ul>"); inList = false; }
      if (inOrderedList) { html.push("</ol>"); inOrderedList = false; }
    }

    function flushTable() {
      if (!inTable) return;
      inTable = false;
      if (!tableRows.length) return;
      const [headerRow, ...bodyRows] = tableRows;
      tableRows = [];
      const thCells = headerRow.map((cell) => `<th>${inlineFormat(cell.trim())}</th>`).join("");
      const tbodyHtml = bodyRows
        .filter((row) => !row.every((cell) => /^[-: ]+$/.test(cell)))
        .map((row) => `<tr>${row.map((cell) => `<td>${inlineFormat(cell.trim())}</td>`).join("")}</tr>`)
        .join("");
      html.push(`<div class="md-table-wrap"><table class="md-table"><thead><tr>${thCells}</tr></thead><tbody>${tbodyHtml}</tbody></table></div>`);
    }

    function inlineFormat(text) {
      return T.escapeHtml(text)
        .replace(/`([^`]+)`/g, "<code>$1</code>")
        .replace(/\*\*\*([^*]+)\*\*\*/g, "<strong><em>$1</em></strong>")
        .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
        .replace(/\*([^*]+)\*/g, "<em>$1</em>")
        .replace(/___([^_]+)___/g, "<strong><em>$1</em></strong>")
        .replace(/__([^_]+)__/g, "<strong>$1</strong>")
        .replace(/_([^_]+)_/g, "<em>$1</em>")
        .replace(/~~([^~]+)~~/g, "<del>$1</del>")
        .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" rel="noopener noreferrer" target="_blank">$1</a>');
    }

    for (const line of lines) {
      if (line.startsWith("```")) {
        if (inCodeBlock) {
          const lang = T.escapeHtml(codeLang);
          html.push(`<pre><code${lang ? ` class="lang-${lang}"` : ""}>${T.escapeHtml(codeLines.join("\n"))}</code></pre>`);
          codeLines = [];
          codeLang = "";
          inCodeBlock = false;
        } else {
          closeList();
          flushTable();
          codeLang = line.slice(3).trim();
          inCodeBlock = true;
        }
        continue;
      }
      if (inCodeBlock) { codeLines.push(line); continue; }

      // Table rows
      if (line.startsWith("|") && line.includes("|", 1)) {
        closeList();
        if (!inTable) inTable = true;
        const cells = line.split("|").slice(1, -1);
        tableRows.push(cells);
        continue;
      } else {
        flushTable();
      }

      const headingMatch = line.match(/^(#{1,6})\s+(.*)/);
      if (headingMatch) {
        closeList();
        const level = Math.min(headingMatch[1].length + 1, 6);
        html.push(`<h${level}>${inlineFormat(headingMatch[2])}</h${level}>`);
      } else if (line.startsWith("> ")) {
        closeList();
        html.push(`<blockquote>${inlineFormat(line.slice(2))}</blockquote>`);
      } else if (/^\s*-\s+\[[ xX]\]\s+/.test(line)) {
        if (!inCheckList) { closeList(); html.push('<ul class="md-checklist">'); inCheckList = true; }
        const checked = /^\s*-\s+\[[xX]\]/.test(line) ? "checked" : "";
        const text = line.replace(/^\s*-\s+\[[ xX]\]\s+/, "");
        html.push(`<li><input type="checkbox" disabled ${checked}> ${inlineFormat(text)}</li>`);
      } else if (/^\s*[-*]\s+/.test(line)) {
        if (!inList) { closeList(); html.push("<ul>"); inList = true; }
        html.push(`<li>${inlineFormat(line.replace(/^\s*[-*]\s+/, ""))}</li>`);
      } else if (/^\s*\d+\.\s+/.test(line)) {
        if (!inOrderedList) { closeList(); html.push("<ol>"); inOrderedList = true; }
        html.push(`<li>${inlineFormat(line.replace(/^\s*\d+\.\s+/, ""))}</li>`);
      } else if (/^---+$/.test(line.trim())) {
        closeList(); html.push("<hr>");
      } else if (line.trim()) {
        closeList();
        html.push(`<p>${inlineFormat(line)}</p>`);
      } else {
        closeList();
      }
    }
    if (inCodeBlock) html.push(`<pre><code>${T.escapeHtml(codeLines.join("\n"))}</code></pre>`);
    flushTable();
    closeList();
    return html.join("");
  };

  T.api = async function api(path, options = {}) {
    const res = await fetch(T.API + path, {
      credentials: "same-origin",
      headers:
        options.body && !(options.body instanceof FormData)
          ? { "Content-Type": "application/json", ...options.headers }
          : options.headers,
      ...options,
    });
    if (res.status === 401) {
      T.authRequired = true;
      T.showLogin();
      throw new Error("Unauthorized");
    }
    const ct = res.headers.get("content-type") || "";
    const body = ct.includes("application/json")
      ? await res.json()
      : await res.text();
    if (!res.ok) {
      const err =
        typeof body === "object" && body && body.error
          ? body.error.message || body.error
          : res.statusText;
      throw new Error(typeof err === "string" ? err : JSON.stringify(err));
    }
    return body;
  };
})(window.Tssp);
