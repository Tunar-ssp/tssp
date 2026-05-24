window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

T.searchQueryString = function searchQueryString(q) {
    const params = new URLSearchParams({ q, limit: "50" });
    const kind = T.$("#search-kind")?.value;
    if (kind && kind !== "all") params.set("kind", kind);
    const tag = T.$("#search-tag")?.value.trim();
    if (tag) params.set("tag", tag);
    const mime = T.$("#search-type")?.value.trim();
    if (mime) params.set("type", mime);
    const visibility = T.$("#search-visibility")?.value;
    if (visibility) params.set("visibility", visibility);
    if (T.$("#search-pinned")?.checked) params.set("pinned", "true");
    return params.toString();
  };

  T.runSearch = async function runSearch(q) {
    const body = T.$("#search-body");
    const resultsEl = T.$("#search-results") || body;
    const sub = T.$("#search-subtitle");
    if (!resultsEl) return;
    if (!q || q.length < 1) {
      if (sub) sub.textContent = "Find files, notes, and workspaces from one place.";
      if (resultsEl) {
        resultsEl.innerHTML =
          '<div id="search-body" class="empty-state compact"><strong>Start with the global search bar</strong><span>Use filters here to narrow by kind, tags, MIME type, visibility, or pin state.</span></div>';
      }
      return;
    }
    const filterParts = [];
    const kind = T.$("#search-kind")?.value;
    if (kind && kind !== "all") filterParts.push(kind);
    if (T.$("#search-tag")?.value.trim()) filterParts.push("tag");
    if (T.$("#search-type")?.value.trim()) filterParts.push("type");
    if (T.$("#search-visibility")?.value) filterParts.push(T.$("#search-visibility").value);
    if (T.$("#search-pinned")?.checked) filterParts.push("pinned");
    if (sub) {
      sub.textContent =
        filterParts.length > 0
          ? `Results for "${q}" (${filterParts.join(", ")})`
          : `Results for "${q}"`;
    }
    if (resultsEl) resultsEl.innerHTML = '<div class="empty-state compact">Searching...</div>';
    T.setView("search");
    try {
      const searchData = await T.api("/search?" + T.searchQueryString(q));
      const results = searchData.results || [];
      if (!results.length) {
        if (sub) sub.textContent = `No matches for "${q}"`;
        if (resultsEl) {
          resultsEl.innerHTML =
            '<div id="search-body" class="empty-state compact"><strong>No matches</strong><span>Try a broader query or clear filters.</span></div>';
        }
        return;
      }
      const countLabel = `${searchData.result_count ?? results.length} result${results.length !== 1 ? "s" : ""}`;
      if (sub) {
        sub.textContent = filterParts.length > 0
          ? `${countLabel} for "${q}" (${filterParts.join(", ")})`
          : `${countLabel} for "${q}"`;
      }
      resultsEl.innerHTML = results
        .map((result) => {
          const type = result.type || "item";
          const name = T.escapeHtml(result.title || result.name || result.id);
          const id = T.escapeHtml(result.id);
          const vis = result.visibility != null ? T.stateBadge(result.visibility) : "";
          const tags = T.tagsHtml(result.tags);
          let actions = "";
          let detail = "";
          let extra = "";
          if (type === "file") {
            detail = T.escapeHtml((result.folder_path || "Bucket root") + " · " + T.formatBytes(result.size_bytes));
            extra = `${vis}${tags}`;
            actions = `<button type="button" class="btn btn-text btn-sm" data-preview-file="${id}">Preview</button><a class="btn btn-text btn-sm" href="${T.fileDownloadUrl(result.id)}" download>Download</a>`;
          } else if (type === "note") {
            const rawSnippet = result.snippet || (result.body || "")
              .trim()
              .replace(/^#+\s+/gm, "")
              .replace(/\*\*?([^*]+)\*\*?/g, "$1")
              .replace(/`([^`]+)`/g, "$1")
              .replace(/\n+/g, " ")
              .trim()
              .slice(0, 120);
            detail = T.escapeHtml(rawSnippet);
            extra = (result.pinned ? '<span class="pin" title="Pinned">*</span> ' : "") + tags;
            actions = `<button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Open</button>`;
          } else if (type === "workspace") {
            detail = T.escapeHtml((result.snippet || "").slice(0, 120));
            extra = `<span class="type-pill">${T.escapeHtml(result.language || "text")}</span>`;
            actions = `<button type="button" class="btn btn-text btn-sm" data-ws-edit="${id}">Open</button>`;
          }
          const typeLabel = { file: "File", note: "Note", workspace: "Workspace" }[type] || type;
          return `<article class="search-result-card">
            <div class="search-result-main">
              <span class="search-result-type search-type-${T.escapeHtml(type)}">${T.escapeHtml(typeLabel)}</span>
              <div class="search-result-name"><strong>${name}</strong></div>
              ${detail ? `<div class="row-meta">${detail}</div>` : ""}
              ${extra ? `<div class="search-result-extra">${extra}</div>` : ""}
            </div>
            <div class="search-result-actions">${actions}</div>
          </article>`;
        })
        .join("");
    } catch (error) {
      if (resultsEl) {
        resultsEl.innerHTML = `<div id="search-body" class="empty-state error">${T.escapeHtml(error.message)}</div>`;
      }
    }
  };

})(window.Tssp);
