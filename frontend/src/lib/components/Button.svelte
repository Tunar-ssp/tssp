<script lang="ts">
  import type { Snippet } from 'svelte';

  interface $$Props {
    kind?: 'primary' | 'accent' | 'ghost' | 'solid' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    icon?: any;
    iconRight?: any;
    children?: Snippet;
    onclick?: (e: MouseEvent) => void;
  }

  let {
    kind = 'ghost',
    size = 'md',
    disabled = false,
    icon = null,
    iconRight = null,
    children,
    onclick,
  }: $$Props = $props();

  const sizes = {
    sm: { h: 26, px: 10, fs: 12 },
    md: { h: 32, px: 14, fs: 13 },
    lg: { h: 38, px: 18, fs: 14 },
  };

  const kinds = {
    primary: { bg: 'var(--text)', fg: 'var(--bg-deep)', bd: 'transparent' },
    accent: { bg: 'var(--blue)', fg: '#0a1228', bd: 'transparent' },
    ghost: { bg: 'transparent', fg: 'var(--text-2)', bd: 'var(--border)' },
    solid: { bg: 'var(--surface-2)', fg: 'var(--text)', bd: 'var(--border)' },
    danger: { bg: 'rgba(255,107,107,.1)', fg: 'var(--danger)', bd: 'rgba(255,107,107,.25)' },
  };

  const style = sizes[size];
  const theme = kinds[kind];
</script>

<button
  {disabled}
  style:height="{style.h}px"
  style:padding="0 {style.px}px"
  style:font-size="{style.fs}px"
  style:background={theme.bg}
  style:color={theme.fg}
  style:border="1px solid {theme.bd}"
  class="btn"
  {onclick}
>
  {#if icon}
    <span class="icon">{icon}</span>
  {/if}
  {#if children}
    {@render children()}
  {/if}
  {#if iconRight}
    <span class="icon">{iconRight}</span>
  {/if}
</button>

<style>
  .btn {
    border-radius: 8px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    font-family: inherit;
    font-weight: 500;
    white-space: nowrap;
    border: none;
    transition: all 0.15s ease;
  }

  .btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn:focus-visible {
    outline: 2px solid var(--blue);
    outline-offset: 2px;
  }
</style>
