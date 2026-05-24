window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.showBanner = function showBanner(msg, kind = "info") {
    const el = T.$("#banner");
    if (!el) return;
    if (!msg) {
      el.classList.add("hidden");
      return;
    }
    el.textContent = msg;
    el.className = `banner ${kind}`;
    el.classList.remove("hidden");
  };
})(window.Tssp);
