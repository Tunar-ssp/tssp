<script lang="ts">
  import type { WorkspaceDocumentRecord, WorkspaceRecord } from "../../lib/api";
  import { inferLanguageFromPath } from "../../lib/utils/workspace";
  import { formatRelativeDate } from "../../lib/utils/format";

  export let workspace: WorkspaceRecord | null = null;
  export let document: WorkspaceDocumentRecord | null = null;
  export let bodyDraft = "";
  export let saveStatus = "";
  export let onBodyChange: (value: string) => void;
  export let onSave: () => void;
  export let onCreateDocument: () => void;
  export let onDeleteDocument: () => void;
</script>

<article class="panel-card ide-editor-card">
  <header class="panel-head">
    <strong>Editor surface</strong>
    <span>tabs, save state, markdown preview, find bar</span>
  </header>

  {#if !workspace}
    <div class="empty-copy">Choose a workspace to start editing documents.</div>
  {:else if !document}
    <div class="empty-copy">No document selected yet.</div>
  {:else}
    <div class="action-stack">
      <button class="btn btn-primary" type="button" on:click={onSave}>Save</button>
      <button class="btn btn-secondary" type="button" on:click={onCreateDocument}>New document</button>
      <button class="btn btn-secondary" type="button" on:click={onDeleteDocument}>Delete document</button>
    </div>

    <div class="detail-stack">
      <div class="detail-row"><span>Path</span><strong>{document.path}</strong></div>
      <div class="detail-row"><span>Language</span><strong>{document.language || inferLanguageFromPath(document.path)}</strong></div>
      <div class="detail-row"><span>Updated</span><strong>{formatRelativeDate(document.updated_at)}</strong></div>
      <div class="detail-row"><span>Status</span><strong>{saveStatus || "idle"}</strong></div>
    </div>

    <textarea
      class="note-textarea"
      value={bodyDraft}
      on:input={(event) => onBodyChange((event.currentTarget as HTMLTextAreaElement).value)}
    />
  {/if}
</article>
