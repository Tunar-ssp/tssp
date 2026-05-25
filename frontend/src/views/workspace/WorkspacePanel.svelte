<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import Outline from '$lib/components/Outline.svelte';

  interface $$Props {
    content: string;
    activeTab: 'preview' | 'outline' | 'terminal';
    onTabChange?: (tab: 'preview' | 'outline' | 'terminal') => void;
  }

  let { content, activeTab, onTabChange }: $$Props = $props();

  function renderMarkdownLite(md: string): string {
    if (!md) return '<p></p>';
    return md
      .replace(/^### (.*?)$/gm, '<h3>$1</h3>')
      .replace(/^## (.*?)$/gm, '<h2>$1</h2>')
      .replace(/^# (.*?)$/gm, '<h1>$1</h1>')
      .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.*?)\*/g, '<em>$1</em>')
      .replace(/`(.*?)`/g, '<code>$1</code>')
      .replace(/\n/g, '<br>');
  }
</script>

<aside class="workspace-panel">
  <div class="panel-tabs">
    <button
      type="button"
      class:active={activeTab === 'preview'}
      onclick={() => onTabChange?.('preview')}
    >
      Preview
    </button>
    <button
      type="button"
      class:active={activeTab === 'outline'}
      onclick={() => onTabChange?.('outline')}
    >
      Outline
    </button>
    <button
      type="button"
      class:active={activeTab === 'terminal'}
      onclick={() => onTabChange?.('terminal')}
    >
      Terminal
    </button>
  </div>

  {#if activeTab === 'preview'}
    <div class="panel-content preview-pane">
      {#if content.trim()}
        <div class="markdown-preview">
          {@html renderMarkdownLite(content)}
        </div>
      {:else}
        <div class="panel-empty">
          <Icons.Eye size={24} />
          <p>Preview will show rendered markdown here</p>
        </div>
      {/if}
    </div>
  {:else if activeTab === 'outline'}
    <div class="panel-content outline-pane">
      <Outline {content} onSelectItem={() => {}} />
    </div>
  {:else}
    <div class="panel-content terminal-pane">
      <div class="panel-empty">
        <Icons.Terminal size={24} />
        <p>Terminal integration coming soon</p>
      </div>
    </div>
  {/if}
</aside>

<style>
  .workspace-panel {
    width: 320px;
    display: flex;
    flex-direction: column;
    border-left: 1px solid var(--border);
    background: var(--surface);
  }

  .panel-tabs {
    display: flex;
    gap: 2px;
    padding: 12px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
  }

  .panel-tabs button {
    flex: 1;
    padding: 8px 12px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 12px;
    transition: all 0.2s;
  }

  .panel-tabs button:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .panel-tabs button.active {
    background: rgba(110, 168, 255, 0.14);
    color: var(--blue);
    font-weight: 600;
  }

  .panel-content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 16px;
  }

  .preview-pane {
    font-size: 14px;
    line-height: 1.6;
    color: var(--text);
  }

  .outline-pane {
    font-size: 13px;
  }

  .terminal-pane {
    background: #0a0a0a;
    color: #00ff00;
    font-family: var(--ff-mono);
    font-size: 12px;
  }

  .panel-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--muted);
    gap: 12px;
  }

  .panel-empty p {
    margin: 0;
    font-size: 13px;
  }

  .markdown-preview {
    word-break: break-word;
  }

  :global(.markdown-preview h1) {
    font-size: 24px;
    font-weight: 700;
    margin: 16px 0 8px;
  }

  :global(.markdown-preview h2) {
    font-size: 20px;
    font-weight: 600;
    margin: 12px 0 6px;
  }

  :global(.markdown-preview h3) {
    font-size: 16px;
    font-weight: 600;
    margin: 10px 0 4px;
  }

  :global(.markdown-preview code) {
    background: var(--surface-2);
    padding: 2px 6px;
    border-radius: 3px;
    font-family: var(--ff-mono);
    font-size: 12px;
  }

  :global(.markdown-preview strong) {
    font-weight: 600;
  }
</style>
