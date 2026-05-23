window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.loadNotes = async function loadNotes() {
    const body = T.$("#notes-body");
    body.innerHTML = T.tableMessage(5, "Loading notes…");
    try {
      const data = await T.api("/notes?limit=200");
      const notes = data.notes || [];
      if (!notes.length) {
        body.innerHTML = T.tableMessage(5, "No notes yet. Create a markdown note to start.");
        return;
      }
      body.innerHTML = notes
        .map((note) => {
          const id = T.escapeHtml(note.id);
          const pinned = note.pinned_at != null ? '<span class="pin">★</span>' : "";
          return `<tr>
            <td><div class="file-name">${pinned}<strong>${T.escapeHtml(note.title || "Untitled")}</strong></div><div class="row-meta">${T.escapeHtml(T.formatDate(note.created_at))}</div></td>
            <td>${T.escapeHtml(T.formatDate(note.updated_at))}</td>
            <td>${T.tagsHtml(note.tags) || "—"}</td>
            <td class="muted">${T.escapeHtml((note.body || "").slice(0, 120))}</td>
            <td class="col-actions">
              <button type="button" class="btn btn-text btn-sm" data-edit-note="${id}">Edit</button>
              <button type="button" class="btn btn-text btn-sm" data-pin-note="${id}" data-pinned="${note.pinned_at != null ? "1" : "0"}">${note.pinned_at != null ? "Unpin" : "Pin"}</button>
              <button type="button" class="btn btn-text btn-sm btn-danger" data-delete-note="${id}">Delete</button>
            </td>
          </tr>`;
        })
        .join("");
    } catch (error) {
      body.innerHTML = T.tableMessage(5, error.message);
    }
  };

  T.refreshNotePreview = function refreshNotePreview() {
    const preview = T.$("#note-preview");
    if (preview) preview.innerHTML = T.simpleMarkdown(T.$("#note-body-input").value);
  };

  T.openNoteDialog = function openNoteDialog(note) {
    T.editingNoteId = note ? note.id : null;
    T.editingNoteTags = note ? note.tags || [] : [];
    T.editingNotePinned = note ? note.pinned_at != null : false;
    T.$("#note-dialog-title").textContent = note ? "Edit note" : "New note";
    T.$("#note-title-input").value = note ? note.title || "" : "";
    T.$("#note-tags-input").value = T.editingNoteTags.join(", ");
    T.$("#note-pin-input").checked = T.editingNotePinned;
    T.$("#note-body-input").value = note ? note.body || "" : "";
    T.$("#note-save-status").textContent = "";
    T.refreshNotePreview();
    T.setView("note-editor");
    T.$("#note-title-input").focus();
  };

  T.closeNoteEditor = function closeNoteEditor() {
    T.setView("notes");
  };

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

  T.saveNote = async function saveNote() {
    const title = T.$("#note-title-input").value.trim();
    const body = T.$("#note-body-input").value;
    const tags = T.tagsFromInput(T.$("#note-tags-input").value);
    const pin = T.$("#note-pin-input").checked;
    const payload = { body };
    if (title) payload.title = title;
    T.$("#note-save-status").textContent = "Saving…";
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
      T.$("#note-save-status").textContent = "Saved";
      T.showBanner("Note saved", "success");
      T.closeNoteEditor();
      T.loadNotes();
      return saved;
    } catch (error) {
      T.$("#note-save-status").textContent = "";
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
      T.loadNotes();
    } catch (error) {
      T.showBanner(error.message, "error");
    }
  };
})(window.Tssp);
