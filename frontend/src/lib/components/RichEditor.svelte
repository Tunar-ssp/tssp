<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    content: string;
    onChange?: (content: string) => void;
    placeholder?: string;
  }

  let { content = '', onChange, placeholder = 'Start typing... (Ctrl+/ for formatting)' }: Props = $props();

  let editorDiv: HTMLDivElement;
  let isFocused = $state(false);

  $effect(() => {
    if (editorDiv && content !== editorDiv.innerHTML) {
      editorDiv.innerHTML = content || '<p></p>';
    }
  });

  function handleInput() {
    if (editorDiv) {
      const html = editorDiv.innerHTML;
      onChange?.(html);
    }
  }

  function handleBlur() {
    isFocused = false;
  }

  function handleFocus() {
    isFocused = true;
  }

  function toggleBold() {
    document.execCommand('bold', false, undefined);
    editorDiv?.focus();
  }

  function toggleItalic() {
    document.execCommand('italic', false, undefined);
    editorDiv?.focus();
  }

  function toggleUnderline() {
    document.execCommand('underline', false, undefined);
    editorDiv?.focus();
  }

  function toggleCode() {
    document.execCommand('formatBlock', false, '<pre>');
    editorDiv?.focus();
  }

  function toggleH1() {
    document.execCommand('formatBlock', false, '<h1>');
    editorDiv?.focus();
  }

  function toggleH2() {
    document.execCommand('formatBlock', false, '<h2>');
    editorDiv?.focus();
  }

  function toggleBulletList() {
    document.execCommand('insertUnorderedList', false, undefined);
    editorDiv?.focus();
  }

  function toggleOrderedList() {
    document.execCommand('insertOrderedList', false, undefined);
    editorDiv?.focus();
  }

  function insertLink() {
    const url = prompt('Enter URL:');
    if (url) {
      document.execCommand('createLink', false, url);
      editorDiv?.focus();
    }
  }

  function getWordCount(text: string): number {
    const div = document.createElement('div');
    div.innerHTML = text;
    return div.textContent?.trim().split(/\s+/).length || 0;
  }
</script>

<div class="editor-wrapper">
  <div class="toolbar">
    <div class="toolbar-group">
      <button class="toolbar-btn" title="Bold (Ctrl+B)" on:click={toggleBold}>
        <Icons.Bold size={16} />
      </button>
      <button class="toolbar-btn" title="Italic (Ctrl+I)" on:click={toggleItalic}>
        <Icons.Italic size={16} />
      </button>
      <button class="toolbar-btn" title="Underline (Ctrl+U)" on:click={toggleUnderline}>
        <Icons.Underline size={16} />
      </button>
    </div>

    <div class="toolbar-group">
      <button class="toolbar-btn" title="Heading 1" on:click={toggleH1}>
        <span style="font-weight: bold; font-size: 14px;">H1</span>
      </button>
      <button class="toolbar-btn" title="Heading 2" on:click={toggleH2}>
        <span style="font-weight: bold; font-size: 13px;">H2</span>
      </button>
      <button class="toolbar-btn" title="Code Block" on:click={toggleCode}>
        <Icons.Code2 size={16} />
      </button>
    </div>

    <div class="toolbar-group">
      <button class="toolbar-btn" title="Bullet List" on:click={toggleBulletList}>
        <Icons.List size={16} />
      </button>
      <button class="toolbar-btn" title="Ordered List" on:click={toggleOrderedList}>
        <Icons.ListOrdered size={16} />
      </button>
      <button class="toolbar-btn" title="Link" on:click={insertLink}>
        <Icons.Link2 size={16} />
      </button>
    </div>
  </div>

  <div class="editor-container" class:focused={isFocused}>
    <div
      bind:this={editorDiv}
      class="editor-content"
      contenteditable="true"
      role="textbox"
      aria-label="Rich text editor"
      on:input={handleInput}
      on:focus={handleFocus}
      on:blur={handleBlur}
    >
      <p></p>
    </div>
  </div>

  <div class="editor-footer">
    <span>{getWordCount(editorDiv?.innerHTML || '')} words</span>
  </div>
</div>

<style>
  .editor-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    overflow: hidden;
    background: var(--bg);
  }

  .toolbar {
    display: flex;
    gap: var(--s-1);
    padding: var(--s-2);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    flex-wrap: wrap;
    align-items: center;
  }

  .toolbar-group {
    display: flex;
    gap: var(--s-1);
    border-right: 1px solid var(--border);
    padding-right: var(--s-2);
  }

  .toolbar-group:last-child {
    border-right: none;
  }

  .toolbar-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
    font-size: 12px;
    font-weight: 600;
  }

  .toolbar-btn:hover {
    background: var(--surface);
    color: var(--text);
    border-color: var(--blue);
  }

  .editor-container {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-3);
    border: 1px solid transparent;
    transition: border-color var(--duration-quick);
  }

  .editor-container.focused {
    border-color: var(--blue);
  }

  .editor-content {
    outline: none;
    min-height: 300px;
    line-height: 1.6;
    color: var(--text);
    word-wrap: break-word;
  }

  .editor-content :global(h1) {
    font-size: 2em;
    font-weight: 700;
    margin: 0.5em 0;
  }

  .editor-content :global(h2) {
    font-size: 1.5em;
    font-weight: 600;
    margin: 0.4em 0;
  }

  .editor-content :global(p) {
    margin: 0.5em 0;
  }

  .editor-content :global(ul),
  .editor-content :global(ol) {
    margin: 0.5em 0 0.5em 2em;
  }

  .editor-content :global(li) {
    margin: 0.25em 0;
  }

  .editor-content :global(pre) {
    background: var(--surface-2);
    border-radius: var(--r-2);
    padding: var(--s-3);
    overflow-x: auto;
    margin: 0.5em 0;
  }

  .editor-content :global(code) {
    background: var(--surface-2);
    padding: 0.1em 0.3em;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.9em;
  }

  .editor-content :global(a) {
    color: var(--blue);
    text-decoration: underline;
  }

  .editor-footer {
    padding: var(--s-2) var(--s-3);
    border-top: 1px solid var(--border);
    background: var(--surface);
    font-size: var(--fs-12);
    color: var(--muted);
  }
</style>
