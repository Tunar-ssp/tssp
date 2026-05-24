<script lang="ts">
  import type { NoteRecord } from "../../lib/api";
  import { formatRelativeDate } from "../../lib/utils/format";

  export let notes: NoteRecord[] = [];
  export let activeId: string | null = null;
  export let onSelectNote: (note: NoteRecord) => void;
</script>

<div class="stack-list">
  {#each notes as note, index}
    <button
      type="button"
      class:selected={activeId === note.id}
      class={`stack-card ${index % 3 === 0 ? "accent-blue" : index % 3 === 1 ? "accent-violet" : "accent-green"}`}
      on:click={() => onSelectNote(note)}
    >
      <strong>{note.title || "Untitled note"}</strong>
      <span>
        {note.pinned_at ? "Pinned · " : ""}{note.tags.join(", ") || "untagged"} ·
        {formatRelativeDate(note.updated_at)}
      </span>
      <span>{(note.body || "").replace(/\s+/g, " ").trim().slice(0, 96) || "Empty note"}</span>
    </button>
  {/each}
</div>
