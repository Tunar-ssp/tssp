<script lang="ts">
  interface $$Props {
    variant?: 'default' | 'success' | 'warning' | 'danger' | 'info';
    size?: 'sm' | 'md';
    class?: string;
  }

  let {
    variant = 'default',
    size = 'md',
    class: className,
    children,
  }: $$Props = $props();

  const variantClasses = {
    default: 'badge-default',
    success: 'badge-success',
    warning: 'badge-warning',
    danger: 'badge-danger',
    info: 'badge-info',
  };

  const sizeClasses = {
    sm: 'badge-sm',
    md: 'badge-md',
  };
</script>

<span class="badge {variantClasses[variant]} {sizeClasses[size]} {className || ''}">
  <slot />
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px 8px;
    border-radius: var(--r-full);
    font-weight: 600;
    white-space: nowrap;
    font-family: var(--ff-sans);
  }

  .badge-sm {
    font-size: var(--fs-11);
    padding: 2px 6px;
  }

  .badge-md {
    font-size: var(--fs-12);
    padding: 4px 10px;
  }

  .badge-default {
    background: var(--surface-2);
    color: var(--text-2);
  }

  .badge-success {
    background: rgba(52, 211, 153, 0.15);
    color: var(--success);
  }

  .badge-warning {
    background: rgba(251, 191, 36, 0.15);
    color: var(--warning);
  }

  .badge-danger {
    background: rgba(255, 107, 107, 0.15);
    color: var(--danger);
  }

  .badge-info {
    background: rgba(110, 168, 255, 0.15);
    color: var(--blue);
  }
</style>
