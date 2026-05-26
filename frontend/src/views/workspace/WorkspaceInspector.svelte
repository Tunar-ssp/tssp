<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import Outline from '$lib/components/Outline.svelte';
  import TerminalClient from './TerminalClient.svelte';

  interface CapabilityStatus {
    status: 'available' | 'disabled' | 'forbidden' | 'unavailable_sandbox' | 'unavailable';
    message?: string;
  }

  interface LspStatus {
    status: 'available' | 'disabled' | 'unavailable' | 'not_implemented';
    available_languages?: string[];
    message?: string;
  }

  interface $$Props {
    tab: 'preview' | 'outline' | 'terminal';
    workspaceId?: string;
    previewHtml: string;
    content: string;
    selectedLanguage: string;
    terminalCapability: CapabilityStatus | null;
    lspCapability: LspStatus | null;
    onTabChange: (tab: 'preview' | 'outline' | 'terminal') => void;
  }

  let {
    tab,
    workspaceId = '',
    previewHtml,
    content,
    selectedLanguage,
    terminalCapability,
    lspCapability,
    onTabChange,
  }: $$Props = $props();

  function getTerminalStatusLabel(status: CapabilityStatus | null): string {
    if (!status) return 'Loading...';
    switch (status.status) {
      case 'available': return 'Terminal Ready';
      case 'disabled': return 'Terminal Disabled';
      case 'forbidden': return 'Admin Only';
      case 'unavailable_sandbox': return 'Sandbox Unavailable';
      case 'unavailable': return 'Not Available';
      default: return 'Unknown Status';
    }
  }

  function getTerminalStatusIcon(status: CapabilityStatus | null): any {
    if (!status) return Icons.Loader;
    switch (status.status) {
      case 'available': return Icons.CheckCircle;
      case 'disabled': return Icons.XCircle;
      case 'forbidden': return Icons.Lock;
      case 'unavailable_sandbox': return Icons.AlertTriangle;
      case 'unavailable': return Icons.Slash;
      default: return Icons.HelpCircle;
    }
  }

  function getLspStatusLabel(status: LspStatus | null): string {
    if (!status) return 'Loading...';
    switch (status.status) {
      case 'available': return 'LSP Available';
      case 'disabled': return 'LSP Disabled';
      case 'unavailable': return 'LSP Unavailable';
      case 'not_implemented': return 'Coming Soon';
      default: return 'Unknown Status';
    }
  }
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
      {#if !terminalCapability}
        <div class="capability-loading">
          <Icons.Loader size={18} class="spinner" />
          <span>Loading terminal status...</span>
        </div>
      {:else if terminalCapability.status === 'available'}
        <TerminalClient {workspaceId} isAvailable={true} />
      {:else if terminalCapability.status === 'forbidden'}
        <div class="terminal-disabled">
          <Icons.Lock size={18} />
          <div>
            <strong>Admin Only</strong>
            <p>Terminal access requires admin privileges.</p>
          </div>
        </div>
      {:else if terminalCapability.status === 'unavailable_sandbox'}
        <div class="terminal-warning">
          <Icons.AlertTriangle size={18} />
          <div>
            <strong>Sandbox Unavailable</strong>
            <p>Terminal requires bubblewrap or systemd-nspawn to be installed on the server.</p>
          </div>
        </div>
      {:else if terminalCapability.status === 'disabled'}
        <div class="terminal-disabled">
          <Icons.Slash size={18} />
          <div>
            <strong>Terminal Disabled</strong>
            <p>Terminal support is not enabled in the server configuration.</p>
          </div>
        </div>
      {:else}
        <div class="terminal-warning">
          <Icons.HelpCircle size={18} />
          <div>
            <strong>Status Unknown</strong>
            <p>Unable to determine terminal status. Check server logs.</p>
          </div>
        </div>
      {/if}
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
  .terminal-warning {
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

  .plain-preview-card pre {
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

  .terminal-disabled {
    display: flex;
    gap: 12px;
    color: var(--muted);
  }

  .terminal-disabled strong {
    display: block;
    color: var(--text);
    margin-bottom: 6px;
  }

  .capability-loading {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 18px;
    border-radius: 20px;
    border: 1px solid var(--border);
    background: rgba(18, 21, 29, 0.98);
    color: var(--text-2);
  }

  :global(.capability-loading .spinner) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .sidebar-label {
    font-family: var(--ff-mono);
    font-size: 12px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--dim);
  }
</style>
