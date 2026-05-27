<script lang="ts">
  import type { Snippet } from 'svelte';
  import Card from './Card.svelte';
  import * as Icons from 'lucide-svelte';

  interface Props {
    title: string;
    value: string | number;
    meta?: string;
    icon?: any;
    children?: Snippet;
  }

  let { title, value, meta, icon: IconComponent, children }: Props = $props();
</script>

<Card>
  <div class="stat-card">
    <div class="stat-header">
      <h3>{title}</h3>
      {#if IconComponent}
        <IconComponent size={20} />
      {/if}
    </div>
    <div class="stat-value">{value}</div>
    {#if children}
      <div class="stat-content">
        {@render children()}
      </div>
    {/if}
    {#if meta}
      <div class="stat-meta">{meta}</div>
    {/if}
  </div>
</Card>

<style>
  .stat-card {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .stat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .stat-header h3 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text-2);
  }

  .stat-value {
    font-size: var(--fs-28);
    font-weight: 700;
    color: var(--text);
  }

  .stat-content {
    flex: 1;
  }

  .stat-meta {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    font-size: var(--fs-12);
    color: var(--muted);
  }
</style>
