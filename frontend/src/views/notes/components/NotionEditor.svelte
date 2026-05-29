<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import { TaskList, TaskItem } from '@tiptap/extension-list';
  import { TableKit } from '@tiptap/extension-table';
  import { Placeholder } from '@tiptap/extensions';
  import * as Icons from 'lucide-svelte';
  import { markdownToHtml, docToMarkdown } from '$lib/notes/markdown';
  import { SLASH_COMMANDS, filterSlashCommands, type SlashCommand } from '$lib/notes/slashCommands';

  interface Props {
    /** Note id; changing it loads a different document. */
    noteId: string;
    /** Initial markdown body. Read once per noteId change. */
    markdown: string;
    editable?: boolean;
    onChange?: (markdown: string) => void;
  }

  let { noteId, markdown, editable = true, onChange }: Props = $props();

  let element: HTMLDivElement;
  let editor: Editor | null = null;
  let loadedNoteId = $state<string | null>(null);
  /** Suppress onChange while we programmatically swap documents. */
  let applyingExternal = false;

  // Slash command menu state
  let showSlash = $state(false);
  let slashQuery = $state('');
  let slashIndex = $state(0);
  let slashPos = $state({ top: 0, left: 0 });
  let slashRange = { from: 0, to: 0 };
  let filtered = $derived(filterSlashCommands(slashQuery));

  // Selection bubble menu state
  let showBubble = $state(false);
  let bubblePos = $state({ top: 0, left: 0 });
  let activeMarks = $state<Record<string, boolean>>({});

  function emitChange() {
    if (!editor || applyingExternal) return;
    onChange?.(docToMarkdown(editor.getJSON() as any));
  }

  function closeSlash() {
    showSlash = false;
    slashQuery = '';
    slashIndex = 0;
  }

  function detectSlash() {
    if (!editor) return closeSlash();
    const { state } = editor;
    const sel = state.selection;
    if (!sel.empty) return closeSlash();
    const resolved = sel.$from;
    if (resolved.parent.type.name === 'codeBlock') return closeSlash();

    const textBefore = resolved.parent.textBetween(0, resolved.parentOffset, undefined, '￼');
    const match = /(?:^|\s)\/(\w*)$/.exec(textBefore);
    if (!match) return closeSlash();

    slashQuery = match[1];
    slashIndex = 0;
    slashRange = { from: sel.from - match[1].length - 1, to: sel.from };

    const coords = editor.view.coordsAtPos(sel.from);
    const rect = element.getBoundingClientRect();
    slashPos = { top: coords.bottom - rect.top + 6, left: coords.left - rect.left };
    showSlash = true;
  }

  function applyCommand(cmd: SlashCommand) {
    if (!editor) return;
    editor.chain().focus().deleteRange(slashRange).run();
    cmd.run(editor);
    closeSlash();
  }

  function updateBubble() {
    if (!editor) return;
    const { state } = editor;
    const sel = state.selection;
    const isText = sel.constructor.name !== 'NodeSelection';
    if (sel.empty || !isText || state.selection.$from.parent.type.name === 'codeBlock') {
      showBubble = false;
      return;
    }
    const start = editor.view.coordsAtPos(sel.from);
    const end = editor.view.coordsAtPos(sel.to);
    const rect = element.getBoundingClientRect();
    const left = (start.left + end.left) / 2 - rect.left;
    bubblePos = { top: start.top - rect.top - 44, left };
    activeMarks = {
      bold: editor.isActive('bold'),
      italic: editor.isActive('italic'),
      strike: editor.isActive('strike'),
      code: editor.isActive('code'),
      underline: editor.isActive('underline'),
      link: editor.isActive('link'),
    };
    showBubble = true;
  }

  function toggleLink() {
    if (!editor) return;
    if (editor.isActive('link')) {
      editor.chain().focus().unsetLink().run();
      return;
    }
    const previous = editor.getAttributes('link').href ?? '';
    const url = window.prompt('Link URL', previous);
    if (url === null) return;
    if (url === '') {
      editor.chain().focus().unsetLink().run();
    } else {
      editor.chain().focus().setLink({ href: url }).run();
    }
  }

  onMount(() => {
    editor = new Editor({
      element,
      editable,
      extensions: [
        StarterKit.configure({
          link: { openOnClick: false, autolink: true, HTMLAttributes: { rel: 'noopener noreferrer' } },
          heading: { levels: [1, 2, 3] },
        }),
        TaskList,
        TaskItem.configure({ nested: true }),
        TableKit.configure({ table: { resizable: true } }),
        Placeholder.configure({
          placeholder: ({ node }: { node: { type: { name: string } } }) =>
            node.type.name === 'heading' ? 'Heading' : "Type '/' for commands…",
        }),
      ],
      content: markdownToHtml(markdown),
      editorProps: {
        attributes: { class: 'tiptap-content', spellcheck: 'true' },
        handleKeyDown: (_view, event) => {
          if (!showSlash) return false;
          const list = filtered;
          if (event.key === 'ArrowDown') {
            slashIndex = (slashIndex + 1) % Math.max(list.length, 1);
            return true;
          }
          if (event.key === 'ArrowUp') {
            slashIndex = (slashIndex - 1 + list.length) % Math.max(list.length, 1);
            return true;
          }
          if (event.key === 'Enter') {
            if (list[slashIndex]) applyCommand(list[slashIndex]);
            return true;
          }
          if (event.key === 'Escape') {
            closeSlash();
            return true;
          }
          return false;
        },
      },
      onUpdate: () => {
        emitChange();
        detectSlash();
      },
      onSelectionUpdate: () => {
        detectSlash();
        updateBubble();
      },
      onBlur: () => {
        // Let click handlers on the menus run before hiding.
        setTimeout(() => {
          showBubble = false;
        }, 150);
      },
    });
    loadedNoteId = noteId;
  });

  // Swap document when a different note is opened.
  $effect(() => {
    const currentId = noteId;
    const body = markdown;
    if (editor && currentId !== loadedNoteId) {
      applyingExternal = true;
      editor.commands.setContent(markdownToHtml(body), { emitUpdate: false });
      applyingExternal = false;
      loadedNoteId = currentId;
      closeSlash();
      showBubble = false;
    }
  });

  onDestroy(() => {
    editor?.destroy();
    editor = null;
  });
</script>

<div class="notion-editor" bind:this={element}>
  {#if showSlash}
    <div class="slash-menu" style="top:{slashPos.top}px; left:{slashPos.left}px;">
      {#if filtered.length === 0}
        <div class="slash-empty">No matching blocks</div>
      {:else}
        {#each filtered as cmd, i (cmd.id)}
          {@const IconComp = (Icons as Record<string, any>)[cmd.icon] ?? Icons.Square}
          <button
            type="button"
            class="slash-item"
            class:active={i === slashIndex}
            onmousedown={(e) => {
              e.preventDefault();
              applyCommand(cmd);
            }}
            onmouseenter={() => (slashIndex = i)}
          >
            <span class="slash-icon"><IconComp size={16} /></span>
            <span class="slash-text">
              <span class="slash-title">{cmd.title}</span>
              <span class="slash-subtitle">{cmd.subtitle}</span>
            </span>
          </button>
        {/each}
      {/if}
    </div>
  {/if}

  {#if showBubble}
    <div class="bubble-menu" style="top:{bubblePos.top}px; left:{bubblePos.left}px;">
      <button type="button" class:active={activeMarks.bold} title="Bold (Ctrl+B)" onmousedown={(e) => { e.preventDefault(); editor?.chain().focus().toggleBold().run(); }}>
        <Icons.Bold size={15} />
      </button>
      <button type="button" class:active={activeMarks.italic} title="Italic (Ctrl+I)" onmousedown={(e) => { e.preventDefault(); editor?.chain().focus().toggleItalic().run(); }}>
        <Icons.Italic size={15} />
      </button>
      <button type="button" class:active={activeMarks.underline} title="Underline" onmousedown={(e) => { e.preventDefault(); editor?.chain().focus().toggleUnderline().run(); }}>
        <Icons.Underline size={15} />
      </button>
      <button type="button" class:active={activeMarks.strike} title="Strikethrough" onmousedown={(e) => { e.preventDefault(); editor?.chain().focus().toggleStrike().run(); }}>
        <Icons.Strikethrough size={15} />
      </button>
      <button type="button" class:active={activeMarks.code} title="Inline code" onmousedown={(e) => { e.preventDefault(); editor?.chain().focus().toggleCode().run(); }}>
        <Icons.Code size={15} />
      </button>
      <span class="bubble-divider"></span>
      <button type="button" class:active={activeMarks.link} title="Link" onmousedown={(e) => { e.preventDefault(); toggleLink(); }}>
        <Icons.Link size={15} />
      </button>
    </div>
  {/if}
</div>

<style>
  .notion-editor {
    position: relative;
    width: 100%;
    min-height: 100%;
  }

  :global(.notion-editor .tiptap-content) {
    outline: none;
    color: var(--text);
    font-size: 16px;
    line-height: 1.7;
    font-family: var(--font-sans, system-ui, sans-serif);
    caret-color: var(--accent, #6ea8fe);
  }

  :global(.notion-editor .tiptap-content > * + *) {
    margin-top: 6px;
  }

  :global(.notion-editor .tiptap-content h1) {
    font-size: 1.9em;
    font-weight: 700;
    line-height: 1.25;
    margin: 24px 0 4px;
    letter-spacing: -0.02em;
  }
  :global(.notion-editor .tiptap-content h2) {
    font-size: 1.45em;
    font-weight: 650;
    margin: 20px 0 2px;
    letter-spacing: -0.01em;
  }
  :global(.notion-editor .tiptap-content h3) {
    font-size: 1.18em;
    font-weight: 600;
    margin: 16px 0 2px;
  }

  :global(.notion-editor .tiptap-content p) {
    margin: 0;
  }

  :global(.notion-editor .tiptap-content ul),
  :global(.notion-editor .tiptap-content ol) {
    padding-left: 1.4em;
    margin: 4px 0;
  }
  :global(.notion-editor .tiptap-content li) {
    margin: 2px 0;
  }
  :global(.notion-editor .tiptap-content li p) {
    margin: 0;
  }

  :global(.notion-editor .tiptap-content ul[data-type='taskList']) {
    list-style: none;
    padding-left: 0.2em;
  }
  :global(.notion-editor .tiptap-content ul[data-type='taskList'] li) {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }
  :global(.notion-editor .tiptap-content ul[data-type='taskList'] li > label) {
    margin-top: 4px;
    user-select: none;
  }
  :global(.notion-editor .tiptap-content ul[data-type='taskList'] input[type='checkbox']) {
    width: 16px;
    height: 16px;
    accent-color: var(--accent, #6ea8fe);
    cursor: pointer;
  }
  :global(.notion-editor .tiptap-content ul[data-type='taskList'] li[data-checked='true'] > div) {
    color: var(--muted);
    text-decoration: line-through;
  }

  :global(.notion-editor .tiptap-content blockquote) {
    border-left: 3px solid var(--accent, #6ea8fe);
    padding-left: 14px;
    margin: 8px 0;
    color: var(--text-2, #aab);
  }

  :global(.notion-editor .tiptap-content pre) {
    background: var(--surface-2, #15171e);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 14px 16px;
    margin: 8px 0;
    overflow-x: auto;
    font-family: 'SFMono-Regular', ui-monospace, Menlo, monospace;
    font-size: 13.5px;
    line-height: 1.55;
  }
  :global(.notion-editor .tiptap-content pre code) {
    background: none;
    padding: 0;
    font-size: inherit;
  }
  :global(.notion-editor .tiptap-content code) {
    background: var(--surface-2, #15171e);
    border: 1px solid var(--border);
    border-radius: 5px;
    padding: 1px 5px;
    font-family: 'SFMono-Regular', ui-monospace, Menlo, monospace;
    font-size: 0.88em;
  }

  :global(.notion-editor .tiptap-content table) {
    border-collapse: collapse;
    margin: 10px 0;
    width: 100%;
    table-layout: fixed;
    overflow: hidden;
  }
  :global(.notion-editor .tiptap-content th),
  :global(.notion-editor .tiptap-content td) {
    border: 1px solid var(--border);
    padding: 7px 10px;
    vertical-align: top;
    text-align: left;
    min-width: 60px;
  }
  :global(.notion-editor .tiptap-content th) {
    background: var(--surface-2, #15171e);
    font-weight: 600;
  }
  :global(.notion-editor .tiptap-content .selectedCell::after) {
    content: '';
    position: absolute;
    inset: 0;
    background: rgba(110, 168, 254, 0.18);
    pointer-events: none;
  }

  :global(.notion-editor .tiptap-content hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 18px 0;
  }

  :global(.notion-editor .tiptap-content a) {
    color: var(--accent, #6ea8fe);
    text-decoration: underline;
    text-underline-offset: 2px;
    cursor: pointer;
  }

  /* placeholder for empty document */
  :global(.notion-editor .tiptap-content p.is-editor-empty:first-child::before),
  :global(.notion-editor .tiptap-content .is-empty::before) {
    content: attr(data-placeholder);
    color: var(--muted);
    float: left;
    height: 0;
    pointer-events: none;
  }

  .slash-menu {
    position: absolute;
    z-index: 200;
    width: 280px;
    max-height: 320px;
    overflow-y: auto;
    background: var(--surface, #14181f);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.45);
    padding: 6px;
  }

  .slash-empty {
    padding: 12px;
    color: var(--muted);
    font-size: 13px;
    text-align: center;
  }

  .slash-item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 8px 10px;
    border: none;
    background: transparent;
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    color: var(--text);
  }
  .slash-item.active {
    background: var(--surface-2, rgba(110, 168, 254, 0.12));
  }
  .slash-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: 8px;
    background: var(--surface-2, #1b1f29);
    border: 1px solid var(--border);
    color: var(--text-2);
    flex-shrink: 0;
  }
  .slash-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .slash-title {
    font-size: 13.5px;
    font-weight: 550;
  }
  .slash-subtitle {
    font-size: 11.5px;
    color: var(--muted);
  }

  .bubble-menu {
    position: absolute;
    z-index: 200;
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px;
    background: var(--surface, #14181f);
    border: 1px solid var(--border);
    border-radius: 10px;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.45);
  }
  .bubble-menu button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border: none;
    background: transparent;
    border-radius: 7px;
    color: var(--text-2);
    cursor: pointer;
  }
  .bubble-menu button:hover {
    background: var(--surface-2, #1b1f29);
    color: var(--text);
  }
  .bubble-menu button.active {
    background: var(--accent, #6ea8fe);
    color: #07131f;
  }
  .bubble-divider {
    width: 1px;
    height: 20px;
    background: var(--border);
    margin: 0 2px;
  }
</style>
