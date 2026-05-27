/**
 * Notion-like block types for rich note editing
 */

export type BlockType =
  | 'paragraph'
  | 'heading-1'
  | 'heading-2'
  | 'heading-3'
  | 'bulleted-list'
  | 'numbered-list'
  | 'toggle'
  | 'quote'
  | 'code'
  | 'divider'
  | 'callout'
  | 'table';

export interface TextFormatting {
  bold?: boolean;
  italic?: boolean;
  strikethrough?: boolean;
  code?: boolean;
  color?: string;
  backgroundColor?: string;
}

export interface Block {
  id: string;
  type: BlockType;
  content: string;
  children?: Block[];
  formatting?: TextFormatting;
  metadata?: Record<string, any>;
  collapsed?: boolean;
}

export const BLOCK_LABELS: Record<BlockType, string> = {
  'paragraph': 'Paragraph',
  'heading-1': 'Heading 1',
  'heading-2': 'Heading 2',
  'heading-3': 'Heading 3',
  'bulleted-list': 'Bulleted list',
  'numbered-list': 'Numbered list',
  'toggle': 'Toggle list',
  'quote': 'Quote',
  'code': 'Code',
  'divider': 'Divider',
  'callout': 'Callout',
  'table': 'Table',
};

export const SLASH_COMMANDS: Record<string, BlockType> = {
  'text': 'paragraph',
  'h1': 'heading-1',
  'h2': 'heading-2',
  'h3': 'heading-3',
  'bullet': 'bulleted-list',
  'number': 'numbered-list',
  'toggle': 'toggle',
  'quote': 'quote',
  'code': 'code',
  'divider': 'divider',
  'callout': 'callout',
  'table': 'table',
};

export function createBlock(type: BlockType = 'paragraph', content: string = ''): Block {
  return {
    id: Math.random().toString(36).substring(7),
    type,
    content,
    children: [],
    collapsed: false,
  };
}

export function formatBlockContent(block: Block, formatting: TextFormatting): Block {
  return {
    ...block,
    formatting: { ...block.formatting, ...formatting },
  };
}
