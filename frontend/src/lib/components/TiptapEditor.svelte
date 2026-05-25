<script lang="ts">
  import { useEditor, EditorContent } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import Link from '@tiptap/extension-link';
  import * as Icons from 'lucide-svelte';

  interface Props {
    content: string;
    onChange?: (content: string) => void;
    placeholder?: string;
  }

  let { content = '', onChange, placeholder = 'Start typing...' }: Props = $props();

  let editor = $state<any>(null);
  let isFocused = $state(false);

  $effect(() => {
    if (!editor) {
      editor = useEditor({
        extensions: [
          StarterKit,
          Link.configure({
            openOnClick: false,
            autolink: true,
          }),
        ],
        content,
        onUpdate: ({ editor }) => {
          const html = editor.getHTML();
          onChange?.(html);
        },
      });
    }

    return () => {
      if (editor) {
        editor.destroy();
      }
    };
  });

  function toggleBold() {
    editor?.chain().focus().toggleBold().run();
  }

  function toggleItalic() {
    editor?.chain().focus().toggleItalic().run();
  }

  function toggleUnderline() {
    editor?.chain().focus().toggleUnderline().run();
  }

  function toggleCode() {
    editor?.chain().focus().toggleCode().run();
  }

  function toggleCodeBlock() {
    editor?.chain().focus().toggleCodeBlock().run();
  }

  function toggleH1() {
    editor?.chain().focus().toggleHeading({ level: 1 }).run();
  }

  function toggleH2() {
    editor?.chain().focus().toggleHeading({ level: 2 }).run();
  }

  function toggleH3() {
    editor?.chain().focus().toggleHeading({ level: 3 }).run();
  }

  function toggleBulletList() {
    editor?.chain().focus().toggleBulletList().run();
  }

  function toggleOrderedList() {
    editor?.chain().focus().toggleOrderedList().run();
  }

  function toggleBlockquote() {
    editor?.chain().focus().toggleBlockquote().run();
  }

  function insertLink() {
    const url = prompt('Enter URL:');
    if (url) {
      editor?.chain().focus().extendMarkRange('link').setLink({ href: url }).run();
    }
  }

  function insertTable() {
    editor?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
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
        on:click={toggleBold}
        class:active={editor?.isActive('bold')}
      >
        <Icons.Bold size={16} />
      </button>
      <button
        class="toolbar-btn"
        title="Italic (Ctrl+I)"
        on:click={toggleItalic}
        class:active={editor?.isActive('italic')}
      >
        <Icons.Italic size={16} />
      </button>
      <button
        class="toolbar-btn"
        title="Underline (Ctrl+U)"
        on:click={toggleUnderline}
        class:active={editor?.isActive('underline')}
      >
        <Icons.Underline size={16} />
      </button>
      <button
        class="toolbar-btn"
        title="Code"
        on:click={toggleCode}
        class:active={editor?.isActive('code')}
      >
        <Icons.Code2 size={16} />
      </button>
    </div>

    <div class="toolbar-group">
      <button
        class="toolbar-btn"
        title="Heading 1"
        on:click={toggleH1}
        class:active={editor?.isActive('heading', { level: 1 })}
      >
        <span style="font-weight: bold;">H1</span>
      </button>
      <button
        class="toolbar-btn"
        title="Heading 2"
        on:click={toggleH2}
        class:active={editor?.isActive('heading', { level: 2 })}
      >
        <span style="font-weight: bold;">H2</span>
      </button>
      <button
        class="toolbar-btn"
        title="Heading 3"
        on:click={toggleH3}
        class:active={editor?.isActive('heading', { level: 3 })}
      >
        <span style="font-weight: bold;">H3</span>
      </button>
    </div>

    <div class="toolbar-group">
      <button
        class="toolbar-btn"
        title="Bullet List"
        on:click={toggleBulletList}
        class:active={editor?.isActive('bulletList')}
      >
        <Icons.List size={16} />
      </button>
      <button
        class="toolbar-btn"
        title="Ordered List"
        on:click={toggleOrderedList}
        class:active={editor?.isActive('orderedList')}
      >
        <Icons.ListOrdered size={16} />
      </button>
      <button
        class="toolbar-btn"
        title="Code Block"
        on:click={toggleCodeBlock}
        class:active={editor?.isActive('codeBlock')}
      >
        <Icons.Brackets size={16} />
      </button>
    </div>

    <div class="toolbar-group">
      <button
        class="toolbar-btn"
        title="Blockquote"
        on:click={toggleBlockquote}
        class:active={editor?.isActive('blockquote')}
      >
        <Icons.Quote2 size={16} />
      </button>
      <button class="toolbar-btn" title="Link" on:click={insertLink}>
        <Icons.Link2 size={16} />
      </button>
    </div>
  </div>

  <div class="editor-container" class:focused={isFocused}>
    <EditorContent
      editor={editor}
      on:focus={handleEditorFocus}
      on:blur={handleEditorBlur}
    />
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

  .toolbar-btn.active {
    background: var(--blue);
    color: white;
    border-color: var(--blue);
  }

  .editor-container {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-4);
  }

  .editor-container :global(.ProseMirror) {
    outline: none;
    min-height: 300px;
  }

  .editor-container :global(.ProseMirror p) {
    margin: 0 0 1em 0;
    line-height: 1.6;
  }

  .editor-container :global(.ProseMirror h1) {
    font-size: 2em;
    font-weight: 700;
    margin: 0.5em 0;
  }

  .editor-container :global(.ProseMirror h2) {
    font-size: 1.5em;
    font-weight: 600;
    margin: 0.4em 0;
  }

  .editor-container :global(.ProseMirror h3) {
    font-size: 1.25em;
    font-weight: 600;
    margin: 0.3em 0;
  }

  .editor-container :global(.ProseMirror ul) {
    margin: 1em 0;
    padding-left: 2em;
  }

  .editor-container :global(.ProseMirror ol) {
    margin: 1em 0;
    padding-left: 2em;
  }

  .editor-container :global(.ProseMirror li) {
    margin: 0.25em 0;
  }

  .editor-container :global(.ProseMirror code) {
    background: var(--surface-2);
    padding: 0.1em 0.3em;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.9em;
    color: var(--orange);
  }

  .editor-container :global(.ProseMirror pre) {
    background: var(--surface-2);
    border-radius: var(--r-2);
    padding: var(--s-3);
    overflow-x: auto;
    margin: 1em 0;
  }

  .editor-container :global(.ProseMirror pre code) {
    background: none;
    padding: 0;
    color: inherit;
  }

  .editor-container :global(.ProseMirror blockquote) {
    border-left: 4px solid var(--blue);
    padding-left: 1em;
    margin: 1em 0;
    color: var(--text-2);
    font-style: italic;
  }

  .editor-container :global(.ProseMirror a) {
    color: var(--blue);
    text-decoration: underline;
    cursor: pointer;
  }

  .editor-container :global(.ProseMirror a:hover) {
    color: var(--blue);
    text-decoration-color: transparent;
  }

  .editor-container.focused {
    border-color: var(--blue);
  }
</style>
