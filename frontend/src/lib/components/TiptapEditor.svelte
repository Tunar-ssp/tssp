<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    content: string;
    onChange?: (content: string) => void;
    placeholder?: string;
  }

  let { content = '', onChange, placeholder = 'Start typing...' }: Props = $props();

  let isFocused = $state(false);
  let editorContent = $state('');
  let editorElement: HTMLDivElement | undefined;

  $effect(() => {
    editorContent = content;
  });

  function handleInput(e: Event) {
    if (editorElement) {
      editorContent = editorElement.textContent || '';
      onChange?.(editorContent);
    }
  }

  function handlePaste(e: ClipboardEvent) {
    e.preventDefault();
    const text = e.clipboardData?.getData('text/plain') || '';
    if (document.execCommand) {
      document.execCommand('insertText', false, text);
    }
  }

  function wrapSelection(before: string, after: string = before) {
    const selection = window.getSelection();
    if (!selection || selection.toString() === '') return;

    if (editorElement && document.execCommand) {
      editorElement.focus();
      const text = selection.toString();
      document.execCommand('insertText', false, before + text + after);
    }
  }

  function toggleBold() {
    if (document.execCommand) {
      document.execCommand('bold', false);
      editorElement?.focus();
    }
  }

  function toggleItalic() {
    if (document.execCommand) {
      document.execCommand('italic', false);
      editorElement?.focus();
    }
  }

  function toggleUnderline() {
    if (document.execCommand) {
      document.execCommand('underline', false);
      editorElement?.focus();
    }
  }

  function toggleCode() {
    wrapSelection('`');
  }

  function toggleCodeBlock() {
    wrapSelection('```\n', '\n```');
  }

  function toggleH1() {
    wrapSelection('# ');
  }

  function toggleH2() {
    wrapSelection('## ');
  }

  function toggleH3() {
    wrapSelection('### ');
  }

  function toggleBulletList() {
    wrapSelection('- ');
  }

  function toggleOrderedList() {
    wrapSelection('1. ');
  }

  function toggleBlockquote() {
    wrapSelection('> ');
  }

  function insertLink() {
    const url = prompt('Enter URL:');
    if (url) {
      const selection = window.getSelection();
      const text = selection?.toString() || url;
      wrapSelection('[' + text + '](' + url + ')');
    }
  }

  function handleEditorFocus() {
    isFocused = true;
  }

  function handleEditorBlur() {
    isFocused = false;
  }
</script>

<div class="editor-wrapper">
  <div class="toolbar">
    <div class="toolbar-group">
      <button
        class="toolbar-btn"
        title="Bold (Ctrl+B)"
        onclick={toggleBold}
      >
        <Icons.Bold size={16} />
      </button>
      <button
        class="toolbar-btn"
        title="Italic (Ctrl+I)"
        onclick={toggleItalic}
      >
        <Icons.Italic size={16} />
      </button>
      <button
        class="toolbar-btn"
        title="Underline (Ctrl+U)"
        onclick={toggleUnderline}
      >
        <Icons.Underline size={16} />
      </button>
      <button
        class="toolbar-btn"
        title="Code"
        onclick={toggleCode}
      >
        <Icons.Code2 size={16} />
      </button>
    </div>

    <div class="toolbar-group">
      <button
        class="toolbar-btn"
        title="Heading 1"
        onclick={toggleH1}
      >
        <span style="font-weight: bold;">H1</span>
      </button>
      <button
        class="toolbar-btn"
        title="Heading 2"
        onclick={toggleH2}
      >
        <span style="font-weight: bold;">H2</span>
      </button>
      <button
        class="toolbar-btn"
        title="Heading 3"
        onclick={toggleH3}
      >
        <span style="font-weight: bold;">H3</span>
      </button>
    </div>

    <div class="toolbar-group">
      <button
        class="toolbar-btn"
        title="Bullet List"
        onclick={toggleBulletList}
      >
        <Icons.List size={16} />
      </button>
      <button
        class="toolbar-btn"
        title="Ordered List"
        onclick={toggleOrderedList}
      >
        <Icons.ListOrdered size={16} />
      </button>
      <button
        class="toolbar-btn"
        title="Code Block"
        onclick={toggleCodeBlock}
      >
        <Icons.Brackets size={16} />
      </button>
    </div>

    <div class="toolbar-group">
      <button
        class="toolbar-btn"
        title="Blockquote"
        onclick={toggleBlockquote}
      >
        <Icons.Quote size={16} />
      </button>
      <button class="toolbar-btn" title="Link" onclick={insertLink}>
        <Icons.Link2 size={16} />
      </button>
    </div>
  </div>

  <div class="editor-container" class:focused={isFocused}>
    <div
      bind:this={editorElement}
      class="editor-content"
      contenteditable="true"
      onfocus={handleEditorFocus}
      onblur={handleEditorBlur}
      oninput={handleInput}
      onpaste={handlePaste}
      role="textbox"
      aria-label="Note editor"
      aria-placeholder={placeholder}
    >
      {editorContent}
    </div>
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
    border: 1px solid transparent;
    border-radius: var(--r-1);
    background: transparent;
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
    background: var(--surface-2);
    color: var(--text);
  }

  .editor-container {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-4);
  }

  .editor-content {
    outline: none;
    min-height: 300px;
    line-height: 1.6;
    color: var(--text);
    word-wrap: break-word;
  }

  .editor-content:empty::before {
    content: attr(aria-placeholder);
    color: var(--text-3);
    pointer-events: none;
  }

  .editor-container.focused {
    border-color: var(--blue);
  }
</style>
