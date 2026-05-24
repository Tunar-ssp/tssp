window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.allNotes = [];
  T.noteAutosaveTimer = null;
  T.noteLastSavedBody = "";
  T.activeNoteTag = null;

  // ── Tag chips ────────────────────────────────────────────────────────────

  function renderTagChips(tags) {
    const list = T.$("#note-tag-chips-list");
    if (!list) return;
    list.innerHTML = tags
      .filter((tag) => !tag.startsWith("color:"))
      .map(
        (tag) =>
          `<span class="tag-chip">${T.escapeHtml(tag)}<button type="button" class="tag-chip-remove" data-remove-tag="${T.escapeHtml(tag)}" aria-label="Remove tag ${T.escapeHtml(tag)}">×</button></span>`
      )
      .join("");
  }

  function addTag(raw) {
    const tag = raw.trim().replace(/,/g, "").slice(0, 32);
    if (!tag || tag.startsWith("color:")) return false;
    const key = tag.toLowerCase();
    if (T.editingNoteTags.some((t) => t.toLowerCase() === key)) return false;
    T.editingNoteTags = [...T.editingNoteTags, tag];
    renderTagChips(T.editingNoteTags);
    scheduleTagSync();
    return true;
  }

  function removeTag(tag) {
    const key = tag.toLowerCase();
    T.editingNoteTags = T.editingNoteTags.filter((t) => t.toLowerCase() !== key);
    renderTagChips(T.editingNoteTags);
    scheduleTagSync();
  }

  let tagSyncTimer = null;
  function scheduleTagSync() {
    if (!T.editingNoteId) return;
    clearTimeout(tagSyncTimer);
    const id = T.editingNoteId;
    tagSyncTimer = setTimeout(async () => {
      if (T.editingNoteId !== id) return;
      try {
        await syncNoteTags(id, T.editingNoteTags);
      } catch {
        // silent — user can still save manually
      }
    }, 1500);
  }

  function bindTagChipsInput() {
    const input = T.$("#note-tags-input");
    const field = T.$("#note-tag-chips-field");
    if (!input) return;

    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === "," || e.key === "Tab") {
        if (e.key !== "Tab" || input.value.trim()) e.preventDefault();
        if (addTag(input.value)) input.value = "";
      } else if (e.key === "Backspace" && input.value === "" && T.editingNoteTags.length) {
        removeTag(T.editingNoteTags[T.editingNoteTags.length - 1]);
      }
    });

    input.addEventListener("blur", () => {
      if (input.value.trim()) {
        addTag(input.value);
        input.value = "";
      }
    });

    if (field) {
      field.addEventListener("click", (e) => {
        const btn = e.target.closest("[data-remove-tag]");
        if (btn) { removeTag(btn.dataset.removeTag); return; }
        input.focus();
      });
    }
  }

  // ── Note list ────────────────────────────────────────────────────────────

  T.loadNotes = async function loadNotes() {
    const grid = T.$("#notes-grid");
    if (!grid) return;
    grid.innerHTML = '<div class="notes-loading">Loading notes…</div>';
    try {
      const data = await T.api("/notes?limit=200");
      T.allNotes = data.notes || [];
      T.activeNoteTag = null;
      T.renderNoteCards();
    } catch (error) {
      grid.innerHTML = `<div class="notes-empty-state">${T.escapeHtml(error.message)}</div>`;
    }
  };

  function renderNoteTagBar() {
    const bar = T.$("#notes-tag-bar");
    if (!bar) return;
    const tagCounts = new Map();
    for (const note of T.allNotes) {
      for (const tag of note.tags || []) {
        if (tag.startsWith("color:")) continue;
        tagCounts.set(tag, (tagCounts.get(tag) || 0) + 1);
      }
    }
    if (!tagCounts.size) { bar.innerHTML = ""; return; }
    const sorted = [...tagCounts.entries()].sort((a, b) => b[1] - a[1]);
    bar.innerHTML = sorted
      .map(([tag, count]) => {
        const active = T.activeNoteTag === tag;
        return `<button type="button" class="note-tag-filter-chip${active ? " active" : ""}" data-tag-filter="${T.escapeHtml(tag)}">${T.escapeHtml(tag)} <span class="note-tag-filter-count">${count}</span></button>`;
      })
      .join("");
  }

  T.renderNoteCards = function renderNoteCards() {
    const grid = T.$("#notes-grid");
    if (!grid) return;

    renderNoteTagBar();

    const query = (T.$("#notes-local-search")?.value || "").toLowerCase().trim();
    const pinnedOnly = T.$("#notes-pinned-filter")?.checked;

    let notes = T.allNotes.slice();

    if (pinnedOnly) notes = notes.filter((n) => n.pinned_at != null);
    if (T.activeNoteTag) notes = notes.filter((n) => (n.tags || []).some((t) => t === T.activeNoteTag));
    if (query) notes = notes.filter((n) =>
      (n.title || "").toLowerCase().includes(query) || (n.body || "").toLowerCase().includes(query)
    );

    if (!notes.length) {
      if (T.allNotes.length === 0) {
        grid.innerHTML = `<div class="notes-empty-hero">
          <div class="notes-empty-icon">📝</div>
          <div class="notes-empty-title">No notes yet</div>
          <div class="notes-empty-sub">Write down ideas, docs, and quick thoughts. Use <kbd>/</kbd> for formatting commands.</div>
          <button type="button" class="btn btn-primary" id="notes-empty-cta">New note</button>
        </div>`;
        grid.querySelector("#notes-empty-cta")?.addEventListener("click", () => T.openNoteDialog(null));
      } else {
        grid.innerHTML = `<div class="notes-empty-state">No notes match your filters.</div>`;
      }
      return;
    }

    const pinned = notes.filter((n) => n.pinned_at != null);
    const unpinned = notes.filter((n) => n.pinned_at == null);
    const parts = [];

    if (pinned.length) {
      parts.push('<div class="notes-section-label">Pinned</div>');
      parts.push('<div class="notes-cards-row">');
      parts.push(...pinned.map(noteCard));
      parts.push('</div>');
      if (unpinned.length) {
        parts.push('<div class="notes-section-label">Other notes</div>');
      }
    }
    if (unpinned.length) {
      parts.push('<div class="notes-cards-row">');
      parts.push(...unpinned.map(noteCard));
      parts.push('</div>');
    }

    grid.innerHTML = parts.join("");
    T.updateNotesCount(notes.length);
  };

  const NOTE_COLORS = ["red", "orange", "yellow", "green", "blue", "purple", "pink", "gray"];

  function noteColor(note) {
    const tag = (note.tags || []).find((t) => t.startsWith("color:"));
    return tag ? tag.slice(6) : null;
  }

  function visibleTags(note) {
    return (note.tags || []).filter((t) => !t.startsWith("color:"));
  }

  function noteCard(note) {
    const id = T.escapeHtml(note.id);
    const pinned = note.pinned_at != null;
    const color = noteColor(note);
    const preview = (note.body || "")
      .trim()
      .replace(/^#+\s+/gm, "")
      .replace(/\*\*([^*]+)\*\*/g, "$1")
      .replace(/\*([^*]+)\*/g, "$1")
      .replace(/`([^`]+)`/g, "$1")
      .replace(/~~([^~]+)~~/g, "$1")
      .replace(/^\s*[-*]\s+/gm, "")
      .replace(/^\s*\d+\.\s+/gm, "")
      .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
      .replace(/\n+/g, " ")
      .trim()
      .slice(0, 180);
    const tags = T.tagsHtml(visibleTags(note));
    const dateStr = T.escapeHtml(T.formatDate(note.updated_at));
    const wordCount = (note.body || "").trim().split(/\s+/).filter(Boolean).length;
    const colorClass = color ? ` note-card-color-${T.escapeHtml(color)}` : "";
    const relTime = T.escapeHtml(T.formatRelativeTime(note.updated_at));
    return `<article class="note-card${pinned ? " note-card-pinned" : ""}${colorClass}" role="button" tabindex="0" data-edit-note="${id}" aria-label="Open note ${T.escapeHtml(note.title || "Untitled")}">
      <div class="note-card-header">
        <strong class="note-card-title">${T.escapeHtml(note.title || "Untitled")}</strong>
        ${pinned ? '<span class="note-pin-star" title="Pinned">★</span>' : ""}
      </div>
      <p class="note-card-preview">${T.escapeHtml(preview || "(empty note)")}</p>
      <div class="note-card-footer">
        <div class="note-card-tags">${tags || ""}</div>
        <div class="note-card-meta">
          <span>${wordCount} words</span>
          <span title="${dateStr}">${relTime || dateStr}</span>
        </div>
      </div>
	      <div class="note-card-actions">
	        <button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Edit</button>
	        <button type="button" class="btn btn-text btn-sm" data-duplicate-note="${id}">Duplicate</button>
	        <button type="button" class="btn btn-text btn-sm" data-pin-note="${id}" data-pinned="${pinned ? "1" : "0"}">${pinned ? "Unpin" : "Pin"}</button>
	        <button type="button" class="btn btn-text btn-sm btn-danger" data-delete-note="${id}">Delete</button>
	      </div>
    </article>`;
  }

  T.updateNotesCount = function updateNotesCount(count) {
    const el = T.$("#notes-count");
    if (el) el.textContent = count === T.allNotes.length
      ? `${count} notes`
      : `${count} of ${T.allNotes.length} notes`;
  };

  // ── Note editor ──────────────────────────────────────────────────────────

	  T.refreshNotePreview = function refreshNotePreview() {
	    const preview = T.$("#note-preview");
	    if (preview) preview.innerHTML = T.simpleMarkdown(T.$("#note-body-input")?.value || "");
	    renderNoteOutline();
	  };

	  function renderNoteOutline() {
	    const outline = T.$("#note-outline-list");
	    const body = T.$("#note-body-input")?.value || "";
	    if (!outline) return;
	    const headings = body
	      .split(/\r?\n/)
	      .map((line, index) => ({ line, index }))
	      .filter((item) => /^#{1,3}\s+/.test(item.line))
	      .map((item) => ({
	        level: item.line.match(/^#+/)[0].length,
	        title: item.line.replace(/^#{1,3}\s+/, "").trim(),
	        line: item.index + 1,
	      }));
	    if (!headings.length) {
	      const blockCount = body.split(/\n{2,}/).filter((block) => block.trim()).length;
	      outline.innerHTML = `<div class="note-outline-empty">
	        <strong>${blockCount}</strong>
	        <span>blocks detected</span>
	        <small>Add headings to build a page outline.</small>
	      </div>`;
	      return;
	    }
	    outline.innerHTML = headings
	      .map((heading) => `<button type="button" class="note-outline-item level-${heading.level}" data-note-goto-line="${heading.line}">
	        <span>${T.escapeHtml(heading.title || "Untitled section")}</span>
	        <small>L${heading.line}</small>
	      </button>`)
	      .join("");
	  }

	  function insertAtCursor(text) {
	    const area = T.$("#note-body-input");
	    if (!area || !text) return;
	    const start = area.selectionStart;
	    const end = area.selectionEnd;
	    const before = area.value.slice(0, start);
	    const after = area.value.slice(end);
	    const prefix = before && !before.endsWith("\n") ? "\n" : "";
	    const suffix = after && !text.endsWith("\n") ? "\n" : "";
	    area.value = before + prefix + text + suffix + after;
	    const cursor = before.length + prefix.length + text.length;
	    area.focus();
	    area.selectionStart = area.selectionEnd = cursor;
	    T.refreshNotePreview();
	    T.updateNoteWordCount();
	    scheduleAutosave();
	  }

	  T.insertNoteBlock = function insertNoteBlock(kind) {
	    const blocks = {
	      heading: "## Section title\n\nWrite the idea here.",
	      todo: "- [ ] First task\n- [ ] Follow-up task\n",
	      bullet: "- Key point\n- Supporting detail\n",
	      numbered: "1. First step\n2. Next step\n",
	      callout: "> [!NOTE]\n> Important context or decision.\n",
	      code: "```text\npaste code or command output here\n```\n",
	      table: "| Item | Status | Notes |\n| --- | --- | --- |\n| Example | Open | Add details |\n",
	    };
	    insertAtCursor(blocks[kind] || "");
	  };

	  T.openNoteTemplate = function openNoteTemplate() {
	    T.openNoteDialog(null);
	    T.$("#note-title-input").value = "Untitled project note";
	    T.$("#note-body-input").value =
	      "# Project note\n\n> [!NOTE]\n> Context, goal, or decision.\n\n## Tasks\n\n- [ ] First task\n- [ ] Follow-up task\n\n## Notes\n\nWrite details here.\n";
	    T.refreshNotePreview();
	    T.updateNoteWordCount();
	  };

  function scheduleAutosave() {
    clearTimeout(T.noteAutosaveTimer);
    const body = T.$("#note-body-input")?.value || "";
    if (!T.editingNoteId) return;
    T.noteAutosaveTimer = setTimeout(() => {
      const currentBody = T.$("#note-body-input")?.value || "";
      if (currentBody !== T.noteLastSavedBody) {
        T.autoSaveNote();
      }
    }, 2000);
  }

  T.autoSaveNote = async function autoSaveNote() {
    if (!T.editingNoteId) return;
    const body = T.$("#note-body-input")?.value || "";
    const title = T.$("#note-title-input")?.value.trim();
    const status = T.$("#note-save-status");
    if (status) { status.textContent = "Autosaving…"; status.className = "save-status saving"; }
    try {
      await T.api("/notes/" + encodeURIComponent(T.editingNoteId), {
        method: "PUT",
        body: JSON.stringify({ body, ...(title ? { title } : {}) }),
      });
      T.noteLastSavedBody = body;
      if (status) { status.textContent = "Saved"; status.className = "save-status saved"; }
    } catch {
      if (status) { status.textContent = "Save failed"; status.className = "save-status dirty"; }
    }
  };

  function renderColorPicker(tags) {
    const current = (tags || []).find((t) => t.startsWith("color:"));
    const currentColor = current ? current.slice(6) : "";
    T.$("#note-color-picker")?.querySelectorAll(".note-color-btn").forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.color === currentColor);
    });
  }

  function bindColorPicker() {
    const picker = T.$("#note-color-picker");
    if (!picker || picker.dataset.bound) return;
    picker.dataset.bound = "1";
    picker.addEventListener("click", (e) => {
      const btn = e.target.closest(".note-color-btn");
      if (!btn) return;
      const color = btn.dataset.color;
      // Remove any existing color tag
      T.editingNoteTags = T.editingNoteTags.filter((t) => !t.startsWith("color:"));
      if (color) T.editingNoteTags.push(`color:${color}`);
      renderTagChips(T.editingNoteTags);
      renderColorPicker(T.editingNoteTags);
      scheduleTagSync();
    });
  }

  T.openNoteDialog = function openNoteDialog(note) {
    T.editingNoteId = note ? note.id : null;
    T.editingNoteTags = note ? note.tags || [] : [];
    T.editingNotePinned = note ? note.pinned_at != null : false;
    T.noteLastSavedBody = note ? note.body || "" : "";
    T.$("#note-dialog-title").textContent = note ? "Edit note" : "New note";
    T.$("#note-title-input").value = note ? note.title || "" : "";
    T.$("#note-tags-input").value = "";
    renderTagChips(T.editingNoteTags);
    renderColorPicker(T.editingNoteTags);
    bindColorPicker();
    T.$("#note-pin-input").checked = T.editingNotePinned;
    T.$("#note-body-input").value = note ? note.body || "" : "";
    T.$("#note-save-status").textContent = "";
    T.$("#note-save-status").className = "save-status";
    T.refreshNotePreview();
    T.updateNoteWordCount();
    T.setView("note-editor");
    T.$("#note-title-input").focus();
  };

  T.updateNoteWordCount = function updateNoteWordCount() {
    const el = T.$("#note-word-count");
    if (!el) return;
    const text = T.$("#note-body-input")?.value || "";
    const words = text.trim().split(/\s+/).filter(Boolean).length;
    const chars = text.length;
    el.textContent = `${words} words · ${chars} chars`;
  };

  T.closeNoteEditor = function closeNoteEditor(force) {
    clearTimeout(T.noteAutosaveTimer);
    clearTimeout(tagSyncTimer);
    // For new unsaved notes, prompt before discarding
    if (!force && !T.editingNoteId) {
      const body = T.$("#note-body-input")?.value || "";
      const title = T.$("#note-title-input")?.value || "";
      if ((body.trim() || title.trim()) && !confirm("Discard this new note?")) return;
    }
    T.setView("notes");
  };

  // ── Slash command menu ───────────────────────────────────────────────────

  const SLASH_COMMANDS = [
    { icon: "H1", label: "Heading 1",       key: "h1",       snippet: "# ",                   desc: "Large section title" },
    { icon: "H2", label: "Heading 2",       key: "h2",       snippet: "## ",                  desc: "Medium section title" },
    { icon: "H3", label: "Heading 3",       key: "h3",       snippet: "### ",                 desc: "Subsection title" },
    { icon: "•",  label: "Bullet list",     key: "bullet",   snippet: "- ",                   desc: "Unordered list" },
    { icon: "1.",  label: "Numbered list",  key: "numbered", snippet: "1. ",                  desc: "Ordered list" },
    { icon: "☑",  label: "Checklist",       key: "todo",     snippet: "- [ ] ",              desc: "Task item" },
    { icon: ">",  label: "Quote / Callout", key: "callout",  snippet: "> [!NOTE]\n> ",        desc: "Callout block" },
    { icon: "</>", label: "Code block",     key: "code",     snippet: "```\n\n```",            desc: "Code or terminal output" },
    { icon: "—",  label: "Divider",         key: "hr",       snippet: "\n---\n",              desc: "Horizontal rule" },
    { icon: "⊞",  label: "Table",           key: "table",    snippet: "| Column | Column |\n| --- | --- |\n| Cell | Cell |\n", desc: "Markdown table" },
    { icon: "↗",  label: "Link",            key: "link",     snippet: "[link text](url)",     desc: "Hyperlink" },
    { icon: "``", label: "Inline code",     key: "icode",    snippet: "``",                   desc: "Inline code span" },
    { icon: "**", label: "Bold",            key: "bold",     snippet: "****",                 desc: "Bold text" },
    { icon: "_",  label: "Italic",          key: "italic",   snippet: "__",                   desc: "Italic text" },
  ];

  let slashMenu = null;
  let slashStart = -1;
  let slashItems = [];
  let slashSelected = 0;

  function showSlashMenu(area, query) {
    closeSlashMenu();
    slashItems = query
      ? SLASH_COMMANDS.filter((c) =>
          c.label.toLowerCase().includes(query.toLowerCase()) ||
          c.key.includes(query.toLowerCase()))
      : SLASH_COMMANDS;
    if (!slashItems.length) return;

    slashSelected = 0;
    const rect = area.getBoundingClientRect();
    // approximate position: use cursor line
    const textBefore = area.value.slice(0, area.selectionStart);
    const lines = textBefore.split("\n");
    const lineIdx = lines.length - 1;
    const lineHeight = parseFloat(getComputedStyle(area).lineHeight) || 22;
    const paddingTop = parseFloat(getComputedStyle(area).paddingTop) || 8;
    const top = rect.top + window.scrollY + paddingTop + lineIdx * lineHeight + lineHeight;
    const left = rect.left + window.scrollX + 12;

    slashMenu = document.createElement("div");
    slashMenu.className = "slash-menu";
    slashMenu.style.cssText = `position:fixed;left:${Math.min(left, window.innerWidth - 280)}px;top:${Math.min(top, window.innerHeight - 320)}px;z-index:300;`;
    renderSlashMenu(area);
    document.body.appendChild(slashMenu);
  }

  function renderSlashMenu(area) {
    if (!slashMenu) return;
    slashMenu.innerHTML = `<div class="slash-menu-list">${
      slashItems.map((cmd, i) =>
        `<button type="button" class="slash-menu-item${i === slashSelected ? " active" : ""}" data-slash-key="${T.escapeHtml(cmd.key)}">
          <span class="slash-icon">${T.escapeHtml(cmd.icon)}</span>
          <span class="slash-main">
            <span class="slash-label">${T.escapeHtml(cmd.label)}</span>
            <span class="slash-desc">${T.escapeHtml(cmd.desc)}</span>
          </span>
        </button>`
      ).join("")
    }</div>`;

    slashMenu.querySelectorAll(".slash-menu-item").forEach((btn, i) => {
      btn.addEventListener("mouseenter", () => { slashSelected = i; renderSlashMenu(area); });
      btn.addEventListener("click", () => applySlashCommand(area, btn.dataset.slashKey));
    });
  }

  function applySlashCommand(area, key) {
    const cmd = SLASH_COMMANDS.find((c) => c.key === key);
    if (!cmd || slashStart < 0) { closeSlashMenu(); return; }
    // Replace from slashStart to current cursor with the snippet
    const before = area.value.slice(0, slashStart);
    const after = area.value.slice(area.selectionStart);
    let snippet = cmd.snippet;
    // Place cursor at end of snippet (or inside paired chars)
    let cursorOffset = snippet.length;
    if (cmd.key === "icode") cursorOffset = 1;
    if (cmd.key === "bold") cursorOffset = 2;
    if (cmd.key === "italic") cursorOffset = 1;
    if (cmd.key === "code") cursorOffset = "```\n".length;
    area.value = before + snippet + after;
    area.selectionStart = area.selectionEnd = before.length + cursorOffset;
    area.focus();
    closeSlashMenu();
    T.refreshNotePreview();
    T.updateNoteWordCount();
    scheduleAutosave();
  }

  function closeSlashMenu() {
    if (slashMenu) { slashMenu.remove(); slashMenu = null; }
    slashStart = -1;
  }

  T.bindNoteEditorEvents = function bindNoteEditorEvents() {
    const bodyInput = T.$("#note-body-input");
    const titleInput = T.$("#note-title-input");
    if (bodyInput) {
      bodyInput.addEventListener("input", (e) => {
        T.refreshNotePreview();
        T.updateNoteWordCount();
        scheduleAutosave();

        // Slash command detection
        const val = bodyInput.value;
        const pos = bodyInput.selectionStart;
        const textBefore = val.slice(0, pos);
        const lineStart = textBefore.lastIndexOf("\n") + 1;
        const lineText = textBefore.slice(lineStart);
        if (lineText.startsWith("/")) {
          slashStart = lineStart;
          showSlashMenu(bodyInput, lineText.slice(1));
        } else {
          closeSlashMenu();
        }
      });

      bodyInput.addEventListener("keydown", (e) => {
        if (!slashMenu) return;
        if (e.key === "ArrowDown") {
          e.preventDefault();
          slashSelected = (slashSelected + 1) % slashItems.length;
          renderSlashMenu(bodyInput);
        } else if (e.key === "ArrowUp") {
          e.preventDefault();
          slashSelected = (slashSelected - 1 + slashItems.length) % slashItems.length;
          renderSlashMenu(bodyInput);
        } else if (e.key === "Enter") {
          e.preventDefault();
          applySlashCommand(bodyInput, slashItems[slashSelected]?.key);
        } else if (e.key === "Escape") {
          closeSlashMenu();
        }
      });

      bodyInput.addEventListener("blur", () => {
        // Delay to allow click on slash menu item
        setTimeout(closeSlashMenu, 180);
      });
    }
    if (titleInput) {
      titleInput.addEventListener("input", scheduleAutosave);
    }
    const pinInput = T.$("#note-pin-input");
    if (pinInput) {
      pinInput.addEventListener("change", async () => {
        if (!T.editingNoteId) return;
        try {
          await syncNotePin(T.editingNoteId, pinInput.checked);
          T.editingNotePinned = pinInput.checked;
        } catch (e) {
          T.showBanner(e.message, "error");
          pinInput.checked = T.editingNotePinned;
        }
      });
    }
	    const noteSearch = T.$("#notes-local-search");
	    const notePinnedFilter = T.$("#notes-pinned-filter");
	    if (noteSearch) noteSearch.addEventListener("input", () => T.renderNoteCards());
	    if (notePinnedFilter) notePinnedFilter.addEventListener("change", () => T.renderNoteCards());
	    T.$("#notes-template-btn")?.addEventListener("click", () => T.openNoteTemplate());
	    T.$$(".note-block-toolbar [data-note-insert]").forEach((button) => {
	      if (button.dataset.bound) return;
	      button.dataset.bound = "1";
	      button.addEventListener("click", () => T.insertNoteBlock(button.dataset.noteInsert));
	    });

	    const noteTagBar = T.$("#notes-tag-bar");
    if (noteTagBar) {
      noteTagBar.addEventListener("click", (e) => {
        const btn = e.target.closest("[data-tag-filter]");
        if (!btn) return;
        const tag = btn.dataset.tagFilter;
        T.activeNoteTag = T.activeNoteTag === tag ? null : tag;
        T.renderNoteCards();
      });
    }

	    bindTagChipsInput();
	    T.$("#note-outline-list")?.addEventListener("click", (e) => {
	      const button = e.target.closest("[data-note-goto-line]");
	      if (!button) return;
	      const area = T.$("#note-body-input");
	      if (!area) return;
	      const targetLine = Number(button.dataset.noteGotoLine);
	      const offset = area.value
	        .split("\n")
	        .slice(0, Math.max(0, targetLine - 1))
	        .join("\n").length;
	      area.focus();
	      area.selectionStart = area.selectionEnd = targetLine > 1 ? offset + 1 : 0;
	    });
	  };

  // ── Tag sync helpers ─────────────────────────────────────────────────────

  async function syncNoteTags(id, desiredTags) {
    const current = new Set(T.editingNoteTags.map((tag) => tag.toLocaleLowerCase()));
    const desired = new Set(desiredTags.map((tag) => tag.toLocaleLowerCase()));
    const toAdd = desiredTags.filter((tag) => !current.has(tag.toLocaleLowerCase()));
    const toRemove = T.editingNoteTags.filter((tag) => !desired.has(tag.toLocaleLowerCase()));
    if (toAdd.length) {
      await T.api("/notes/" + encodeURIComponent(id) + "/tags", {
        method: "POST",
        body: JSON.stringify(toAdd),
      });
    }
    for (const tag of toRemove) {
      await T.api(
        "/notes/" + encodeURIComponent(id) + "/tags/" + encodeURIComponent(tag),
        { method: "DELETE" }
      );
    }
  }

  async function syncNotePin(id, desired) {
    if (desired === T.editingNotePinned) return;
    await T.api("/notes/" + encodeURIComponent(id) + "/pin", {
      method: desired ? "PUT" : "DELETE",
    });
  }

  // ── Save ─────────────────────────────────────────────────────────────────

  T.saveNote = async function saveNote() {
    clearTimeout(T.noteAutosaveTimer);
    const title = T.$("#note-title-input").value.trim();
    const body = T.$("#note-body-input").value;
    // flush any pending text in the chips input
    const rawInput = T.$("#note-tags-input");
    if (rawInput && rawInput.value.trim()) { addTag(rawInput.value); rawInput.value = ""; }
    const tags = [...T.editingNoteTags];
    const pin = T.$("#note-pin-input").checked;
    const payload = { body };
    if (title) payload.title = title;
    const status = T.$("#note-save-status");
    if (status) { status.textContent = "Saving…"; status.className = "save-status saving"; }
    try {
      let saved;
      if (T.editingNoteId) {
        saved = await T.api("/notes/" + encodeURIComponent(T.editingNoteId), {
          method: "PUT",
          body: JSON.stringify(payload),
        });
        await syncNoteTags(T.editingNoteId, tags);
        await syncNotePin(T.editingNoteId, pin);
      } else {
        saved = await T.api("/notes", {
          method: "POST",
          body: JSON.stringify({ ...payload, tags, pin }),
        });
      }
      T.noteLastSavedBody = body;
      if (status) { status.textContent = "Saved"; status.className = "save-status saved"; }
      T.showBanner("Note saved", "success");
      T.closeNoteEditor();
      T.loadNotes();
      return saved;
    } catch (error) {
      if (status) { status.textContent = ""; status.className = "save-status"; }
      T.showBanner(error.message, "error");
      return null;
    }
  };

  T.openNote = async function openNote(id) {
    try {
      const note = await T.api("/notes/" + encodeURIComponent(id));
      T.openNoteDialog(note);
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };

	  T.toggleNotePin = async function toggleNotePin(id, pinned) {
	    try {
	      await T.api("/notes/" + encodeURIComponent(id) + "/pin", {
	        method: pinned ? "DELETE" : "PUT",
	      });
	      T.loadNotes();
	    } catch (error) {
	      T.showBanner(error.message, "error");
	    }
	  };

	  T.duplicateNote = async function duplicateNote(id) {
	    try {
	      const note = await T.api("/notes/" + encodeURIComponent(id));
	      await T.api("/notes", {
	        method: "POST",
	        body: JSON.stringify({
	          title: `${note.title || "Untitled"} copy`,
	          body: note.body || "",
	          tags: note.tags || [],
	          pin: false,
	        }),
	      });
	      T.showBanner("Note duplicated", "success");
	      T.loadNotes();
	    } catch (error) {
	      T.showBanner(error.message, "error");
	    }
	  };

	  T.deleteNote = async function deleteNote(id) {
    if (!confirm("Delete this note?")) return;
    try {
      await T.api("/notes/" + encodeURIComponent(id), { method: "DELETE" });
      T.showBanner("Note deleted", "success");
      T.allNotes = T.allNotes.filter((n) => n.id !== id);
      T.renderNoteCards();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };
})(window.Tssp);
