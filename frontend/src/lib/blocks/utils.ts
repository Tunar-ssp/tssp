/**
 * Utility functions for block operations.
 * Handles serialization, deserialization, and block transformations.
 */

import { v4 as uuidv4 } from 'uuid';
import type { Block, BlockCreateOptions, BlockType, ChecklistBlock, CalloutBlock, CodeBlock, TypedBlock } from './types';

/**
 * Generate unique ID for a block
 */
export function generateBlockId(): string {
  return uuidv4().slice(0, 8);
}

/**
 * Create a new block with sensible defaults
 */
export function createBlock(options: BlockCreateOptions, id: string = generateBlockId()): TypedBlock {
  const base: Block = {
    id,
    type: options.type,
    content: options.content || '',
    children: [],
  };

  switch (options.type) {
    case 'checklist':
      return {
        ...base,
        type: 'checklist',
        checked: options.checked || false,
      } as ChecklistBlock;

    case 'callout':
      return {
        ...base,
        type: 'callout',
        color: options.color || 'blue',
        icon: '💡',
      } as CalloutBlock;

    case 'code':
      return {
        ...base,
        type: 'code',
        language: options.language || 'javascript',
      } as CodeBlock;

    default:
      return base as Block;
  }
}

/**
 * Serialize blocks to JSON string (stored in note body)
 */
export function serializeBlocks(blocks: Block[]): string {
  return JSON.stringify(blocks, null, 2);
}

/**
 * Deserialize blocks from JSON string
 */
export function deserializeBlocks(json: string): Block[] {
  try {
    const parsed = JSON.parse(json);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

/**
 * Convert plain text markdown to blocks
 * Handles basic markdown syntax detection
 */
export function markdownToBlocks(markdown: string): Block[] {
  const lines = markdown.split('\n');
  const blocks: Block[] = [];
  let currentListItems: Block[] = [];
  let currentListType: 'bulleted_list' | 'numbered_list' | null = null;

  for (const line of lines) {
    const trimmed = line.trim();

    // Skip empty lines
    if (!trimmed) {
      if (currentListItems.length > 0) {
        blocks.push(...currentListItems);
        currentListItems = [];
        currentListType = null;
      }
      continue;
    }

    // Heading detection
    const headingMatch = trimmed.match(/^(#{1,3})\s+(.+)$/);
    if (headingMatch) {
      if (currentListItems.length > 0) {
        blocks.push(...currentListItems);
        currentListItems = [];
        currentListType = null;
      }
      const level = headingMatch[1].length as 1 | 2 | 3;
      blocks.push(
        createBlock({
          type: `heading_${level}`,
          content: headingMatch[2],
        })
      );
      continue;
    }

    // Bullet list detection
    const bulletMatch = trimmed.match(/^[-*]\s+(.+)$/);
    if (bulletMatch) {
      if (currentListType !== 'bulleted_list') {
        if (currentListItems.length > 0) {
          blocks.push(...currentListItems);
          currentListItems = [];
        }
        currentListType = 'bulleted_list';
      }
      currentListItems.push(
        createBlock({
          type: 'bulleted_list',
          content: bulletMatch[1],
        })
      );
      continue;
    }

    // Numbered list detection
    const numberedMatch = trimmed.match(/^\d+\.\s+(.+)$/);
    if (numberedMatch) {
      if (currentListType !== 'numbered_list') {
        if (currentListItems.length > 0) {
          blocks.push(...currentListItems);
          currentListItems = [];
        }
        currentListType = 'numbered_list';
      }
      currentListItems.push(
        createBlock({
          type: 'numbered_list',
          content: numberedMatch[1],
        })
      );
      continue;
    }

    // Quote detection
    const quoteMatch = trimmed.match(/^>\s+(.+)$/);
    if (quoteMatch) {
      if (currentListItems.length > 0) {
        blocks.push(...currentListItems);
        currentListItems = [];
        currentListType = null;
      }
      blocks.push(
        createBlock({
          type: 'quote',
          content: quoteMatch[1],
        })
      );
      continue;
    }

    // Default: paragraph
    if (currentListItems.length > 0) {
      blocks.push(...currentListItems);
      currentListItems = [];
      currentListType = null;
    }
    blocks.push(
      createBlock({
        type: 'paragraph',
        content: trimmed,
      })
    );
  }

  // Flush remaining list items
  if (currentListItems.length > 0) {
    blocks.push(...currentListItems);
  }

  return blocks.length > 0 ? blocks : [createBlock({ type: 'paragraph', content: '' })];
}

/**
 * Convert blocks back to plain text (for saving/export)
 */
export function blocksToMarkdown(blocks: Block[]): string {
  return blocks
    .map((block) => {
      switch (block.type) {
        case 'heading_1':
          return `# ${block.content}`;
        case 'heading_2':
          return `## ${block.content}`;
        case 'heading_3':
          return `### ${block.content}`;
        case 'bulleted_list':
          return `- ${block.content}`;
        case 'numbered_list':
          return `1. ${block.content}`;
        case 'checklist':
          return `- [${(block as ChecklistBlock).checked ? 'x' : ' '}] ${block.content}`;
        case 'quote':
          return `> ${block.content}`;
        case 'callout':
          return `💡 ${block.content}`;
        case 'code':
          return `\`\`\`\n${block.content}\n\`\`\``;
        case 'divider':
          return '---';
        default:
          return block.content;
      }
    })
    .join('\n');
}

/**
 * Find a block by ID recursively
 */
export function findBlockById(blocks: Block[], id: string): Block | null {
  for (const block of blocks) {
    if (block.id === id) return block;
    if (block.children) {
      const found = findBlockById(block.children, id);
      if (found) return found;
    }
  }
  return null;
}

/**
 * Find parent block and index of a child block
 */
export function findBlockParent(
  blocks: Block[],
  childId: string
): { parent: Block; index: number } | null {
  for (const block of blocks) {
    if (block.children) {
      const index = block.children.findIndex((b) => b.id === childId);
      if (index !== -1) {
        return { parent: block, index };
      }
      const found = findBlockParent(block.children, childId);
      if (found) return found;
    }
  }
  return null;
}

/**
 * Detect block type from markdown syntax
 */
export function detectBlockType(text: string): BlockType {
  const trimmed = text.trim();

  if (/^#\s/.test(trimmed)) return 'heading_1';
  if (/^##\s/.test(trimmed)) return 'heading_2';
  if (/^###\s/.test(trimmed)) return 'heading_3';
  if (/^[-*]\s/.test(trimmed)) return 'bulleted_list';
  if (/^\d+\.\s/.test(trimmed)) return 'numbered_list';
  if (/^>\s/.test(trimmed)) return 'quote';
  if (/^-\s\[\s*\]/.test(trimmed)) return 'checklist';

  return 'paragraph';
}

/**
 * Extract markdown prefix from text (e.g., "# " from "# Heading")
 */
export function extractMarkdownPrefix(text: string): string {
  const match = text.match(/^(#{1,3}|[-*]|>\s|`{3}|\d+\.\s|-\s\[\s*\])\s+/);
  return match ? match[1] : '';
}
