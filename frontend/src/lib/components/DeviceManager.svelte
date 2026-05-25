<script lang="ts">
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import { api } from '$lib/api';

  interface Device {
    token: string;
    name: string;
    ip_address?: string;
    user_agent?: string;
    created_at: number;
    last_used_at: number;
  }

  let devices: Device[] = $state([]);
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let currentDeviceToken = $state<string | null>(null);

  onMount(async () => {
    await loadDevices();
  });

  async function loadDevices() {
    try {
      isLoading = true;
      const data = await api.listAdminDevices();
      devices = (data.devices || []).map(d => ({
        id: d.id,
        token: d.token,
        name: 'Admin Trusted',
        fingerprint: d.token.substring(0, 8),
        created_at: d.trusted_at || Date.now() / 1000,
        last_used_at: d.trusted_at || Date.now() / 1000,
        is_current: false
      })) as Device[];
      currentDeviceToken = localStorage.getItem('device_token');
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load devices';
    } finally {
      isLoading = false;
    }
  }

  async function revokeDevice(token: string) {
    if (!confirm('Revoke this device session?')) return;

    try {
      await api.removeAdminDevice(token);
      devices = devices.filter(d => d.token !== token);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to revoke device';
    }
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function getDeviceIcon(userAgent?: string): any {
    if (!userAgent) return Icons.Smartphone;
    if (userAgent.includes('Win')) return Icons.Monitor;
    if (userAgent.includes('Mac')) return Icons.Apple;
    if (userAgent.includes('Linux')) return Icons.Cpu;
    if (userAgent.includes('iPhone') || userAgent.includes('iOS')) return Icons.Smartphone;
    if (userAgent.includes('Android')) return Icons.Smartphone;
    return Icons.Smartphone;
  }

  function getDeviceName(device: Device): string {
    if (device.name) return device.name;
    if (device.user_agent?.includes('Chrome')) return 'Chrome';
    if (device.user_agent?.includes('Firefox')) return 'Firefox';
    if (device.user_agent?.includes('Safari')) return 'Safari';
    return 'Unknown Device';
  }
</script>

<div class="device-manager">
  <div class="device-header">
    <h3>Active Sessions</h3>
    <p class="subtitle">Manage your logged-in devices</p>
  </div>

  {#if isLoading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading devices...</p>
    </div>
  {:else if error}
    <div class="error-message">
      <Icons.AlertCircle size={16} />
      <p>{error}</p>
    </div>
  {:else if devices.length === 0}
    <div class="empty-state">
      <Icons.Smartphone size={32} />
      <p>No active sessions</p>
    </div>
  {:else}
    <div class="devices-list">
      {#each devices as device (device.token)}
        {@const DeviceIcon = getDeviceIcon(device.user_agent)}
        {@const DeviceIconComponent = DeviceIcon}
        {@const isCurrent = device.token === currentDeviceToken}
        <div class="device-card" class:current={isCurrent}>
          <div class="device-icon">
            <DeviceIconComponent size={20} />
          </div>

          <div class="device-info">
            <div class="device-name">
              {getDeviceName(device)}
              {#if isCurrent}
                <span class="badge">Current</span>
              {/if}
            </div>
            <div class="device-details">
              {#if device.ip_address}
                <span>{device.ip_address}</span>
              {/if}
              <span>•</span>
              <span>Last used {formatDate(device.last_used_at || device.created_at)}</span>
            </div>
          </div>

          {#if !isCurrent}
            <button
              type="button"
              class="revoke-btn"
              title="Revoke this session"
              onclick={() => revokeDevice(device.token)}
            >
              <Icons.X size={16} />
            </button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .device-manager {
    padding: var(--s-3);
    border-radius: var(--r-2);
    border: 1px solid var(--border);
    background: var(--surface);
  }

  .device-header {
    margin-bottom: var(--s-3);
  }

  .device-header h3 {
    margin: 0 0 var(--s-1) 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }

  .subtitle {
    margin: 0;
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .loading,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    padding: var(--s-6);
    color: var(--muted);
    text-align: center;
  }

  .loading p,
  .empty-state p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-3);
    background: rgba(255, 107, 107, 0.1);
    border: 1px solid rgba(255, 107, 107, 0.25);
    border-radius: var(--r-1);
    color: var(--danger);
  }

  .error-message p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .devices-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .device-card {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-3);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    transition: all var(--duration-quick);
  }

  .device-card:hover {
    background: var(--surface-2);
  }

  .device-card.current {
    border-color: var(--blue);
    background: rgba(110, 168, 255, 0.05);
  }

  .device-icon {
    flex-shrink: 0;
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface);
    border-radius: var(--r-1);
    color: var(--text-2);
  }

  .device-info {
    flex: 1;
    min-width: 0;
  }

  .device-name {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    margin: 0 0 var(--s-1) 0;
    font-weight: 600;
    color: var(--text);
    font-size: var(--fs-13);
  }

  .badge {
    display: inline-block;
    padding: 2px 6px;
    background: var(--blue);
    color: white;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 600;
  }

  .device-details {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    font-size: var(--fs-11);
    color: var(--text-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .revoke-btn {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    background: transparent;
    color: var(--danger);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick);
  }

  .revoke-btn:hover {
    background: var(--danger);
    color: white;
    border-color: var(--danger);
  }
</style>
