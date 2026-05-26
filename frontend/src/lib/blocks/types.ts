/**
 * Block system for Notion-like note editor.
 * Clean abstraction for polymorphic block types.
 */

export type BlockType =
  | 'paragraph'
  | 'heading_1'
  | 'heading_2'
  | 'heading_3'
  | 'bulleted_list'
  | 'numbered_list'
  | 'checklist'
  | 'quote'
  | 'callout'
  | 'code'
  | 'divider';

export type CalloutColor = 'blue' | 'red' | 'yellow' | 'green' | 'purple' | 'gray';

/**
 * Base block structure - all blocks have these properties
 */
export interface Block {
  id: string;
  type: BlockType;
  content: string;
  children?: Block[];
  metadata?: Record<string, unknown>;
}

/**
 * Extended block with type-specific properties
 */
export interface ChecklistBlock extends Block {
  type: 'checklist';
  checked: boolean;
}

export interface CalloutBlock extends Block {
  type: 'callout';
  color: CalloutColor;
  icon?: string;
}

export interface CodeBlock extends Block {
  type: 'code';
  language?: string;
}

export type TypedBlock = Block | ChecklistBlock | CalloutBlock | CodeBlock;

/**
 * Block creation options
 */
export interface BlockCreateOptions {
  type: BlockType;
  content?: string;
  checked?: boolean;
  color?: CalloutColor;
  language?: string;
}

/**
 * Slash command definition
 */
export interface SlashCommand {
  name: string;
  label: string;
  description: string;
  icon: string;
  action: (blockId: string) => BlockCreateOptions;
}

/**
 * Editor state for a single note
 */
export interface EditorState {
  blocks: Block[];
  selection: {
    blockId: string;
    offset: number;
  } | null;
  isDirty: boolean;
}
