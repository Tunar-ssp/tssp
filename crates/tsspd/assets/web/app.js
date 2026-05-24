(() => {
  "use strict";

  const VERSION = "2026-05-24-4";
  const MODULES = [
    "/assets/js/api.js?v=" + VERSION,
    "/assets/js/ui/format.js?v=" + VERSION,
    "/assets/js/ui/render.js?v=" + VERSION,
    "/assets/js/ui/toast.js?v=" + VERSION,
    "/assets/js/ui/dialogs.js?v=" + VERSION,
    "/assets/js/state.js?v=" + VERSION,
    "/assets/js/upload.js?v=" + VERSION,
    "/assets/js/features/overview.js?v=" + VERSION,
    "/assets/js/features/search.js?v=" + VERSION,
    "/assets/js/features/media.js?v=" + VERSION,
    "/assets/js/features/public.js?v=" + VERSION,
    "/assets/js/features/workspaces.js?v=" + VERSION,
    "/assets/js/files.js?v=" + VERSION,
    "/assets/js/notes.js?v=" + VERSION,
    "/assets/js/admin.js?v=" + VERSION,
    "/assets/js/editor.js?v=" + VERSION,
    "/assets/js/app.js?v=" + VERSION,
  ];

  function loadScript(src) {
    return new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = src;
      script.async = false;
      script.onload = () => resolve();
      script.onerror = () => reject(new Error(`Failed to load ${src}`));
      document.head.appendChild(script);
    });
  }

  async function bootstrapLegacyDashboard() {
    if (window.Tssp?.probeAuth) {
      return;
    }
    if (document.body) {
      document.body.dataset.tsspLegacyBootstrap = "1";
    }

    try {
      for (const src of MODULES) {
        if (window.Tssp?.probeAuth) {
          return;
        }
        await loadScript(src);
      }
    } catch (error) {
      console.error("Legacy dashboard bootstrap failed:", error);
      window.Tssp?.showBootError?.(
        "Dashboard failed to load",
        "This browser loaded the deprecated dashboard entrypoint and could not upgrade itself.",
        error instanceof Error ? `${error.name}: ${error.message}` : String(error),
      );
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => {
      bootstrapLegacyDashboard();
    }, { once: true });
  } else {
    bootstrapLegacyDashboard();
  }
})();
