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

  let textarea: HTMLTextAreaElement | null = null;
  let slashOpen = false;
  let slashFilter = "";

  const slashCommands = [
    { id: "h1", label: "Heading 1", insert: "# " },
    { id: "h2", label: "Heading 2", insert: "## " },
    { id: "bullet", label: "Bullet list", insert: "- " },
    { id: "check", label: "Checklist", insert: "- [ ] " },
    { id: "code", label: "Code block", insert: "```\n\n```" },
    { id: "quote", label: "Callout", insert: "> " },
    { id: "rule", label: "Divider", insert: "---\n" },
    { id: "table", label: "Table", insert: "| Col A | Col B |\n| --- | --- |\n| | |\n" },
  ];

  $: headings = extractHeadings(bodyDraft);
  $: previewHtml = renderMarkdown(bodyDraft);
  $: filteredSlash = slashCommands.filter((c) =>
    c.label.toLowerCase().includes(slashFilter.toLowerCase()),
  );

  function insertAtCursor(text: string) {
    if (!textarea) {
      onBodyChange(bodyDraft + text);
      return;
    }
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const next = `${bodyDraft.slice(0, start)}${text}${bodyDraft.slice(end)}`;
    onBodyChange(next);
    slashOpen = false;
    slashFilter = "";
  }

  function handleBodyInput(ev: Event) {
    const el = ev.currentTarget as HTMLTextAreaElement;
    const val = el.value;
    onBodyChange(val);
    const lineStart = val.lastIndexOf("\n", el.selectionStart - 1) + 1;
    const prefix = val.slice(lineStart, el.selectionStart);
    if (prefix === "/") {
      slashOpen = true;
      slashFilter = "";
    } else if (prefix.startsWith("/")) {
      slashOpen = true;
      slashFilter = prefix.slice(1);
    } else {
      slashOpen = false;
    }
  }

  function wrapSelection(before: string, after: string) {
    if (!textarea) return;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selected = bodyDraft.slice(start, end);
    const next = `${bodyDraft.slice(0, start)}${before}${selected}${after}${bodyDraft.slice(end)}`;
    onBodyChange(next);
  }

  function handleKeydown(ev: KeyboardEvent) {
    if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === "s") {
      ev.preventDefault();
      onSave();
    }
    if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === "b") {
      ev.preventDefault();
      wrapSelection("**", "**");
    }
    if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === "i") {
      ev.preventDefault();
      wrapSelection("*", "*");
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<article class="note-editor">
  {#if !note}
    <div class="empty-state"><strong>Select a note</strong> or create a new page.</div>
  {:else}
    <header class="note-toolbar">
      <input class="title-input" type="text" value={titleDraft} placeholder="Untitled" on:input={(e) => onTitleChange(e.currentTarget.value)} />
      <span class="status">{autosaveStatus}</span>
      <div class="actions">
        <button type="button" class="btn btn-sm" on:click={onSave}>Save</button>
        <button type="button" class="btn btn-sm" on:click={onDuplicate}>Duplicate</button>
        <button type="button" class="btn btn-sm btn-danger" on:click={onDelete}>Delete</button>
      </div>
    </header>

    <div class="block-toolbar">
      {#each [
        { label: "H1", insert: "# " },
        { label: "List", insert: "- " },
        { label: "Todo", insert: "- [ ] " },
        { label: "Code", insert: "```\n\n```" },
        { label: "Quote", insert: "> " },
      ] as block}
        <button type="button" class="btn btn-sm btn-ghost" on:click={() => insertAtCursor(block.insert)}>{block.label}</button>
      {/each}
    </div>

    <div class="editor-grid">
      <div class="editor-pane">
        <textarea
          bind:this={textarea}
          class="body-input mono"
          value={bodyDraft}
          placeholder="Type / for commands…"
          on:input={handleBodyInput}
        ></textarea>
        {#if slashOpen}
          <div class="slash-menu">
            {#each filteredSlash as cmd}
              <button type="button" class="slash-item" on:click={() => insertAtCursor(cmd.insert.replace(/^\//, ""))}>
                {cmd.label}
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <aside class="preview-pane">
        <h3>Outline</h3>
        {#if headings.length === 0}
          <p class="muted">Headings appear here</p>
        {:else}
          <ul class="outline">
            {#each headings as h}
              <li style={`padding-left: ${(h.level - 1) * 12}px`}>{h.text}</li>
            {/each}
          </ul>
        {/if}
        <h3>Preview</h3>
        <div class="md-preview">{@html previewHtml}</div>
      </aside>
    </div>
  {/if}
</article>

<style>
  .note-editor { display: flex; flex-direction: column; min-height: 0; height: 100%; background: var(--bg-card); border-left: 1px solid var(--border); }
  .note-toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; padding: 12px 16px; border-bottom: 1px solid var(--border); }
  .title-input { flex: 1; min-width: 180px; border: none; background: transparent; font-size: 20px; font-weight: 600; outline: none; }
  .status { font-size: 12px; color: var(--text-muted); }
  .actions { display: flex; gap: 6px; margin-left: auto; }
  .block-toolbar { display: flex; flex-wrap: wrap; gap: 4px; padding: 8px 16px; border-bottom: 1px solid var(--border); }
  .editor-grid { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) 280px; }
  .editor-pane { position: relative; min-height: 0; }
  .body-input { width: 100%; height: 100%; min-height: 320px; border: none; resize: none; padding: 16px; background: var(--bg-base); color: var(--text); font-size: 14px; line-height: 1.6; outline: none; }
  .slash-menu { position: absolute; left: 16px; bottom: 16px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 6px; display: grid; gap: 2px; min-width: 160px; box-shadow: 0 8px 32px rgba(0,0,0,0.4); }
  .slash-item { text-align: left; border: none; background: transparent; padding: 8px 10px; border-radius: var(--radius-sm); font-size: 13px; }
  .slash-item:hover { background: var(--bg-hover); }
  .preview-pane { overflow: auto; padding: 12px 16px; border-left: 1px solid var(--border); font-size: 13px; }
  .preview-pane h3 { margin: 0 0 8px; font-size: 11px; text-transform: uppercase; letter-spacing: 0.08em; color: var(--text-muted); }
  .outline { list-style: none; padding: 0; margin: 0 0 16px; }
  .outline li { padding: 4px 0; color: var(--text-muted); }
  .md-preview :global(p) { margin: 0 0 8px; }
  .md-preview :global(.md-code) { background: var(--bg-surface); padding: 10px; border-radius: var(--radius-sm); overflow: auto; font-family: var(--font-mono); font-size: 12px; }
  .md-preview :global(.md-callout) { border-left: 3px solid var(--brand); padding: 8px 12px; background: var(--brand-dim); margin: 8px 0; }
  .muted { color: var(--text-dim); font-size: 12px; }
  @media (max-width: 900px) { .editor-grid { grid-template-columns: 1fr; } .preview-pane { display: none; } }
</style>
