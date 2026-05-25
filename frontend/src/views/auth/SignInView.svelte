<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { api } from '$lib/api';
  import { user, isLoading, error } from '$lib/stores/auth';
  import { navigateTo, preferences } from '$lib/stores/ui';
  import { success } from '$lib/stores/notifications';
  import Btn from '$lib/components/Btn.svelte';

  let name = $state('');
  let secret = $state('');
  let localError = $state('');

  async function handleSignIn() {
    localError = '';

    if (!secret) {
      localError = 'Access code or password required';
      return;
    }

    isLoading.set(true);
    error.set(null);

    try {
      const normalizedName = name.trim() || undefined;
      const data = await api.login({
        name: normalizedName,
        code: secret,
        password: secret,
      });
      user.set({ id: '', name: data.name, role: data.role as 'admin' | 'user' });
      success(`Welcome back, ${data.name}!`);
      navigateTo($preferences.landingApp || 'home');
    } catch (e) {
      localError = e instanceof Error ? e.message : 'Sign in failed';
      error.set(localError);
    } finally {
      isLoading.set(false);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleSignIn();
    }
  }
</script>

<div class="signin-container">
  <div class="signin-card">
    <div class="signin-header">
      <div class="signin-logo">
        <Icons.Zap size={32} />
      </div>
      <h1>TSSP</h1>
      <p>Self-hosted File Transfer System</p>
    </div>

    <form
      class="signin-form"
      onsubmit={(event) => {
        event.preventDefault();
        void handleSignIn();
      }}
    >
      {#if localError || $error}
        <div class="signin-error">
          <Icons.AlertCircle size={16} />
          <span>{localError || $error}</span>
        </div>
      {/if}

      <div class="form-group">
        <label for="name">Name</label>
        <input
          id="name"
          type="text"
          placeholder="local user name"
          bind:value={name}
          onkeydown={handleKeydown}
          disabled={$isLoading}
        />
      </div>

      <div class="form-group">
        <label for="password">Access code or password</label>
        <input
          id="password"
          type="password"
          placeholder="••••••••"
          bind:value={secret}
          onkeydown={handleKeydown}
          disabled={$isLoading}
          required
        />
      </div>

      <Btn
        kind="primary"
        size="lg"
        disabled={$isLoading}
        onclick={handleSignIn}
        style="width: 100%"
      >
        {#if $isLoading}
          <div class="spinner"></div>
          Signing in...
        {:else}
          Sign In
        {/if}
      </Btn>
    </form>

    <div class="signin-footer">
      <p>Use your account access code, or the shared password when the server is in password mode.</p>
    </div>
  </div>
</div>

<style>
  .signin-container {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    background: var(--bg);
    padding: var(--s-6);
  }

  .signin-card {
    width: 100%;
    max-width: 400px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    padding: var(--s-8);
    box-shadow: var(--shadow-modal);
  }

  .signin-header {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-3);
    margin-bottom: var(--s-8);
    text-align: center;
  }

  .signin-logo {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    background: var(--blue-subtle);
    border-radius: var(--r-3);
    color: var(--blue);
  }

  .signin-header h1 {
    margin: 0;
    font-size: var(--fs-32);
    font-weight: 700;
    color: var(--text);
  }

  .signin-header p {
    margin: 0;
    font-size: var(--fs-13);
    color: var(--muted);
  }

  .signin-form {
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
    margin-bottom: var(--s-6);
  }

  .signin-error {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-3);
    background: rgba(255, 107, 107, 0.1);
    border: 1px solid rgba(255, 107, 107, 0.2);
    border-radius: var(--r-2);
    color: var(--danger);
    font-size: var(--fs-12);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .form-group label {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
  }

  .form-group input {
    padding: var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    border-radius: var(--r-2);
    font-family: var(--ff-sans);
    font-size: var(--fs-13);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .form-group input:focus {
    outline: none;
    border-color: var(--blue);
    box-shadow: 0 0 0 3px rgba(110, 168, 255, 0.1);
  }

  .form-group input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .signin-footer {
    text-align: center;
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .signin-footer p {
    margin: 0;
  }

  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
