<script lang="ts">
  import type { StatusDotProps } from './primitives.svelte';

  interface $$Props extends StatusDotProps {
    class?: string;
  }

  let {
    tone = 'info',
    class: className,
    ...rest
  }: $$Props = $props();

  const toneClasses = {
    ok: 'dot-ok',
    warn: 'dot-warn',
    err: 'dot-err',
    info: 'dot-info',
  };
</script>

<span class="status-dot {toneClasses[tone]} {className || ''}" {...rest}></span>

<style>
  .status-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .dot-ok {
    background: var(--success);
    box-shadow: 0 0 8px rgba(52, 211, 153, 0.4);
  }

  .dot-warn {
    background: var(--warning);
    box-shadow: 0 0 8px rgba(251, 191, 36, 0.4);
  }

  .dot-err {
    background: var(--danger);
    box-shadow: 0 0 8px rgba(255, 107, 107, 0.4);
  }

  .dot-info {
    background: var(--blue);
    box-shadow: 0 0 8px rgba(110, 168, 255, 0.4);
  }
</style>
