<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { isAdmin } from '$lib/stores/auth';
  import { info } from '$lib/stores/notifications';
  import Btn from '$lib/components/Btn.svelte';
  import Card from '$lib/components/Card.svelte';
  import StatusDot from '$lib/components/StatusDot.svelte';

  interface InviteCode {
    id: string;
    code: string;
    created_by: string;
    created_at: number;
    expires_at: number;
    used_by?: string;
    used_at?: number;
  }

  let inviteCodes = $state<InviteCode[]>([]);
  let isLoading = $state(false);
  let showCreateForm = $state(false);

  async function loadInviteCodes() {
    if (!$isAdmin) return;
    isLoading = true;
    inviteCodes = [];
    isLoading = false;
  }

  async function createInviteCode() {
    showCreateForm = false;
    info('Invite Codes Disabled', 'The backend does not expose invite-code APIs yet.');
  }

  async function revokeInviteCode(codeId: string) {
    if (!confirm('Revoke this invite code?')) return;

    info('Invite Codes Disabled', `Cannot revoke ${codeId}; invite-code APIs are not available.`);
  }

  function copyCode(code: string) {
    navigator.clipboard.writeText(code);
    info('Copied', 'Invite code copied to clipboard');
  }

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  }

  function isExpired(expiresAt: number) {
    return expiresAt < Date.now() / 1000;
  }

  function isUsed(usedAt?: number) {
    return usedAt !== undefined;
  }

  $effect(() => {
    if ($isAdmin) {
      loadInviteCodes();
    }
  });
</script>

{#if !$isAdmin}
  <div class="not-admin">
    <Icons.Lock size={48} />
    <h3>Admin Only</h3>
    <p>Only administrators can view and manage invite codes</p>
  </div>
{:else}
  <div class="invites-view">
    <div class="view-header">
      <div>
        <h2>Invite Codes</h2>
        <p class="subtitle">Manage user invitations</p>
      </div>
      <Btn
        kind="primary"
        disabled
        onClick={() => (showCreateForm = !showCreateForm)}
      >
        <Icons.Plus size={14} />
        Invite API missing
      </Btn>
    </div>

    {#if showCreateForm}
      <div class="create-form">
        <Card>
          <div class="form-content">
            <p>Create a new invite code for a new user</p>
            <div class="form-actions">
              <Btn kind="primary" onClick={createInviteCode}>
                Create Code
              </Btn>
              <Btn
                kind="ghost"
                onClick={() => (showCreateForm = false)}
              >
                Cancel
              </Btn>
            </div>
          </div>
        </Card>
      </div>
    {/if}

    {#if isLoading}
      <div class="loading">
        <div class="spinner"></div>
        Loading codes...
      </div>
    {:else if inviteCodes.length === 0}
      <div class="empty">
        <Icons.Mail size={48} />
        <h3>No invite codes</h3>
        <p>Invite-code APIs are not implemented on the backend yet.</p>
      </div>
    {:else}
      <div class="invites-list">
        {#each inviteCodes as code (code.id)}
          {@const expired = isExpired(code.expires_at)}
          {@const used = isUsed(code.used_at)}
          <Card>
            <div class="code-card">
              <div class="code-header">
                <div class="code-status">
                  {#if used}
                    <StatusDot tone="ok" />
                    <span>Used</span>
                  {:else if expired}
                    <StatusDot tone="warn" />
                    <span>Expired</span>
                  {:else}
                    <StatusDot tone="info" />
                    <span>Active</span>
                  {/if}
                </div>
                <div class="code-value">
                  <code>{code.code}</code>
                  <button type="button" class="copy-btn" onclick={() => copyCode(code.code)}>
                    <Icons.Copy size={14} />
                  </button>
                </div>
              </div>

              <div class="code-meta">
                <div>Created by {code.created_by} on {formatDate(code.created_at)}</div>
                <div>Expires {formatDate(code.expires_at)}</div>
                {#if used && code.used_by}
                  <div>Used by {code.used_by} on {formatDate(code.used_at || 0)}</div>
                {/if}
              </div>

              {#if !used && !expired}
                <div class="code-actions">
                  <Btn
                    kind="danger"
                    size="sm"
                    onClick={() => revokeInviteCode(code.id)}
                  >
                    <Icons.Trash2 size={14} />
                    Revoke
                  </Btn>
                </div>
              {/if}
            </div>
          </Card>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .not-admin {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    color: var(--muted);
    padding: var(--s-6);
  }

  .not-admin h3 {
    margin: 0;
    color: var(--text-2);
  }

  .not-admin p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .invites-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-6);
    border-bottom: 1px solid var(--border);
  }

  .view-header h2 {
    margin: 0;
    font-size: var(--fs-24);
    color: var(--text);
  }

  .subtitle {
    margin: var(--s-2) 0 0;
    font-size: var(--fs-13);
    color: var(--muted);
  }

  .create-form {
    padding: 0 var(--s-6) var(--s-4);
  }

  .form-content {
    padding: var(--s-4);
  }

  .form-content p {
    margin: 0 0 var(--s-4);
    color: var(--text-2);
  }

  .form-actions {
    display: flex;
    gap: var(--s-3);
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    color: var(--muted);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    color: var(--muted);
  }

  .empty h3 {
    margin: 0;
    color: var(--text-2);
  }

  .empty p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .invites-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
    padding: var(--s-6);
  }

  .code-card {
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
  }

  .code-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
  }

  .code-status {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    font-size: var(--fs-13);
    color: var(--text-2);
  }

  .code-value {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .code-value code {
    background: var(--surface-2);
    padding: var(--s-2) var(--s-3);
    border-radius: var(--r-1);
    font-family: var(--ff-mono);
    font-size: var(--fs-12);
    color: var(--text);
    font-weight: 500;
  }

  .copy-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    border-radius: var(--r-1);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .copy-btn:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .code-meta {
    font-size: var(--fs-12);
    color: var(--muted);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .code-actions {
    padding-top: var(--s-4);
    border-top: 1px solid var(--border);
  }
</style>
