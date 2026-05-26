<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface MetricSnapshot {
    timestamp: number;
    cpuUsage?: number;
    memoryUsage?: number;
    renderTime?: number;
    fileSize?: number;
  }

  interface $$Props {
    metrics?: MetricSnapshot[];
    currentCpu?: number;
    currentMemory?: number;
    lastRenderTime?: number;
    fileSize?: number;
  }

  let {
    metrics = [],
    currentCpu = 0,
    currentMemory = 0,
    lastRenderTime = 0,
    fileSize = 0,
  }: $$Props = $props();

  function formatBytes(bytes: number): string {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    const value = bytes / 1024 ** index;
    return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
  }

  function getHealthStatus(value: number): 'good' | 'warning' | 'critical' {
    if (value < 30) return 'good';
    if (value < 70) return 'warning';
    return 'critical';
  }

  let cpuStatus = $derived(getHealthStatus(currentCpu));
  let memoryStatus = $derived(getHealthStatus(currentMemory));
  let avgRenderTime = $derived(
    metrics.length > 0
      ? metrics.reduce((sum, m) => sum + (m.renderTime || 0), 0) / metrics.length
      : 0
  );
</script>

<div class="performance-monitor">
  <div class="monitor-header">
    <h3>Performance</h3>
  </div>

  <div class="metrics-grid">
    <div class="metric-card">
      <div class="metric-label">CPU Usage</div>
      <div class="metric-value" class={cpuStatus}>
        {currentCpu.toFixed(1)}%
      </div>
      <div class="metric-bar">
        <div class="bar-fill" style="width: {currentCpu}%; background: {cpuStatus === 'good' ? 'var(--green)' : cpuStatus === 'warning' ? 'var(--orange)' : 'var(--danger)'}"></div>
      </div>
    </div>

    <div class="metric-card">
      <div class="metric-label">Memory</div>
      <div class="metric-value" class={memoryStatus}>
        {currentMemory.toFixed(1)}%
      </div>
      <div class="metric-bar">
        <div class="metric-bar" style="width: {currentMemory}%; background: {memoryStatus === 'good' ? 'var(--green)' : memoryStatus === 'warning' ? 'var(--orange)' : 'var(--danger)'}"></div>
      </div>
    </div>

    <div class="metric-card">
      <div class="metric-label">Render Time</div>
      <div class="metric-value">
        {lastRenderTime.toFixed(2)}ms
      </div>
      <small class="metric-note">Avg: {avgRenderTime.toFixed(2)}ms</small>
    </div>

    <div class="metric-card">
      <div class="metric-label">File Size</div>
      <div class="metric-value">
        {formatBytes(fileSize)}
      </div>
    </div>
  </div>

  {#if metrics.length > 0}
    <div class="metrics-chart">
      <div class="chart-header">
        <span>Timeline</span>
        <small>{metrics.length} samples</small>
      </div>
      <div class="chart">
        {#each metrics as metric, i (i)}
          <div
            class="chart-bar"
            style="height: {Math.min(100, (metric.cpuUsage || 0) * 2)}%"
            title="CPU: {metric.cpuUsage?.toFixed(1)}% | Memory: {metric.memoryUsage?.toFixed(1)}%"
          ></div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .performance-monitor {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
    background: var(--surface);
    border-radius: 12px;
    border: 1px solid var(--border);
  }

  .monitor-header {
    border-bottom: 1px solid var(--border);
    padding-bottom: 12px;
  }

  .monitor-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text);
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .metric-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: var(--bg);
    border-radius: 8px;
    border: 1px solid var(--hairline);
  }

  .metric-label {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .metric-value {
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
  }

  .metric-value.good {
    color: var(--green);
  }

  .metric-value.warning {
    color: var(--orange);
  }

  .metric-value.critical {
    color: var(--danger);
  }

  .metric-bar {
    height: 4px;
    background: var(--surface);
    border-radius: 2px;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .metric-note {
    font-size: 10px;
    color: var(--dim);
    font-family: var(--ff-mono);
  }

  .metrics-chart {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: var(--bg);
    border-radius: 8px;
    border: 1px solid var(--hairline);
  }

  .chart-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    color: var(--muted);
  }

  .chart {
    display: flex;
    align-items: flex-end;
    justify-content: space-around;
    height: 60px;
    gap: 2px;
  }

  .chart-bar {
    flex: 1;
    background: var(--blue);
    border-radius: 2px 2px 0 0;
    opacity: 0.7;
    transition: all 0.2s ease;
    min-height: 1px;
  }

  .chart-bar:hover {
    opacity: 1;
  }
</style>
