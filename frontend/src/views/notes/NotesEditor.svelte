<script lang="ts">
  import type { NoteRecord } from "../../lib/api";
  import { extractHeadings, renderMarkdown } from "../../lib/utils/markdown";

  export let note: NoteRecord | null = null;
  export let titleDraft = "";
  export let bodyDraft = "";
  export let autosaveStatus = "";
  export let onTitleChange: (value: string) => void;
  export let onBodyChange: (value: string) => void;
  export let onSave: () => void;
  export let onDuplicate: () => void;
  export let onDelete: () => void;
  export let onCreate: () => void;

  $: headings = extractHeadings(bodyDraft);
  $: previewHtml = renderMarkdown(bodyDraft);
</script>

<article class="panel-card note-editor-preview">
  <header class="panel-head">
    <strong>{note ? "Note editor" : "Notes editor"}</strong>
    <span>{autosaveStatus || "autosave ready"}</span>
  </header>

  {#if !note}
    <div class="empty-copy">Select a note or create a new page to start editing.</div>
  {:else}
    <div class="action-stack">
      <button class="btn btn-primary" type="button" on:click={onSave}>Save</button>
      <button class="btn btn-secondary" type="button" on:click={onDuplicate}>Duplicate</button>
      <button class="btn btn-secondary" type="button" on:click={onDelete}>Delete</button>
      <button class="btn btn-secondary" type="button" on:click={onCreate}>New note</button>
    </div>

    <div class="note-editor-grid">
      <div class="note-editor-column">
        <input
          class="command-input"
          type="text"
          value={titleDraft}
          placeholder="Note title"
          on:input={(event) => onTitleChange((event.currentTarget as HTMLInputElement).value)}
        />
        <textarea
          class="note-textarea"
          value={bodyDraft}
          placeholder="Write with markdown blocks: headings, checklists, callouts, code, tables."
          on:input={(event) => onBodyChange((event.currentTarget as HTMLTextAreaElement).value)}
        />
      </div>

      <div class="note-editor-column">
        <div class="detail-stack">
          <div class="detail-row"><span>Outline</span><strong>{headings.length} heading(s)</strong></div>
          {#each headings as heading}
            <div class="detail-row"><span>H{heading.level}</span><strong>{heading.text}</strong></div>
          {/each}
        </div>
        <div class="md-preview">{@html previewHtml}</div>
      </div>
    </div>
  {/if}
</article>
