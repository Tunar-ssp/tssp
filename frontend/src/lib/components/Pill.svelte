<script lang="ts">
  import type { PillProps } from './primitives.svelte';
  import type { Snippet } from 'svelte';

  interface $$Props extends PillProps {
    class?: string;
    children?: Snippet;
  }

  let {
    tone = 'neutral',
    class: className,
    children,
    ...rest
  }: $$Props = $props();

  const toneClasses = {
    green: 'pill-green',
    pink: 'pill-pink',
    blue: 'pill-blue',
    orange: 'pill-orange',
    violet: 'pill-violet',
    warn: 'pill-warn',
    danger: 'pill-danger',
    neutral: 'pill-neutral',
  };
</script>

<span class="pill {toneClasses[tone]} {className || ''}" {...rest}>
  <slot />
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: var(--s-1) var(--s-3);
    font-size: var(--fs-12);
    font-weight: 500;
    border-radius: var(--r-full);
    white-space: nowrap;
    font-family: var(--ff-sans);
  }

  .pill-green {
    background: var(--green-subtle);
    color: var(--green);
    border: 1px solid rgba(91, 227, 154, 0.2);
  }

  .pill-pink {
    background: var(--pink-subtle);
    color: var(--pink);
    border: 1px solid rgba(255, 95, 162, 0.2);
  }

  .pill-blue {
    background: var(--blue-subtle);
    color: var(--blue);
    border: 1px solid rgba(110, 168, 255, 0.2);
  }

  .pill-orange {
    background: var(--orange-subtle);
    color: var(--orange);
    border: 1px solid rgba(255, 138, 61, 0.2);
  }

  .pill-violet {
    background: var(--violet-subtle);
    color: var(--violet);
    border: 1px solid rgba(163, 148, 255, 0.2);
  }

  .pill-warn {
    background: rgba(251, 191, 36, 0.1);
    color: var(--warning);
    border: 1px solid rgba(251, 191, 36, 0.2);
  }

  .pill-danger {
    background: rgba(255, 107, 107, 0.1);
    color: var(--danger);
    border: 1px solid rgba(255, 107, 107, 0.2);
  }

  .pill-neutral {
    background: var(--surface-2);
    color: var(--text-2);
    border: 1px solid var(--border);
  }
</style>
