<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Breakpoint {
    id: string;
    file: string;
    line: number;
    condition?: string;
    enabled: boolean;
  }

  interface StackFrame {
    name: string;
    file: string;
    line: number;
  }

  interface $$Props {
    isDebugger?: boolean;
    isPaused?: boolean;
    breakpoints?: Breakpoint[];
    callStack?: StackFrame[];
    onStart?: () => void;
    onPause?: () => void;
    onContinue?: () => void;
    onStepOver?: () => void;
    onStepInto?: () => void;
    onStepOut?: () => void;
    onAddBreakpoint?: (file: string, line: number) => void;
    onToggleBreakpoint?: (id: string) => void;
  }

  let {
    isDebugger = false,
    isPaused = false,
    breakpoints = [],
    callStack = [],
    onStart = () => {},
    onPause = () => {},
    onContinue = () => {},
    onStepOver = () => {},
    onStepInto = () => {},
    onStepOut = () => {},
    onAddBreakpoint = () => {},
    onToggleBreakpoint = () => {},
  }: $$Props = $props();
</script>

<div class="debug-panel">
  <div class="debug-header">
    <div class="debug-status">
      <div class="status-indicator" class:running={isDebugger} class:paused={isPaused}>
        {#if isDebugger}
          <Icons.Play size={12} />
        {:else}
          <Icons.Square size={12} />
        {/if}
      </div>
      <span>{isPaused ? 'Paused' : isDebugger ? 'Running' : 'Stopped'}</span>
    </div>

    <div class="debug-controls">
      {#if !isDebugger}
        <button
          type="button"
          class="control-btn"
          onclick={onStart}
          title="Start debugging"
        >
          <Icons.Play size={14} />
        </button>
      {:else if isPaused}
        <button
          type="button"
          class="control-btn"
          onclick={onContinue}
          title="Continue"
        >
          <Icons.Play size={14} />
        </button>
        <button
          type="button"
          class="control-btn"
          onclick={onStepOver}
          title="Step over"
        >
          <Icons.ChevronDown size={14} />
        </button>
        <button
          type="button"
          class="control-btn"
          onclick={onStepInto}
          title="Step into"
        >
          <Icons.ChevronRight size={14} />
        </button>
        <button
          type="button"
          class="control-btn"
          onclick={onStepOut}
          title="Step out"
        >
          <Icons.ChevronUp size={14} />
        </button>
      {:else}
        <button
          type="button"
          class="control-btn"
          onclick={onPause}
          title="Pause"
        >
          <Icons.Pause size={14} />
        </button>
      {/if}
    </div>
  </div>

  <div class="debug-tabs">
    <button type="button" class="debug-tab active">
      Breakpoints ({breakpoints.length})
    </button>
    <button type="button" class="debug-tab">
      Call Stack ({callStack.length})
    </button>
  </div>

  {#if breakpoints.length === 0}
    <div class="empty-state">
      <Icons.Circle size={24} />
      <p>No breakpoints</p>
    </div>
  {:else}
    <div class="breakpoints-list">
      {#each breakpoints as bp (bp.id)}
        <div class="breakpoint-item">
          <button
            type="button"
            class="bp-toggle"
            onclick={() => onToggleBreakpoint(bp.id)}
            title={bp.enabled ? 'Disable' : 'Enable'}
          >
            <Icons.Circle
              size={12}
              style={bp.enabled ? 'fill: var(--danger)' : ''}
            />
          </button>
          <div class="bp-info">
            <span class="bp-file">{bp.file}:{bp.line}</span>
            {#if bp.condition}
              <span class="bp-condition">{bp.condition}</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .debug-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
  }

  .debug-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .debug-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text);
  }

  .status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 2px;
    background: var(--dim);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-size: 8px;
  }

  .status-indicator.running {
    background: var(--green);
  }

  .status-indicator.paused {
    background: var(--orange);
  }

  .debug-controls {
    display: flex;
    gap: 4px;
  }

  .control-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .control-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .debug-tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .debug-tab {
    flex: 1;
    padding: 8px;
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .debug-tab:hover {
    color: var(--text);
  }

  .debug-tab.active {
    color: var(--blue);
    border-bottom-color: var(--blue);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 40px 16px;
    color: var(--muted);
    text-align: center;
  }

  .breakpoints-list {
    flex: 1;
    overflow-y: auto;
  }

  .breakpoint-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--hairline);
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .breakpoint-item:hover {
    background: var(--surface-2);
  }

  .bp-toggle {
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .bp-toggle:hover {
    color: var(--text);
  }

  .bp-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .bp-file {
    font-size: 12px;
    color: var(--text);
    font-family: var(--ff-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .bp-condition {
    font-size: 10px;
    color: var(--muted);
    font-family: var(--ff-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
