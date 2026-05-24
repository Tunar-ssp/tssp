window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.allNotes = [];
  T.noteAutosaveTimer = null;
  T.noteLastSavedBody = "";

  // ── Note list ────────────────────────────────────────────────────────────

  T.loadNotes = async function loadNotes() {
    const grid = T.$("#notes-grid");
    if (!grid) return;
    grid.innerHTML = '<div class="notes-loading">Loading notes…</div>';
    try {
      const data = await T.api("/notes?limit=200");
      T.allNotes = data.notes || [];
      T.renderNoteCards();
    } catch (error) {
      grid.innerHTML = `<div class="notes-empty-state">${T.escapeHtml(error.message)}</div>`;
    }
  };

  T.renderNoteCards = function renderNoteCards() {
    const grid = T.$("#notes-grid");
    if (!grid) return;

    const query = (T.$("#notes-local-search")?.value || "").toLowerCase().trim();
    const tagFilter = (T.$("#notes-tag-filter")?.value || "").trim().toLowerCase();
    const pinnedOnly = T.$("#notes-pinned-filter")?.checked;

    let notes = T.allNotes.slice();

    if (pinnedOnly) notes = notes.filter((n) => n.pinned_at != null);
    if (tagFilter) notes = notes.filter((n) => (n.tags || []).some((t) => t.toLowerCase().includes(tagFilter)));
    if (query) notes = notes.filter((n) =>
      (n.title || "").toLowerCase().includes(query) || (n.body || "").toLowerCase().includes(query)
    );

    if (!notes.length) {
      const emptyMsg = T.allNotes.length === 0
        ? "No notes yet. Create your first note to get started."
        : "No notes match your filters.";
      grid.innerHTML = `<div class="notes-empty-state">${T.escapeHtml(emptyMsg)}</div>`;
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

  function noteCard(note) {
    const id = T.escapeHtml(note.id);
    const pinned = note.pinned_at != null;
    const preview = (note.body || "").trim().replace(/^#+\s+/gm, "").slice(0, 180);
    const tags = T.tagsHtml(note.tags);
    const dateStr = T.escapeHtml(T.formatDate(note.updated_at));
    const wordCount = (note.body || "").trim().split(/\s+/).filter(Boolean).length;
    return `<article class="note-card${pinned ? " note-card-pinned" : ""}" role="button" tabindex="0" data-edit-note="${id}" aria-label="Open note ${T.escapeHtml(note.title || "Untitled")}">
      <div class="note-card-header">
        <strong class="note-card-title">${T.escapeHtml(note.title || "Untitled")}</strong>
        ${pinned ? '<span class="note-pin-star" title="Pinned">★</span>' : ""}
      </div>
      <p class="note-card-preview">${T.escapeHtml(preview || "(empty note)")}</p>
      <div class="note-card-footer">
        <div class="note-card-tags">${tags || ""}</div>
        <div class="note-card-meta">
          <span>${wordCount} words</span>
          <span>${dateStr}</span>
        </div>
      </div>
      <div class="note-card-actions">
        <button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Edit</button>
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

  T.openNoteDialog = function openNoteDialog(note) {
    T.editingNoteId = note ? note.id : null;
    T.editingNoteTags = note ? note.tags || [] : [];
    T.editingNotePinned = note ? note.pinned_at != null : false;
    T.noteLastSavedBody = note ? note.body || "" : "";
    T.$("#note-dialog-title").textContent = note ? "Edit note" : "New note";
    T.$("#note-title-input").value = note ? note.title || "" : "";
    T.$("#note-tags-input").value = T.editingNoteTags.join(", ");
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

  T.closeNoteEditor = function closeNoteEditor() {
    clearTimeout(T.noteAutosaveTimer);
    T.setView("notes");
  };

  T.bindNoteEditorEvents = function bindNoteEditorEvents() {
    const bodyInput = T.$("#note-body-input");
    const titleInput = T.$("#note-title-input");
    if (bodyInput) {
      bodyInput.addEventListener("input", () => {
        T.refreshNotePreview();
        T.updateNoteWordCount();
        scheduleAutosave();
      });
    }
    if (titleInput) {
      titleInput.addEventListener("input", scheduleAutosave);
    }
    const noteSearch = T.$("#notes-local-search");
    const noteTagFilter = T.$("#notes-tag-filter");
    const notePinnedFilter = T.$("#notes-pinned-filter");
    if (noteSearch) noteSearch.addEventListener("input", () => T.renderNoteCards());
    if (noteTagFilter) noteTagFilter.addEventListener("input", () => T.renderNoteCards());
    if (notePinnedFilter) notePinnedFilter.addEventListener("change", () => T.renderNoteCards());
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
    const tags = T.tagsFromInput(T.$("#note-tags-input").value);
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
