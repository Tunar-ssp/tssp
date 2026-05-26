/**
 * Slash command system for block insertion.
 * Provides the / menu with all available block types.
 */

import type { SlashCommand, BlockCreateOptions } from './types';

export interface CommandMetadata {
  category: 'text' | 'lists' | 'special';
  shortcuts?: string[];
  popular?: boolean;
  keywords?: string[];
}

export interface ExtendedSlashCommand extends SlashCommand {
  metadata?: CommandMetadata;
}

export const slashCommands: ExtendedSlashCommand[] = [
  // Text blocks
  {
    name: 'paragraph',
    label: 'Paragraph',
    description: 'Regular text block',
    icon: '¶',
    metadata: {
      category: 'text',
      popular: true,
      keywords: ['text', 'body', 'content'],
    },
    action: () => ({
      type: 'paragraph',
      content: '',
    }),
  },
  {
    name: 'heading_1',
    label: 'Heading 1',
    description: 'Large section heading',
    icon: 'H1',
    metadata: {
      category: 'text',
      shortcuts: ['#'],
      popular: true,
      keywords: ['h1', 'title', 'section', 'heading'],
    },
    action: () => ({
      type: 'heading_1',
      content: '',
    }),
  },
  {
    name: 'heading_2',
    label: 'Heading 2',
    description: 'Medium heading',
    icon: 'H2',
    metadata: {
      category: 'text',
      shortcuts: ['##'],
      popular: true,
      keywords: ['h2', 'heading', 'subsection'],
    },
    action: () => ({
      type: 'heading_2',
      content: '',
    }),
  },
  {
    name: 'heading_3',
    label: 'Heading 3',
    description: 'Small heading',
    icon: 'H3',
    metadata: {
      category: 'text',
      shortcuts: ['###'],
      keywords: ['h3', 'heading', 'subheading'],
    },
    action: () => ({
      type: 'heading_3',
      content: '',
    }),
  },

  // Lists
  {
    name: 'bulleted_list',
    label: 'Bullet List',
    description: 'Unordered list item',
    icon: '•',
    metadata: {
      category: 'lists',
      shortcuts: ['-', '*'],
      popular: true,
      keywords: ['bullet', 'list', 'unordered', 'item'],
    },
    action: () => ({
      type: 'bulleted_list',
      content: '',
    }),
  },
  {
    name: 'numbered_list',
    label: 'Numbered List',
    description: 'Ordered list item',
    icon: '1.',
    metadata: {
      category: 'lists',
      shortcuts: ['1.'],
      popular: true,
      keywords: ['number', 'list', 'ordered', 'item'],
    },
    action: () => ({
      type: 'numbered_list',
      content: '',
    }),
  },
  {
    name: 'checklist',
    label: 'Checklist',
    description: 'Checkbox item',
    icon: '☑',
    metadata: {
      category: 'lists',
      shortcuts: ['[]'],
      keywords: ['check', 'checkbox', 'task', 'todo'],
    },
    action: () => ({
      type: 'checklist',
      content: '',
      checked: false,
    }),
  },

  // Special blocks
  {
    name: 'quote',
    label: 'Quote',
    description: 'Quoted text block',
    icon: '"',
    metadata: {
      category: 'special',
      shortcuts: ['>'],
      keywords: ['quote', 'citation', 'reference'],
    },
    action: () => ({
      type: 'quote',
      content: '',
    }),
  },
  {
    name: 'callout',
    label: 'Callout',
    description: 'Highlighted info block',
    icon: '💡',
    metadata: {
      category: 'special',
      keywords: ['callout', 'highlight', 'note', 'important'],
    },
    action: () => ({
      type: 'callout',
      content: '',
      color: 'blue',
    }),
  },
  {
    name: 'code',
    label: 'Code Block',
    description: 'Code snippet',
    icon: '{ }',
    metadata: {
      category: 'special',
      shortcuts: ['```'],
      keywords: ['code', 'snippet', 'programming', 'syntax'],
    },
    action: () => ({
      type: 'code',
      content: '',
      language: 'javascript',
    }),
  },
  {
    name: 'divider',
    label: 'Divider',
    description: 'Horizontal line separator',
    icon: '—',
    metadata: {
      category: 'special',
      shortcuts: ['---'],
      keywords: ['divider', 'line', 'separator', 'break'],
    },
    action: () => ({
      type: 'divider',
      content: '',
    }),
  },
];

/**
 * Filter commands by search query with relevance scoring
 */
export function filterCommands(query: string): ExtendedSlashCommand[] {
  if (!query.trim()) {
    // Return in category order when no search
    return slashCommands.sort(
      (a, b) =>
        slashCommands.indexOf(a) - slashCommands.indexOf(b)
    );
  }

  const q = query.toLowerCase().trim();
  const scored = slashCommands
    .map((cmd) => {
      let score = 0;

      // Exact match is highest
      if (cmd.name === q) score += 1000;

      // Starts with query
      if (cmd.label.toLowerCase().startsWith(q)) score += 500;

      // Contains in label
      if (cmd.label.toLowerCase().includes(q)) score += 300;

      // Keyboard shortcuts match
      if (cmd.metadata?.shortcuts?.some(s => s.includes(q))) score += 250;

      // Keywords match
      if (cmd.metadata?.keywords?.some(k => k.includes(q))) score += 200;

      // Description match
      if (cmd.description.toLowerCase().includes(q)) score += 100;

      // Popular commands get slight boost
      if (cmd.metadata?.popular) score += 10;

      return { cmd, score };
    })
    .filter(({ score }) => score > 0)
    .sort((a, b) => b.score - a.score)
    .map(({ cmd }) => cmd);

  return scored;
}

/**
 * Find a command by name
 */
export function findCommand(name: string): ExtendedSlashCommand | undefined {
  return slashCommands.find((cmd) => cmd.name === name);
}

/**
 * Group commands by category (for better organization in UI)
 */
export function groupCommandsByCategory() {
  return {
    text: slashCommands.filter(cmd => cmd.metadata?.category === 'text'),
    lists: slashCommands.filter(cmd => cmd.metadata?.category === 'lists'),
    special: slashCommands.filter(cmd => cmd.metadata?.category === 'special'),
  };
}

/**
 * Get all commands grouped with category headers
 */
export function getCommandsWithCategories(): Array<
  { type: 'header'; label: string; category: string } | { type: 'command'; command: ExtendedSlashCommand }
> {
  const grouped = groupCommandsByCategory();
  const result: Array<
    { type: 'header'; label: string; category: string } | { type: 'command'; command: ExtendedSlashCommand }
  > = [];

  if (grouped.text.length > 0) {
    result.push({ type: 'header', label: 'Text', category: 'text' });
    grouped.text.forEach(cmd => result.push({ type: 'command', command: cmd }));
  }

  if (grouped.lists.length > 0) {
    result.push({ type: 'header', label: 'Lists', category: 'lists' });
    grouped.lists.forEach(cmd => result.push({ type: 'command', command: cmd }));
  }

  if (grouped.special.length > 0) {
    result.push({ type: 'header', label: 'Special', category: 'special' });
    grouped.special.forEach(cmd => result.push({ type: 'command', command: cmd }));
  }

  return result;
}
