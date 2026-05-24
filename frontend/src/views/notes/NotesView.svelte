<script lang="ts">
  import { onMount } from "svelte";
  import {
    createNote,
    deleteNote,
    duplicateNote,
    listNotes,
    updateNote,
    type NoteRecord,
  } from "../../lib/api";
  import { formatRelativeDate } from "../../lib/utils/format";
  import NotesEditor from "./NotesEditor.svelte";
  import NotesList from "./NotesList.svelte";

  let notes: NoteRecord[] = [];
  let loading = true;
  let error = "";
  let activeNoteId: string | null = null;
  let titleDraft = "";
  let bodyDraft = "";
  let autosaveStatus = "idle";
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  function selectNote(note: NoteRecord) {
    activeNoteId = note.id;
    titleDraft = note.title || "";
    bodyDraft = note.body || "";
    autosaveStatus = `Editing ${formatRelativeDate(note.updated_at)}`;
  }

  function activeNote() {
    return notes.find((note) => note.id === activeNoteId) || null;
  }

  function scheduleAutosave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      void saveNote();
    }, 1200);
  }

  async function loadNotes() {
    loading = true;
    error = "";
    try {
      const response = await listNotes({ limit: 50, sort: "-updated" });
      notes = response.notes || [];
      if (!activeNoteId && notes[0]) {
        selectNote(notes[0]);
      }
    } catch (nextError) {
      error = nextError instanceof Error ? nextError.message : "Failed to load notes";
    } finally {
      loading = false;
    }
  }

  async function saveNote() {
    const note = activeNote();
    if (!note) return;
    try {
      const updated = await updateNote(note.id, {
        title: titleDraft.trim() || "Untitled note",
        body: bodyDraft,
      });
      notes = notes.map((item) => (item.id === updated.id ? updated : item));
      selectNote(updated);
      autosaveStatus = `Saved ${formatRelativeDate(updated.updated_at)}`;
    } catch (nextError) {
      autosaveStatus = nextError instanceof Error ? nextError.message : "Save failed";
    }
  }

  async function createNewNote() {
    try {
      const created = await createNote({
        title: "Untitled note",
        body: "# Quick capture\n\nStart writing here.",
        tags: ["draft"],
        pin: false,
      });
      notes = [created, ...notes];
      selectNote(created);
      autosaveStatus = "New note created";
    } catch (nextError) {
      autosaveStatus = nextError instanceof Error ? nextError.message : "Create failed";
    }
  }

  async function duplicateActive() {
    const note = activeNote();
    if (!note) return;
    try {
      const duplicate = await duplicateNote(note.id);
      notes = [duplicate, ...notes];
      selectNote(duplicate);
      autosaveStatus = "Duplicated note";
    } catch (nextError) {
      autosaveStatus = nextError instanceof Error ? nextError.message : "Duplicate failed";
    }
  }

  async function deleteActive() {
    const note = activeNote();
    if (!note) return;
    try {
      await deleteNote(note.id);
      notes = notes.filter((item) => item.id !== note.id);
      const next = notes[0] || null;
      if (next) {
        selectNote(next);
      } else {
        activeNoteId = null;
        titleDraft = "";
        bodyDraft = "";
      }
      autosaveStatus = "Note deleted";
    } catch (nextError) {
      autosaveStatus = nextError instanceof Error ? nextError.message : "Delete failed";
    }
  }

  onMount(() => {
    void loadNotes();
  });
</script>

<section class="view-grid notes-layout">
  <div class="hero-card compact">
    <div>
      <div class="eyebrow">Knowledge</div>
      <h1>Structured note pages, not a random textarea.</h1>
      <p>
        Notes now behave like pages with a list, a focused editor, markdown preview, autosave, and
        duplicate/delete actions.
      </p>
    </div>
    <div class="hero-actions">
      <button class="btn btn-primary" type="button" on:click={createNewNote}>New note</button>
      <button class="btn btn-secondary" type="button" on:click={loadNotes}>Refresh</button>
    </div>
  </div>

  <div class="split-view notes-shell">
    <article class="panel-card">
      <header class="panel-head">
        <strong>Notes home</strong>
        <span>cards, tags, recents, pinned</span>
      </header>
      {#if loading}
        <div class="empty-copy">Loading notes...</div>
      {:else if error}
        <div class="empty-copy">{error}</div>
      {:else if notes.length === 0}
        <div class="empty-copy">No notes yet. Create a page to start the workspace.</div>
      {:else}
        <NotesList notes={notes.slice(0, 8)} activeId={activeNoteId} onSelectNote={selectNote} />
      {/if}
    </article>

    <NotesEditor
      note={activeNote()}
      titleDraft={titleDraft}
      bodyDraft={bodyDraft}
      autosaveStatus={autosaveStatus}
      onTitleChange={(value) => {
        titleDraft = value;
        scheduleAutosave();
      }}
      onBodyChange={(value) => {
        bodyDraft = value;
        scheduleAutosave();
      }}
      onSave={saveNote}
      onDuplicate={duplicateActive}
      onDelete={deleteActive}
      onCreate={createNewNote}
    />
  </div>
</section>
