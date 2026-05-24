window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.tagsHtml = function tagsHtml(tags) {
    return (tags || [])
      .map((tag) => `<span class="tag">${T.escapeHtml(tag)}</span>`)
      .join("");
  };

  T.stateBadge = function stateBadge(value) {
    const isPublic = value === "public";
    return `<span class="state-badge ${isPublic ? "public" : "private"}">${isPublic ? "Public" : "Private"}</span>`;
  };

  T.publicLink = function publicLink(file) {
    return file.public_token ? `${window.location.origin}/p/${file.public_token}` : "";
  };

  T.tableMessage = function tableMessage(columns, message) {
    return `<tr><td colspan="${columns}" class="table-empty">${T.escapeHtml(message)}</td></tr>`;
  };
})(window.Tssp);
