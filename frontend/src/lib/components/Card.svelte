<script lang="ts">
  import type { CardProps } from './primitives.svelte';
  import type { Snippet } from 'svelte';

  interface $$Props extends CardProps {
    class?: string;
    children?: Snippet;
  }

  let {
    head,
    foot,
    pad = 16,
    accent,
    class: className,
    children,
    ...rest
  }: $$Props = $props();
</script>

<div
  class="card {className || ''}"
  style="padding: {pad}px; {accent ? `border-left: 3px solid ${accent}; padding-left: ${pad - 3}px;` : ''}"
  {...rest}
>
  {#if head}
    <div class="card-head">
      {#if typeof head === 'function'}
        <svelte:component this={head} />
      {:else if head}
        {@render head()}
      {/if}
    </div>
  {/if}

  <div class="card-body">
    {#if children}
      {@render children()}
    {/if}
  </div>

  {#if foot}
    <div class="card-foot">
      {#if typeof foot === 'function'}
        <svelte:component this={foot} />
      {:else if foot}
        {@render foot()}
      {/if}
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
