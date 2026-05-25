<script lang="ts">
  import type { CardProps } from './primitives.svelte';

  interface $$Props extends CardProps {
    class?: string;
  }

  let {
    head,
    foot,
    pad = 16,
    accent,
    class: className,
    children,
    ...rest
  } = $props<$$Props>();
</script>

<div
  class="card {className || ''}"
  style="padding: {pad}px; {accent ? `border-left: 3px solid ${accent}; padding-left: ${pad - 3}px;` : ''}"
  {...rest}
>
  {#if head}
    <div class="card-head">
      <svelte:component this={head} />
    </div>
  {/if}

  <div class="card-body">
    <slot />
  </div>

  {#if foot}
    <div class="card-foot">
      <svelte:component this={foot} />
    </div>
  {/if}
</div>

<style>
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    box-shadow: var(--shadow-card);
  }

  .card-head {
    margin-bottom: var(--s-4);
    padding-bottom: var(--s-4);
    border-bottom: 1px solid var(--border);
  }

  .card-body {
    margin: 0;
  }

  .card-foot {
    margin-top: var(--s-4);
    padding-top: var(--s-4);
    border-top: 1px solid var(--border);
  }
</style>
