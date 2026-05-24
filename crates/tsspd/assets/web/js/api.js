window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.API = "/api/v1";
  T.SEARCH_DEBOUNCE_MS = 350;

  T.$ = (sel, root = document) => root.querySelector(sel);
  T.$$ = (sel, root = document) => [...root.querySelectorAll(sel)];

  T.hideBootScreen = function hideBootScreen() {
    const boot = T.$("#boot-screen");
    boot?.classList.add("hidden");
  };

  T.showBootError = function showBootError(title, message, details = "") {
    const boot = T.$("#boot-screen");
    if (!boot) {
      return;
    }

    const titleNode = T.$("#boot-title");
    const messageNode = T.$("#boot-message");
    const detailsNode = T.$("#boot-details");

    if (titleNode) {
      titleNode.textContent = title;
    }
    if (messageNode) {
      messageNode.textContent = message;
    }
    if (detailsNode) {
      if (details) {
        detailsNode.textContent = details;
        detailsNode.classList.remove("hidden");
      } else {
        detailsNode.textContent = "";
        detailsNode.classList.add("hidden");
      }
    }

    T.$("#login-screen")?.classList.add("hidden");
    T.$("#app")?.classList.add("hidden");
    boot.classList.remove("hidden");
    document.body.dataset.tsspBootReady = "0";
  };

  T.markBootReady = function markBootReady() {
    document.body.dataset.tsspBootReady = "1";
    T.hideBootScreen();
  };

  T.resetServiceWorker = async function resetServiceWorker() {
    try {
      if ("serviceWorker" in navigator) {
        const registrations = await navigator.serviceWorker.getRegistrations();
        await Promise.all(registrations.map((registration) => registration.unregister()));
      }
      if ("caches" in window) {
        const cacheKeys = await caches.keys();
        await Promise.all(cacheKeys.map((key) => caches.delete(key)));
      }
      location.reload();
    } catch (error) {
      T.showBootError(
        "Unable to reset cache",
        "The browser cache could not be cleared automatically.",
        error instanceof Error ? `${error.name}: ${error.message}` : String(error),
      );
    }
  };

  window.addEventListener("error", (event) => {
    if (document.body.dataset.tsspBootReady === "1") {
      return;
    }
    const error = event.error instanceof Error ? event.error : null;
    T.showBootError(
      "Dashboard failed to load",
      "An unexpected error occurred while starting the UI.",
      error ? `${error.name}: ${error.message}` : String(event.message || "Unknown error"),
    );
  });

  window.addEventListener("unhandledrejection", (event) => {
    if (document.body.dataset.tsspBootReady === "1") {
      return;
    }
    const reason = event.reason instanceof Error ? event.reason : null;
    T.showBootError(
      "Dashboard failed to load",
      "A background task failed while starting the UI.",
      reason ? `${reason.name}: ${reason.message}` : String(event.reason || "Unknown rejection"),
    );
  });

  T.escapeHtml = function escapeHtml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
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
