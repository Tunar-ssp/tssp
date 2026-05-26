<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import NotesOutline from './NotesOutline.svelte';
  import NotesMeta from './NotesMeta.svelte';

  interface $$Props {
    tab: 'preview' | 'outline' | 'meta';
    previewHtml: string;
    content: string;
    createdAt: number;
    updatedAt: number;
    wordCount: number;
    blockCount: number;
    onTabChange: (tab: 'preview' | 'outline' | 'meta') => void;
  }

  let { tab, previewHtml, content, createdAt, updatedAt, wordCount, blockCount, onTabChange }: $$Props = $props();
</script>

<aside class="note-inspector">
  <div class="inspector-tabs">
    <button type="button" class:active={tab === 'preview'} onclick={() => onTabChange('preview')}>
      Preview
    </button>
    <button type="button" class:active={tab === 'outline'} onclick={() => onTabChange('outline')}>
      Outline
    </button>
    <button type="button" class:active={tab === 'meta'} onclick={() => onTabChange('meta')}>
      Meta
    </button>
  </div>

  {#if tab === 'preview'}
    <div class="preview-pane">
      {#if content.trim()}
        <div class="markdown-preview">
          {@html previewHtml}
        </div>
      {:else}
        <div class="inspector-empty">
          <Icons.PanelRightOpen size={18} />
          <p>Preview updates as you write markdown-style content.</p>
        </div>
      {/if}
    </div>
  {:else if tab === 'outline'}
    <NotesOutline {content} onSelectItem={() => {}} />
  {:else}
    <NotesMeta {createdAt} {updatedAt} {wordCount} {blockCount} />
  {/if}
</aside>

<style>
  .note-inspector {
    min-height: 0;
    border-left: 1px solid var(--border);
    background: rgba(14, 16, 22, 0.98);
    display: flex;
    flex-direction: column;
  }

  .inspector-tabs {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    padding: 12px;
    border-bottom: 1px solid var(--border);
    gap: 8px;
  }

  .inspector-tabs button {
    height: 40px;
    border-radius: 14px;
    background: transparent;
    color: var(--text-2);
    border: none;
    cursor: pointer;
  }

  .inspector-tabs button.active {
    background: rgba(34, 42, 58, 0.95);
    color: var(--text);
  }

  .preview-pane {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 18px;
  }

  .markdown-preview {
    color: var(--text);
    line-height: 1.7;
  }

  .markdown-preview :global(h1),
  .markdown-preview :global(h2),
  .markdown-preview :global(h3),
  .markdown-preview :global(h4),
  .markdown-preview :global(h5),
  .markdown-preview :global(h6) {
    margin: 0 0 12px;
    line-height: 1.05;
    letter-spacing: -0.03em;
  }

  .markdown-preview :global(h1) {
    font-size: 42px;
  }

  .markdown-preview :global(h2) {
    font-size: 30px;
  }

  .markdown-preview :global(p),
  .markdown-preview :global(ul),
  .markdown-preview :global(ol),
  .markdown-preview :global(blockquote),
  .markdown-preview :global(pre) {
    margin: 0 0 16px;
  }

  .markdown-preview :global(blockquote) {
    margin-left: 0;
    padding: 12px 14px;
    border-left: 3px solid var(--green);
    background: rgba(17, 45, 32, 0.48);
    border-radius: 12px;
  }

  .markdown-preview :global(pre) {
    overflow: auto;
    padding: 14px;
    border-radius: 16px;
    background: rgba(12, 14, 20, 0.98);
    border: 1px solid var(--border);
  }

  .markdown-preview :global(code) {
    font-family: var(--ff-mono);
  }

  .markdown-preview :global(.task-list) {
    list-style: none;
    padding-left: 0;
  }

  .markdown-preview :global(.task-item) {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .markdown-preview :global(a) {
    color: var(--blue);
  }

  .inspector-empty {
    min-height: 220px;
    border-radius: 26px;
    border: 1px dashed var(--border);
    background: rgba(14, 16, 22, 0.82);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
    text-align: center;
    padding: 20px;
  }

  @media (max-width: 1400px) {
    .note-inspector {
      border-left: 0;
      border-top: 1px solid var(--border);
      min-height: 320px;
    }
  }
</style>
