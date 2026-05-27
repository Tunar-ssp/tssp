<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Props {
    label?: string;
    placeholder?: string;
    value?: string;
    type?: string;
    disabled?: boolean;
    error?: string;
    helperText?: string;
    icon?: any;
    size?: 'sm' | 'md' | 'lg';
    onChange?: (value: string) => void;
    onBlur?: () => void;
    onFocus?: () => void;
  }

  let {
    label,
    placeholder,
    value = '',
    type = 'text',
    disabled = false,
    error,
    helperText,
    icon: Icon,
    size = 'md',
    onChange,
    onBlur,
    onFocus,
  }: Props = $props();

  const sizeClasses = {
    sm: 'h-32 px-10 text-fs-12',
    md: 'h-36 px-12 text-fs-13',
    lg: 'h-40 px-12 text-fs-14',
  };

  const sizeMap = {
    sm: 16,
    md: 18,
    lg: 20,
  };
</script>

<div class="form-field">
  {#if label}
    <label class="field-label">{label}</label>
  {/if}

  <div class="input-wrapper" class:has-error={error}>
    {#if Icon}
      <Icon size={sizeMap[size]} class="field-icon" />
    {/if}

    <input
      {type}
      {placeholder}
      {disabled}
      {value}
      class="field-input {sizeClasses[size]}"
      on:change={(e) => onChange?.(e.currentTarget.value)}
      on:blur={onBlur}
      on:focus={onFocus}
      aria-invalid={!!error}
      aria-describedby={error ? 'error' : undefined}
    />

    {#if error}
      <Icons.AlertCircle size={16} class="error-icon" />
    {/if}
  </div>

  {#if error}
    <div class="field-error" id="error">{error}</div>
  {:else if helperText}
    <div class="field-helper">{helperText}</div>
  {/if}
</div>

<style>
  .form-field {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .field-label {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
    display: block;
  }

  .input-wrapper {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 0 var(--s-3);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    background: var(--surface);
    transition: all 200ms var(--ease-smooth);
  }

  .input-wrapper:focus-within {
    border-color: var(--blue);
    background: var(--surface-hi);
    box-shadow: 0 0 0 3px rgba(110, 168, 255, 0.1);
  }

  .input-wrapper.has-error {
    border-color: var(--danger);
    background: rgba(255, 107, 107, 0.05);
  }

  .input-wrapper.has-error:focus-within {
    box-shadow: 0 0 0 3px rgba(255, 107, 107, 0.1);
  }

  .field-icon {
    color: var(--muted);
    flex-shrink: 0;
  }

  .field-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    outline: none;
    font-family: inherit;
    font-size: inherit;
  }

  .field-input::placeholder {
    color: var(--muted);
  }

  .field-input:disabled {
    color: var(--dim);
    cursor: not-allowed;
  }

  .error-icon {
    color: var(--danger);
    flex-shrink: 0;
  }

  .field-error {
    font-size: var(--fs-12);
    color: var(--danger);
    display: flex;
    align-items: center;
    gap: var(--s-1);
  }

  .field-helper {
    font-size: var(--fs-12);
    color: var(--muted);
  }
</style>
