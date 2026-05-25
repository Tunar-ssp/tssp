<script lang="ts">
  import type { TooltipProps } from './primitives.svelte';

  interface $$Props extends TooltipProps {
    class?: string;
  }

  let {
    content,
    delay = 200,
    kbd,
    class: className,
    children,
    ...rest
  }: $$Props = $props();

  let showTooltip = $state(false);
  let timeout: ReturnType<typeof setTimeout>;

  function onMouseEnter() {
    timeout = setTimeout(() => {
      showTooltip = true;
    }, delay);
  }

  function onMouseLeave() {
    clearTimeout(timeout);
    showTooltip = false;
  }
</script>

<div class="tooltip-wrapper {className || ''}" {...rest}>
  <div
    class="tooltip-trigger"
    on:mouseenter={onMouseEnter}
    on:mouseleave={onMouseLeave}
  >
    <slot />
  </div>

  {#if showTooltip && content}
    <div class="tooltip-content" on:mouseenter={onMouseEnter} on:mouseleave={onMouseLeave}>
      <div class="tooltip-text">{content}</div>
      {#if kbd}
        <div class="tooltip-kbd">{kbd}</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tooltip-wrapper {
    position: relative;
    display: inline-block;
  }

  .tooltip-trigger {
    display: contents;
  }

  .tooltip-content {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    background: var(--surface-hi);
    border: 1px solid var(--border-2);
    border-radius: var(--r-2);
    white-space: nowrap;
    z-index: 1000;
    font-size: var(--fs-12);
    color: var(--text-2);
    animation: tooltipIn var(--duration-quick) var(--ease-smooth);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }

  @keyframes tooltipIn {
    from {
      opacity: 0;
      transform: translateX(-50%) translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }
  }

  .tooltip-content::after {
    content: '';
    position: absolute;
    top: 100%;
    left: 50%;
    transform: translateX(-50%);
    width: 0;
    height: 0;
    border-left: 4px solid transparent;
    border-right: 4px solid transparent;
    border-top: 4px solid var(--surface-hi);
  }

  .tooltip-text {
    color: var(--text-2);
  }

  .tooltip-kbd {
    display: flex;
    gap: 2px;
    padding-left: var(--s-2);
    border-left: 1px solid var(--border);
  }
</style>
