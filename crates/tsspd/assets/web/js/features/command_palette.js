window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  const RECENTS_KEY = "tssp.commandPalette.recents";
  let commandItems = [];
  let selectedIndex = 0;
  let searchTimer = null;

  const QUICK_ACTIONS = [
    {
      type: "action",
      section: "Cloud Drive",
      title: "Upload files",
      detail: "Open the file picker and upload into the selected Drive folder.",
      keywords: "upload file object drive",
      run: () => T.$("#upload-input")?.click(),
    },
    {
      type: "action",
      section: "Cloud Drive",
      title: "Open Cloud Drive",
      detail: "Browse folders, object metadata, previews, and sharing controls.",
      keywords: "files drive objects folders",
      run: () => T.setView("objects"),
    },
    {
      type: "action",
      section: "Sharing",
      title: "Open Sharing Center",
      detail: "Manage public links, QR codes, downloads, and revokes.",
      keywords: "public links share qr",
      run: () => T.setView("public"),
    },
    {
      type: "action",
      section: "Notes",
      title: "Create note",
      detail: "Start a new page in the Notes workspace.",
      keywords: "new note page",
      run: () => T.openNoteDialog?.(null),
    },
    {
      type: "action",
      section: "Workspace",
      title: "Open IDE workspace",
      detail: "Open the admin workspace editor with file tree and metadata.",
      keywords: "editor ide workspace code",
      run: () => T.setView("editor"),
    },
    {
      type: "action",
      section: "Admin",
      title: "Open Admin Control Center",
      detail: "Users, sessions, devices, storage, diagnostics, and safe operations.",
      keywords: "admin operations console health users sessions",
      run: () => T.setView("admin"),
    },
  ];

  function readRecents() {
    try {
      const parsed = JSON.parse(localStorage.getItem(RECENTS_KEY) || "[]");
      return Array.isArray(parsed) ? parsed.filter(Boolean).slice(0, 5) : [];
    } catch {
      return [];
    }
  }

  function rememberQuery(query) {
    const q = query.trim();
    if (!q) return;
    const next = [q, ...readRecents().filter((item) => item.toLowerCase() !== q.toLowerCase())].slice(0, 5);
    try {
      localStorage.setItem(RECENTS_KEY, JSON.stringify(next));
    } catch {
      /* localStorage can be disabled; recents are optional. */
    }
  }

  function itemIcon(item) {
    if (item.type === "file") return T.fileKindIcon?.(item) || "OBJ";
    if (item.type === "note") return "NOTE";
    if (item.type === "workspace") return "IDE";
    return "CMD";
  }

  function itemClass(item) {
    if (item.type === "file") return T.fileKindClass?.(item) || "file-kind-object";
    if (item.type === "note") return "file-kind-note";
    if (item.type === "workspace") return "file-kind-code";
    return "file-kind-command";
  }

  function normalizeSearchResult(result) {
    const type = result.type || "item";
    return {
      type,
      section: type === "file" ? "Files" : type === "note" ? "Notes" : "Workspaces",
      title: result.title || result.name || result.id,
      detail: result.snippet || result.folder_path || result.language || result.mime_type || "",
      tags: result.tags || [],
      id: result.id,
      visibility: result.visibility,
      size_bytes: result.size_bytes,
      mime_type: result.mime_type,
      folder_path: result.folder_path,
      language: result.language,
      result,
      run: () => {
        if (type === "file") {
          T.previewFile?.(result.id);
        } else if (type === "note") {
          T.openNote?.(result.id);
        } else if (type === "workspace") {
          T.setView("workspaces");
          T.openWorkspace?.(result.id);
        }
      },
    };
  }

  function quickMatches(query) {
    const q = query.toLowerCase().trim();
    if (!q) return QUICK_ACTIONS;
    return QUICK_ACTIONS.filter((item) =>
      `${item.section} ${item.title} ${item.detail} ${item.keywords}`.toLowerCase().includes(q)
    );
  }

  function renderCommandItems(items, emptyMessage) {
    const target = T.$("#command-results");
    if (!target) return;
    commandItems = items;
    selectedIndex = Math.min(selectedIndex, Math.max(0, commandItems.length - 1));

    if (!items.length) {
      target.innerHTML = `<div class="command-empty">${T.escapeHtml(emptyMessage || "No results found.")}</div>`;
      return;
    }

    let currentSection = "";
    target.innerHTML = items.map((item, index) => {
      const section = item.section || "Results";
      const sectionHtml = section !== currentSection
        ? `<div class="command-section">${T.escapeHtml(section)}</div>`
        : "";
      currentSection = section;
      const active = index === selectedIndex ? " active" : "";
      const tags = item.tags?.length ? `<span class="command-tags">${T.tagsHtml(item.tags)}</span>` : "";
      const visibility = item.visibility ? T.stateBadge(item.visibility) : "";
      const detailParts = [
        item.detail,
        item.size_bytes != null ? T.formatBytes(item.size_bytes) : "",
        item.language || "",
      ].filter(Boolean).join(" · ");
      return `${sectionHtml}<button type="button" class="command-item${active}" data-command-index="${index}">
        <span class="file-kind-icon ${T.escapeHtml(itemClass(item))}">${T.escapeHtml(itemIcon(item))}</span>
        <span class="command-item-main">
          <strong>${T.escapeHtml(item.title || "Untitled")}</strong>
          ${detailParts ? `<small>${T.escapeHtml(detailParts)}</small>` : ""}
          ${tags || visibility ? `<span class="command-item-meta">${tags}${visibility}</span>` : ""}
        </span>
      </button>`;
    }).join("");
  }

  function renderQuickActions(query) {
    const recents = readRecents().map((recent) => ({
      type: "recent",
      section: "Recent searches",
      title: recent,
      detail: "Search again",
      run: () => {
        const input = T.$("#command-input");
        if (input) input.value = recent;
        runPaletteSearch(recent);
      },
    }));
    renderCommandItems([...quickMatches(query), ...(query ? [] : recents)], "No matching actions.");
  }

  async function runPaletteSearch(query) {
    const q = query.trim();
    if (!q) {
      renderQuickActions("");
      return;
    }
    const quick = quickMatches(q);
    renderCommandItems(quick, "Searching...");
    try {
      const data = await T.api(`/search?q=${encodeURIComponent(q)}&limit=12`);
      const results = (data.results || []).map(normalizeSearchResult);
      renderCommandItems([...quick, ...results], "No files, notes, workspaces, or actions matched.");
    } catch (error) {
      renderCommandItems(quick, error.message);
    }
  }

  function closePalette() {
    T.$("#command-palette")?.classList.add("hidden");
    document.body.classList.remove("command-open");
  }

  function executeSelected(index = selectedIndex) {
    const item = commandItems[index];
    if (!item) return;
    const query = T.$("#command-input")?.value || "";
    rememberQuery(query);
    closePalette();
    T.$("#global-search").value = "";
    item.run?.();
  }

  T.openCommandPalette = function openCommandPalette(seed = "") {
    const palette = T.$("#command-palette");
    const input = T.$("#command-input");
    if (!palette || !input) return;
    palette.classList.remove("hidden");
    document.body.classList.add("command-open");
    input.value = seed;
    selectedIndex = 0;
    renderQuickActions(seed);
    if (seed.trim()) runPaletteSearch(seed);
    requestAnimationFrame(() => {
      input.focus();
      input.select();
    });
  };

  T.bindCommandPalette = function bindCommandPalette() {
    const palette = T.$("#command-palette");
    const input = T.$("#command-input");
    if (!palette || !input || palette.dataset.bound) return;
    palette.dataset.bound = "1";

    T.$("#command-close")?.addEventListener("click", closePalette);
    palette.addEventListener("click", (event) => {
      if (event.target === palette) closePalette();
      const item = event.target.closest("[data-command-index]");
      if (item) executeSelected(Number(item.dataset.commandIndex));
    });

    input.addEventListener("input", () => {
      clearTimeout(searchTimer);
      const query = input.value;
      renderQuickActions(query);
      searchTimer = setTimeout(() => runPaletteSearch(query), T.SEARCH_DEBOUNCE_MS);
    });

    input.addEventListener("keydown", (event) => {
      if (event.key === "Escape") {
        event.preventDefault();
        closePalette();
      } else if (event.key === "ArrowDown") {
        event.preventDefault();
        selectedIndex = Math.min(commandItems.length - 1, selectedIndex + 1);
        renderCommandItems(commandItems);
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        selectedIndex = Math.max(0, selectedIndex - 1);
        renderCommandItems(commandItems);
      } else if (event.key === "Enter") {
        event.preventDefault();
        executeSelected();
      }
    });

    T.$("#global-search")?.addEventListener("focus", () => {
      T.openCommandPalette(T.$("#global-search")?.value || "");
    });
  };
})(window.Tssp);
