<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { api } from '$lib/api';
  import { user } from '$lib/stores/auth';
  import { success, error as showError } from '$lib/stores/notifications';
  import { confirmDialog } from '$lib/stores/dialog';
  import Btn from '$lib/components/Btn.svelte';
  import Card from '$lib/components/Card.svelte';
  import StatusDot from '$lib/components/StatusDot.svelte';

  interface Device {
    id: string;
    name: string;
    trusted_at?: number;
  }

  let devices = $state<Device[]>([]);
  let isLoading = $state(false);

  async function loadDevices() {
    isLoading = true;
    try {
      const result = await api.listDevices();
      devices = result.devices;
    } catch (e) {
      showError(e instanceof Error ? e.message : 'Failed to load devices');
    } finally {
      isLoading = false;
    }
  }

  async function revokeDevice(deviceId: string) {
    const ok = await confirmDialog({
      title: 'Revoke device',
      message: 'This device will lose access and must sign in again.',
      confirmLabel: 'Revoke',
      tone: 'danger',
    });
    if (!ok) return;

    try {
      await api.removeDevice(deviceId);
      success('Device revoked');
      await loadDevices();
    } catch (e) {
      showError(e instanceof Error ? e.message : 'Failed to revoke device');
    }
  }

  $effect(() => {
    if ($user) {
      loadDevices();
    }
  });

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }
</script>

<div class="devices-view">
  <div class="view-header">
    <h2>Trusted Devices</h2>
    <p class="subtitle">Manage devices that can access your account</p>
  </div>

  {#if isLoading}
    <div class="loading">
      <div class="spinner"></div>
      Loading devices...
    </div>
  {:else if devices.length === 0}
    <div class="empty">
      <Icons.Smartphone size={48} />
      <h3>No devices yet</h3>
      <p>Your devices will appear here once you sign in</p>
    </div>
  {:else}
    <div class="devices-list">
      {#each devices as device (device.id)}
        <Card>
          <div class="device-card">
            <div class="device-header">
              <div class="device-icon">
                <Icons.Monitor size={20} />
              </div>
              <div class="device-info">
                <div class="device-name">{device.name}</div>
                {#if device.trusted_at}
                  <div class="device-meta">Trusted {formatDate(device.trusted_at)}</div>
                {/if}
              </div>
              <StatusDot tone="ok" />
            </div>

            <div class="device-footer">
              <Btn
                kind="danger"
                size="sm"
                onclick={() => revokeDevice(device.id)}
              >
                <Icons.Trash2 size={14} />
                Revoke
              </Btn>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<style>
  .devices-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .view-header {
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

  .devices-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
    padding: var(--s-6);
  }

  .device-card {
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
  }

  .device-header {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    justify-content: space-between;
  }

  .device-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    background: var(--surface-2);
    border-radius: var(--r-2);
    color: var(--text-2);
    flex-shrink: 0;
  }

  .device-info {
    flex: 1;
    min-width: 0;
  }

  .device-name {
    font-weight: 500;
    color: var(--text);
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .device-meta {
    font-size: var(--fs-12);
    color: var(--muted);
    margin-top: 4px;
  }

  .device-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding-top: var(--s-4);
    border-top: 1px solid var(--border);
  }
</style>
