/**
 * Note templates - pre-seeded starting structures for common note types
 */

import { createBlock } from './utils';
import type { Block } from './types';

export interface NoteTemplate {
  name: string;
  description: string;
  icon: string;
  blocks: Block[];
}

export const noteTemplates: Record<string, NoteTemplate> = {
  daily_log: {
    name: 'Daily Log',
    description: 'Structured daily notes with sections',
    icon: '📅',
    blocks: [
      createBlock({ type: 'heading_1', content: 'Daily Log - [Date]' }),
      createBlock({ type: 'heading_2', content: 'Today\'s Goals' }),
      createBlock({ type: 'numbered_list', content: '' }),
      createBlock({ type: 'heading_2', content: 'Completed' }),
      createBlock({ type: 'checklist', content: 'Item 1', checked: true }),
      createBlock({ type: 'heading_2', content: 'Notes' }),
      createBlock({ type: 'paragraph', content: '' }),
      createBlock({ type: 'heading_2', content: 'Tomorrow' }),
      createBlock({ type: 'bulleted_list', content: '' }),
    ],
  },

  meeting_minutes: {
    name: 'Meeting Minutes',
    description: 'Capture meeting details and action items',
    icon: '👥',
    blocks: [
      createBlock({ type: 'heading_1', content: 'Meeting: [Topic]' }),
      createBlock({ type: 'paragraph', content: 'Date: [Date] | Attendees: [Names]' }),
      createBlock({ type: 'heading_2', content: 'Agenda' }),
      createBlock({ type: 'numbered_list', content: '' }),
      createBlock({ type: 'heading_2', content: 'Discussion' }),
      createBlock({ type: 'paragraph', content: '' }),
      createBlock({ type: 'heading_2', content: 'Action Items' }),
      createBlock({
        type: 'checklist',
        content: '[Action] - Owner: [Person]',
        checked: false,
      }),
      createBlock({ type: 'heading_2', content: 'Next Steps' }),
      createBlock({ type: 'bulleted_list', content: '' }),
    ],
  },

  project_brief: {
    name: 'Project Brief',
    description: 'Project overview and planning document',
    icon: '🎯',
    blocks: [
      createBlock({ type: 'heading_1', content: 'Project: [Name]' }),
      createBlock({ type: 'heading_2', content: 'Objective' }),
      createBlock({ type: 'paragraph', content: '' }),
      createBlock({ type: 'heading_2', content: 'Scope' }),
      createBlock({ type: 'bulleted_list', content: 'In Scope:' }),
      createBlock({ type: 'bulleted_list', content: 'Out of Scope:' }),
      createBlock({ type: 'heading_2', content: 'Timeline' }),
      createBlock({ type: 'paragraph', content: 'Start: [Date] | End: [Date]' }),
      createBlock({ type: 'heading_2', content: 'Team' }),
      createBlock({ type: 'paragraph', content: 'Lead: [Name] | Team: [Members]' }),
      createBlock({ type: 'heading_2', content: 'Resources' }),
      createBlock({ type: 'bulleted_list', content: '' }),
      createBlock({ type: 'heading_2', content: 'Risks & Mitigation' }),
      createBlock({ type: 'paragraph', content: '' }),
    ],
  },

  reading_notes: {
    name: 'Reading Notes',
    description: 'Book or article notes with key takeaways',
    icon: '📚',
    blocks: [
      createBlock({ type: 'heading_1', content: 'Title: [Book/Article Name]' }),
      createBlock({ type: 'paragraph', content: 'Author: [Name] | Date Read: [Date]' }),
      createBlock({ type: 'heading_2', content: 'Summary' }),
      createBlock({ type: 'paragraph', content: '' }),
      createBlock({ type: 'heading_2', content: 'Key Takeaways' }),
      createBlock({ type: 'numbered_list', content: '' }),
      createBlock({ type: 'heading_2', content: 'Quotes' }),
      createBlock({ type: 'quote', content: '[Quote here]' }),
      createBlock({ type: 'heading_2', content: 'Reflection' }),
      createBlock({ type: 'paragraph', content: '' }),
    ],
  },

  idea_capture: {
    name: 'Idea Capture',
    description: 'Quick brainstorm and idea collection',
    icon: '💡',
    blocks: [
      createBlock({ type: 'heading_1', content: 'Idea: [Title]' }),
      createBlock({ type: 'callout', content: '[Main concept]', color: 'blue' }),
      createBlock({ type: 'heading_2', content: 'Details' }),
      createBlock({ type: 'paragraph', content: '' }),
      createBlock({ type: 'heading_2', content: 'Why?' }),
      createBlock({ type: 'paragraph', content: '' }),
      createBlock({ type: 'heading_2', content: 'Next Steps' }),
      createBlock({ type: 'bulleted_list', content: '' }),
      createBlock({ type: 'heading_2', content: 'Related Ideas' }),
      createBlock({ type: 'bulleted_list', content: '' }),
    ],
  },

  decision_log: {
    name: 'Decision Log',
    description: 'Document decisions and reasoning',
    icon: '⚖️',
    blocks: [
      createBlock({ type: 'heading_1', content: 'Decision: [What?]' }),
      createBlock({ type: 'paragraph', content: 'Date: [Date] | By: [Person]' }),
      createBlock({ type: 'heading_2', content: 'Context' }),
      createBlock({ type: 'paragraph', content: '' }),
      createBlock({ type: 'heading_2', content: 'Options Considered' }),
      createBlock({ type: 'numbered_list', content: '' }),
      createBlock({ type: 'heading_2', content: 'Decision' }),
      createBlock({ type: 'callout', content: '[Decision summary]', color: 'green' }),
      createBlock({ type: 'heading_2', content: 'Rationale' }),
      createBlock({ type: 'bulleted_list', content: '' }),
      createBlock({ type: 'heading_2', content: 'Implications' }),
      createBlock({ type: 'paragraph', content: '' }),
    ],
  },
};

/**
 * Get template by ID
 */
export function getTemplate(id: string): NoteTemplate | undefined {
  return noteTemplates[id];
}

/**
 * List all available templates
 */
export function listTemplates(): Array<[string, NoteTemplate]> {
  return Object.entries(noteTemplates);
}
