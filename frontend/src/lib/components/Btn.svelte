<script lang="ts">
  import type { ButtonProps } from './primitives.svelte';

  interface $$Props extends ButtonProps {
    class?: string;
    children?: import('svelte').Snippet;
    style?: string;
  }

  let {
    kind = 'primary',
    size = 'md',
    icon,
    iconRight,
    disabled = false,
    onClick,
    onclick,
    class: className,
    children,
    ...rest
  }: $$Props = $props();

  function handleClick(event: MouseEvent) {
    onclick?.(event);
    onClick?.();
  }

  const sizeClasses = {
    sm: 'btn-sm',
    md: 'btn-md',
    lg: 'btn-lg',
  };

  const kindClasses = {
    primary: 'btn-primary',
    accent: 'btn-accent',
    ghost: 'btn-ghost',
    solid: 'btn-solid',
    danger: 'btn-danger',
  };
</script>

<button
  class="btn {kindClasses[kind]} {sizeClasses[size]} {className || ''}"
  {disabled}
  onclick={handleClick}
  {...rest}
>
  {#if icon}
    {@const Icon = icon}
    <span class="btn-icon-left">
      <Icon />
    </span>
  {/if}
  {@render children?.()}
  {#if iconRight}
    {@const IconRight = iconRight}
    <span class="btn-icon-right">
      <IconRight />
    </span>
  {/if}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    font-family: var(--ff-sans);
    font-weight: 500;
    border: none;
    border-radius: var(--r-2);
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    white-space: nowrap;
    user-select: none;
  }

  .btn:focus-visible {
    outline: 2px solid var(--blue);
    outline-offset: 2px;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Sizes */
  .btn-sm {
    padding: var(--s-1) var(--s-3);
    font-size: var(--fs-12);
    height: 28px;
  }

  .btn-md {
    padding: var(--s-2) var(--s-4);
    font-size: var(--fs-13);
    height: 36px;
  }

  .btn-lg {
    padding: var(--s-3) var(--s-6);
    font-size: var(--fs-14);
    height: 44px;
  }

  /* Kinds */
  .btn-primary {
    background: var(--blue);
    color: #0a1228;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
    box-shadow: 0 0 0 2px rgba(110, 168, 255, 0.2);
  }

  .btn-primary:active:not(:disabled) {
    transform: scale(0.98);
  }

  .btn-accent {
    background: var(--green);
    color: #0a2818;
  }

  .btn-accent:hover:not(:disabled) {
    opacity: 0.9;
    box-shadow: 0 0 0 2px rgba(91, 227, 154, 0.2);
  }

  .btn-ghost {
    background: transparent;
    color: var(--text);
    border: 1px solid var(--border);
  }

  .btn-ghost:hover:not(:disabled) {
    background: var(--surface);
    border-color: var(--border-2);
  }

  .btn-solid {
    background: var(--surface-2);
    color: var(--text);
    border: 1px solid var(--border);
  }

  .btn-solid:hover:not(:disabled) {
    background: var(--surface-3);
  }

  .btn-danger {
    background: var(--danger);
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    opacity: 0.9;
    box-shadow: 0 0 0 2px rgba(255, 107, 107, 0.2);
  }

  .btn-icon-left,
  .btn-icon-right {
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
