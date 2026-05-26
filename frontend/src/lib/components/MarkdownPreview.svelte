<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { renderMarkdownLite } from '$lib/utils/markdown';

  interface $$Props {
    content?: string;
    showPreview?: boolean;
    onTogglePreview?: (show: boolean) => void;
    onChange?: (content: string) => void;
  }

  let {
    content = '',
    showPreview = true,
    onTogglePreview = () => {},
    onChange = () => {},
  }: $$Props = $props();

  let editorElement: HTMLTextAreaElement | null = $state(null);
  let previewElement: HTMLDivElement | null = $state(null);
  let syncScroll = $state(true);
  let isEditorScrolling = $state(false);

  function handleEditorScroll() {
    if (!syncScroll || !editorElement || !previewElement) return;

    isEditorScrolling = true;
    const editorScrollPercent = editorElement.scrollTop /
      (editorElement.scrollHeight - editorElement.clientHeight);
    previewElement.scrollTop = editorScrollPercent *
      (previewElement.scrollHeight - previewElement.clientHeight);
  }

  function handlePreviewScroll() {
    if (!syncScroll || !editorElement || !previewElement) return;

    const previewScrollPercent = previewElement.scrollTop /
      (previewElement.scrollHeight - previewElement.clientHeight);
    editorElement.scrollTop = previewScrollPercent *
      (editorElement.scrollHeight - editorElement.clientHeight);
  }

  let previewHtml = $derived(renderMarkdownLite(content));

  $effect(() => {
    onChange(content);
  });
</script>

<div class="markdown-preview-container">
  <div class="preview-toolbar">
    <div class="toolbar-left">
      <span class="toolbar-label">Markdown Editor</span>
    </div>

    <div class="toolbar-controls">
      <button
        type="button"
        class="control-btn"
        class:active={syncScroll}
        onclick={() => syncScroll = !syncScroll}
        title="Sync scroll between editor and preview"
        aria-label="Toggle scroll sync"
      >
        <Icons.Link2 size={14} />
      </button>

      <button
        type="button"
        class="control-btn"
        class:active={showPreview}
        onclick={() => onTogglePreview(!showPreview)}
        title="Toggle preview pane"
        aria-label="Toggle preview"
      >
        <Icons.Eye size={14} />
      </button>
    </div>
  </div>

  <div class="preview-content" class:preview-only={!showPreview}>
    <!-- Editor -->
    <div class="editor-pane">
      <textarea
        bind:this={editorElement}
        class="markdown-editor"
        bind:value={content}
        onscroll={handleEditorScroll}
        placeholder="Enter markdown here..."
        spellcheck="false"
      ></textarea>
    </div>

    <!-- Preview -->
    {#if showPreview}
      <div class="preview-divider"></div>
      <div
        class="preview-pane"
        bind:this={previewElement}
        onscroll={handlePreviewScroll}
      >
        <div class="preview-content-inner markdown-body">
          {@html previewHtml}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .markdown-preview-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
  }

  .preview-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    flex-shrink: 0;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .toolbar-label {
    font-size: var(--fs-12);
    color: var(--text-2);
    font-weight: 500;
  }

  .toolbar-controls {
    display: flex;
    gap: var(--s-1);
  }

  .control-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .control-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .control-btn.active {
    background: var(--blue-soft);
    border-color: var(--blue);
    color: var(--blue);
  }

  .preview-content {
    display: flex;
    flex: 1;
    overflow: hidden;
    gap: 0;
  }

  .preview-content.preview-only .editor-pane {
    display: none;
  }

  .preview-content.preview-only .preview-divider {
    display: none;
  }

  .editor-pane {
    flex: 1;
    overflow: hidden;
    display: flex;
  }

  .markdown-editor {
    flex: 1;
    border: none;
    padding: var(--s-3);
    background: var(--bg);
    color: var(--text);
    font-family: var(--ff-mono);
    font-size: var(--fs-13);
    line-height: 1.6;
    resize: none;
    outline: none;
  }

  .markdown-editor::placeholder {
    color: var(--muted);
  }

  .preview-divider {
    width: 1px;
    background: var(--border);
    cursor: col-resize;
  }

  .preview-pane {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-3);
    background: var(--bg);
  }

  .preview-content-inner {
    max-width: 80ch;
  }

  /* Markdown styling */
  :global(.markdown-body) {
    font-family: var(--ff-sans);
    font-size: var(--fs-13);
    line-height: 1.6;
    color: var(--text);
  }

  :global(.markdown-body h1),
  :global(.markdown-body h2),
  :global(.markdown-body h3),
  :global(.markdown-body h4),
  :global(.markdown-body h5),
  :global(.markdown-body h6) {
    font-weight: 600;
    margin-top: 24px;
    margin-bottom: 12px;
    color: var(--text);
  }

  :global(.markdown-body h1) {
    font-size: var(--fs-20);
    border-bottom: 1px solid var(--border);
    padding-bottom: 8px;
  }

  :global(.markdown-body h2) {
    font-size: var(--fs-18);
  }

  :global(.markdown-body h3) {
    font-size: var(--fs-16);
  }

  :global(.markdown-body p) {
    margin: 12px 0;
  }

  :global(.markdown-body a) {
    color: var(--blue);
    text-decoration: none;
  }

  :global(.markdown-body a:hover) {
    text-decoration: underline;
  }

  :global(.markdown-body code) {
    background: var(--surface-2);
    color: var(--text);
    padding: 2px 6px;
    border-radius: var(--r-1);
    font-family: var(--ff-mono);
    font-size: 0.9em;
  }

  :global(.markdown-body pre) {
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    padding: var(--s-3);
    overflow-x: auto;
    margin: 12px 0;
  }

  :global(.markdown-body pre code) {
    background: transparent;
    color: var(--text);
    padding: 0;
  }

  :global(.markdown-body blockquote) {
    border-left: 3px solid var(--border);
    padding-left: var(--s-3);
    color: var(--text-2);
    margin: 12px 0;
  }

  :global(.markdown-body ul),
  :global(.markdown-body ol) {
    margin: 12px 0;
    padding-left: var(--s-6);
  }

  :global(.markdown-body li) {
    margin: 6px 0;
  }

  :global(.markdown-body table) {
    border-collapse: collapse;
    width: 100%;
    margin: 12px 0;
  }

  :global(.markdown-body table td),
  :global(.markdown-body table th) {
    border: 1px solid var(--border);
    padding: var(--s-2);
    text-align: left;
  }

  :global(.markdown-body table th) {
    background: var(--surface-2);
    font-weight: 600;
  }

  :global(.markdown-body hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 24px 0;
  }

  :global(.markdown-body img) {
    max-width: 100%;
    height: auto;
    border-radius: var(--r-2);
  }
</style>
