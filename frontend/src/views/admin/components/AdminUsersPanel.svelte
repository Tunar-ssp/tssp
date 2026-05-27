<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { api, type AdminUser } from '$lib/api';
  import { error as showError } from '$lib/stores/notifications';
  import Card from '$lib/components/Card.svelte';
  import { withLoadingState, formatErrorMessage } from '$lib/utils/apiHelpers';

  interface Props {
    users: AdminUser[];
    isLoading?: boolean;
    onReload?: () => void;
  }

  let { users, isLoading = false, onReload }: Props = $props();

  async function revokeAccess(userId: string) {
    if (!confirm('Revoke access for this user?')) return;

    const result = await withLoadingState(
      () => api.revokeUserAccess(userId),
      { successMessage: 'Access revoked' }
    );

    if (result !== null) {
      onReload?.();
    }
  }
</script>

<div class="admin-panel">
  {#if users.length === 0}
    <div class="empty-state">
      <Icons.Users size={48} />
      <h3>No users</h3>
      <p>No users have been created yet.</p>
    </div>
  {:else}
    <div class="users-list">
      {#each users as user (user.id)}
        <Card>
          <div class="user-item">
            <div class="user-info">
              <h4>{user.name}</h4>
              <p class="user-role">{user.role === 'admin' ? '👑 Admin' : 'User'}</p>
            </div>
            <div class="user-meta">
              <span class="created-date">
                Created {new Date(user.created_at).toLocaleDateString()}
              </span>
              {#if user.role !== 'admin'}
                <button class="revoke-btn" onclick={() => revokeAccess(user.id)}>
                  <Icons.Trash2 size={16} />
                  Revoke
                </button>
              {/if}
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<style>
  .admin-panel {
    flex: 1;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    padding: var(--s-8);
    text-align: center;
    color: var(--muted);
  }

  .empty-state h3 {
    margin: 0;
    color: var(--text-2);
  }

  .users-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .user-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-3);
    gap: var(--s-4);
  }

  .user-info h4 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }

  .user-role {
    margin: var(--s-1) 0 0;
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .user-meta {
    display: flex;
    align-items: center;
    gap: var(--s-3);
  }

  .created-date {
    font-size: var(--fs-12);
    color: var(--muted);
    white-space: nowrap;
  }

  .revoke-btn {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    background: transparent;
    color: var(--danger);
    font-size: var(--fs-12);
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .revoke-btn:hover {
    background: rgba(255, 107, 107, 0.1);
    border-color: var(--danger);
  }
</style>
