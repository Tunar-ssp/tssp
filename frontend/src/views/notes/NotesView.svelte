<script lang="ts">
  import { sortedNotes, activeNote, loadNotes, setActiveNote } from '$lib/stores/notes';
  import NotesList from '$lib/components/NotesList.svelte';
  import NoteEditor from '$lib/components/NoteEditor.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    loadNotes();
  });

  function handleSelectNote(id: string) {
    setActiveNote(id);
  }
</script>

<div class="notes-view">
  <NotesList notes={$sortedNotes} onSelectNote={handleSelectNote} />
  <NoteEditor note={$activeNote} />
</div>

<style>
  .notes-view {
    flex: 1;
    display: flex;
    overflow: hidden;
    background: var(--bg);
  }
</style>
