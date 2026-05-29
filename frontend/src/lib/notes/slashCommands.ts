/**
 * Slash command catalogue for the Notes editor.
 *
 * Each command knows how to transform the current selection via the Tiptap
 * editor chain. The editor component handles detection, filtering, and the
 * menu UI; this module is the single source of truth for what commands exist.
 */
import type { Editor } from '@tiptap/core';

export interface SlashCommand {
  id: string;
  title: string;
  subtitle: string;
  /** lucide-svelte icon name */
  icon: string;
  keywords: string[];
  /** Run the command against the editor at the given range (slash text removed). */
  run: (editor: Editor) => void;
}

export const SLASH_COMMANDS: SlashCommand[] = [
  {
    id: 'text',
    title: 'Text',
    subtitle: 'Plain paragraph',
    icon: 'Type',
    keywords: ['text', 'paragraph', 'plain', 'body'],
    run: (editor) => editor.chain().focus().setParagraph().run(),
  },
  {
    id: 'h1',
    title: 'Heading 1',
    subtitle: 'Large section heading',
    icon: 'Heading1',
    keywords: ['heading', 'h1', 'title', 'big'],
    run: (editor) => editor.chain().focus().toggleHeading({ level: 1 }).run(),
  },
  {
    id: 'h2',
    title: 'Heading 2',
    subtitle: 'Medium section heading',
    icon: 'Heading2',
    keywords: ['heading', 'h2', 'subtitle'],
    run: (editor) => editor.chain().focus().toggleHeading({ level: 2 }).run(),
  },
  {
    id: 'h3',
    title: 'Heading 3',
    subtitle: 'Small section heading',
    icon: 'Heading3',
    keywords: ['heading', 'h3'],
    run: (editor) => editor.chain().focus().toggleHeading({ level: 3 }).run(),
  },
  {
    id: 'bullet',
    title: 'Bulleted list',
    subtitle: 'Simple bulleted list',
    icon: 'List',
    keywords: ['bullet', 'unordered', 'list', 'ul'],
    run: (editor) => editor.chain().focus().toggleBulletList().run(),
  },
  {
    id: 'ordered',
    title: 'Numbered list',
    subtitle: 'List with numbering',
    icon: 'ListOrdered',
    keywords: ['number', 'ordered', 'list', 'ol'],
    run: (editor) => editor.chain().focus().toggleOrderedList().run(),
  },
  {
    id: 'task',
    title: 'To-do list',
    subtitle: 'Track tasks with checkboxes',
    icon: 'ListChecks',
    keywords: ['todo', 'task', 'checkbox', 'check'],
    run: (editor) => editor.chain().focus().toggleTaskList().run(),
  },
  {
    id: 'quote',
    title: 'Quote',
    subtitle: 'Capture a quotation',
    icon: 'Quote',
    keywords: ['quote', 'blockquote', 'citation'],
    run: (editor) => editor.chain().focus().toggleBlockquote().run(),
  },
  {
    id: 'code',
    title: 'Code block',
    subtitle: 'Monospaced code with syntax',
    icon: 'Code2',
    keywords: ['code', 'snippet', 'pre', 'monospace'],
    run: (editor) => editor.chain().focus().toggleCodeBlock().run(),
  },
  {
    id: 'table',
    title: 'Table',
    subtitle: 'Insert a 3×3 table',
    icon: 'Table',
    keywords: ['table', 'grid', 'rows', 'columns'],
    run: (editor) =>
      editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
  },
  {
    id: 'divider',
    title: 'Divider',
    subtitle: 'Visual separator',
    icon: 'Minus',
    keywords: ['divider', 'hr', 'rule', 'separator', 'line'],
    run: (editor) => editor.chain().focus().setHorizontalRule().run(),
  },
];

/** Filter commands by a slash query (matches title and keywords). */
export function filterSlashCommands(query: string): SlashCommand[] {
  const q = query.trim().toLowerCase();
  if (!q) return SLASH_COMMANDS;
  return SLASH_COMMANDS.filter(
    (cmd) =>
      cmd.title.toLowerCase().includes(q) ||
      cmd.keywords.some((keyword) => keyword.includes(q)),
  );
}
