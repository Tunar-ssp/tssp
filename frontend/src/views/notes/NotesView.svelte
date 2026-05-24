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
    autosaveStatus = `Editing · ${formatRelativeDate(note.updated_at)}`;
  }

  function activeNote() {
    return notes.find((note) => note.id === activeNoteId) || null;
  }

  function scheduleAutosave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => void saveNote(), 1500);
  }

  async function loadNotes() {
    loading = true;
    error = "";
    try {
      const response = await listNotes({ limit: 100, sort: "-updated" });
      notes = response.notes || [];
      if (!activeNoteId && notes[0]) selectNote(notes[0]);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load notes";
    } finally {
      loading = false;
    }
  }

  async function saveNote() {
    const note = activeNote();
    if (!note) return;
    try {
      const updated = await updateNote(note.id, {
        title: titleDraft.trim() || "Untitled",
        body: bodyDraft,
      });
      notes = notes.map((item) => (item.id === updated.id ? updated : item));
      autosaveStatus = `Saved · ${formatRelativeDate(updated.updated_at)}`;
    } catch (e) {
      autosaveStatus = e instanceof Error ? e.message : "Save failed";
    }
  }

  async function createNewNote() {
    const created = await createNote({
      title: "Untitled",
      body: "# New page\n\n",
      tags: [],
    });
    notes = [created, ...notes];
    selectNote(created);
  }

  async function duplicateActive() {
    const note = activeNote();
    if (!note) return;
    const duplicate = await duplicateNote(note.id);
    notes = [duplicate, ...notes];
    selectNote(duplicate);
  }

  async function deleteActive() {
    const note = activeNote();
    if (!note || !confirm("Delete this note?")) return;
    await deleteNote(note.id);
    notes = notes.filter((n) => n.id !== note.id);
    if (notes[0]) selectNote(notes[0]);
    else {
      activeNoteId = null;
      titleDraft = "";
      bodyDraft = "";
    }
  }

  onMount(() => void loadNotes());
</script>

<section class="notes">
  <header class="notes-head">
    <div>
      <h1>Knowledge</h1>
      <p class="muted">{autosaveStatus}</p>
    </div>
    <div class="actions">
      <button type="button" class="btn btn-primary btn-sm" on:click={createNewNote}>New note</button>
      <button type="button" class="btn btn-sm" on:click={loadNotes}>Refresh</button>
    </div>
  </header>

  <div class="notes-body">
    <aside class="notes-list">
      {#if loading}
        <div class="empty-state">Loading…</div>
      {:else if error}
        <div class="empty-state">{error}</div>
      {:else if notes.length === 0}
        <div class="empty-state"><strong>No notes</strong>Create your first page.</div>
      {:else}
        {#each notes as note}
          <button
            type="button"
            class="note-card"
            class:active={note.id === activeNoteId}
            on:click={() => selectNote(note)}
          >
            <strong>{note.title || "Untitled"}</strong>
            <span>{(note.body || "").slice(0, 80)}</span>
            <span class="meta">{formatRelativeDate(note.updated_at)}</span>
          </button>
        {/each}
      {/if}
    </aside>

    <NotesEditor
      note={activeNote()}
      {titleDraft}
      {bodyDraft}
      {autosaveStatus}
      onTitleChange={(v) => {
        titleDraft = v;
        scheduleAutosave();
      }}
      onBodyChange={(v) => {
        bodyDraft = v;
        scheduleAutosave();
      }}
      onSave={saveNote}
      onDuplicate={duplicateActive}
      onDelete={deleteActive}
      onCreate={createNewNote}
    />
  </div>
</section>

<style>
  .notes { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .notes-head { display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; border-bottom: 1px solid var(--border); }
  .notes-head h1 { margin: 0; font-size: 16px; }
  .actions { display: flex; gap: 8px; }
  .notes-body { flex: 1; min-height: 0; display: grid; grid-template-columns: 280px minmax(0, 1fr); }
  .notes-list { overflow: auto; border-right: 1px solid var(--border); padding: 10px; display: grid; gap: 8px; align-content: start; }
  .note-card { text-align: left; border: 1px solid var(--border); background: var(--bg-card); border-radius: var(--radius-md); padding: 10px 12px; display: grid; gap: 4px; }
  .note-card.active { border-color: var(--brand); background: var(--brand-dim); }
  .note-card strong { font-size: 13px; }
  .note-card span { font-size: 12px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .meta { font-size: 11px !important; color: var(--text-dim) !important; }
  .muted { color: var(--text-muted); font-size: 12px; margin: 4px 0 0; }
  @media (max-width: 900px) { .notes-body { grid-template-columns: 1fr; } .notes-list { max-height: 200px; } }
</style>
