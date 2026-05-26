/**
 * Slash command system for block insertion.
 * Provides the / menu with all available block types.
 */

import type { SlashCommand, BlockCreateOptions } from './types';

export const slashCommands: SlashCommand[] = [
  // Text blocks
  {
    name: 'paragraph',
    label: 'Paragraph',
    description: 'Regular text block',
    icon: '¶',
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
    action: () => ({
      type: 'code',
      content: '',
      language: 'javascript',
    }),
  },
  {
    name: 'divider',
    label: 'Divider',
    description: 'Horizontal line',
    icon: '—',
    action: () => ({
      type: 'divider',
      content: '',
    }),
  },
];

/**
 * Filter commands by search query
 */
export function filterCommands(query: string): SlashCommand[] {
  if (!query.trim()) return slashCommands;

  const q = query.toLowerCase();
  return slashCommands.filter(
    (cmd) =>
      cmd.label.toLowerCase().includes(q) ||
      cmd.description.toLowerCase().includes(q) ||
      cmd.name.includes(q)
  );
}

/**
 * Find a command by name
 */
export function findCommand(name: string): SlashCommand | undefined {
  return slashCommands.find((cmd) => cmd.name === name);
}

/**
 * Group commands by category (for better organization in UI)
 */
export function groupCommands() {
  return {
    text: slashCommands.slice(0, 4),
    lists: slashCommands.slice(4, 7),
    special: slashCommands.slice(7),
  };
}
