<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import Outline from '$lib/components/Outline.svelte';

  interface $$Props {
    tab: 'preview' | 'outline' | 'terminal';
    previewHtml: string;
    content: string;
    selectedLanguage: string;
    onTabChange: (tab: 'preview' | 'outline' | 'terminal') => void;
  }

  let {
    tab,
    previewHtml,
    content,
    selectedLanguage,
    onTabChange,
  }: $$Props = $props();
</script>

<aside class="workspace-inspector">
  <div class="inspector-tabs">
    <button type="button" class:active={tab === 'preview'} onclick={() => onTabChange('preview')}>
      Preview
    </button>
    <button type="button" class:active={tab === 'outline'} onclick={() => onTabChange('outline')}>
      Outline
    </button>
    <button type="button" class:active={tab === 'terminal'} onclick={() => onTabChange('terminal')}>
      Terminal
    </button>
  </div>

  {#if tab === 'preview'}
    <div class="preview-panel">
      {#if selectedLanguage === 'markdown'}
        <div class="markdown-preview">{@html previewHtml}</div>
      {:else}
        <div class="plain-preview-card">
          <span class="sidebar-label">Preview</span>
          <p>Rich preview is currently optimized for markdown workspaces. Other languages render as raw text below.</p>
          <pre>{content || '// No content yet'}</pre>
        </div>
      {/if}
    </div>
  {:else if tab === 'outline'}
    <div class="outline-panel">
      <Outline {content} onSelectItem={() => {}} />
    </div>
  {:else}
    <div class="terminal-panel">
      <div class="terminal-warning">
        <Icons.AlertTriangle size={18} />
        <div>
          <strong>Run is sandboxed</strong>
          <p>Execution and terminal access stay locked until the backend exposes a safe sandbox. Admin maintenance tools remain available separately.</p>
        </div>
      </div>
      <div class="terminal-placeholder">
        <code>// scratch: run requires sandbox unlock</code>
      </div>
    </div>
  {/if}
</aside>

<style>
  .workspace-inspector {
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: rgba(14, 16, 22, 0.98);
  }

  .inspector-tabs {
    padding: 12px;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
    border-bottom: 1px solid var(--border);
  }

  .inspector-tabs button {
    height: 40px;
    border-radius: 14px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
  }

  .inspector-tabs button.active {
    background: rgba(34, 42, 58, 0.95);
    color: var(--text);
  }

  .preview-panel,
  .outline-panel,
  .terminal-panel {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .preview-panel {
    padding: 18px;
  }

  .outline-panel :global(.outline) {
    height: 100%;
    border-right: 0;
    background: transparent;
  }

  .markdown-preview {
    color: var(--text);
    line-height: 1.7;
  }

  .markdown-preview :global(h1),
  .markdown-preview :global(h2),
  .markdown-preview :global(h3) {
    margin: 0 0 12px;
    line-height: 1.05;
    letter-spacing: -0.03em;
  }

  .markdown-preview :global(h1) {
    font-size: 34px;
  }

  .markdown-preview :global(h2) {
    font-size: 26px;
  }

  .markdown-preview :global(p),
  .markdown-preview :global(ul),
  .markdown-preview :global(ol),
  .markdown-preview :global(pre),
  .markdown-preview :global(blockquote) {
    margin: 0 0 14px;
  }

  .markdown-preview :global(pre) {
    padding: 14px;
    border-radius: 18px;
    overflow: auto;
    background: rgba(12, 14, 20, 0.98);
    border: 1px solid var(--border);
  }

  .plain-preview-card,
  .terminal-warning,
  .terminal-placeholder {
    padding: 18px;
    border-radius: 20px;
    border: 1px solid var(--border);
    background: rgba(18, 21, 29, 0.98);
  }

  .plain-preview-card p,
  .terminal-warning p {
    color: var(--muted);
    line-height: 1.6;
  }

  .plain-preview-card pre,
  .terminal-placeholder code {
    display: block;
    margin-top: 14px;
    padding: 14px;
    border-radius: 16px;
    background: rgba(12, 14, 20, 0.98);
    border: 1px solid var(--border);
    color: var(--text-2);
    font-family: var(--ff-mono);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .terminal-panel {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .terminal-warning {
    display: flex;
    gap: 12px;
    color: var(--warning);
  }

  .terminal-warning strong {
    display: block;
    color: var(--text);
    margin-bottom: 6px;
  }

  .sidebar-label {
    font-family: var(--ff-mono);
    font-size: 12px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--dim);
  }
</style>
