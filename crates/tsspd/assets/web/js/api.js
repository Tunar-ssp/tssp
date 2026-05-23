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
    if (n == null || n === 0) return "—";
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
