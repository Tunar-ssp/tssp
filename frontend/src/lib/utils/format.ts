export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value >= 10 || unitIndex === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unitIndex]}`;
}

export function toDate(value: string | number | Date | null | undefined): Date | null {
  if (value == null) return null;
  if (value instanceof Date) return Number.isNaN(value.valueOf()) ? null : value;
  if (typeof value === "number") {
    const millis = value < 10_000_000_000 ? value * 1000 : value;
    const date = new Date(millis);
    return Number.isNaN(date.valueOf()) ? null : date;
  }
  const date = new Date(value);
  return Number.isNaN(date.valueOf()) ? null : date;
}

export function formatRelativeDate(value: string | number | Date): string {
  const date = toDate(value);
  if (!date) return String(value);
  const minutes = Math.round((Date.now() - date.valueOf()) / 60000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.round(hours / 24);
  return `${days}d ago`;
}

export function formatAbsoluteDate(value: string | number | Date | null | undefined): string {
  const date = toDate(value);
  if (!date) return value == null ? "—" : String(value);
  return date.toLocaleString();
}

export function formatRelative(timestamp?: number): string {
  if (!timestamp) return 'just now';
  const delta = Math.max(0, Math.floor(Date.now() / 1000) - timestamp);
  if (delta < 60) return 'just now';
  if (delta < 3600) return `${Math.floor(delta / 60)}m`;
  if (delta < 86400) return `${Math.floor(delta / 3600)}h`;
  if (delta < 604800) return `${Math.floor(delta / 86400)}d`;
  return `${Math.floor(delta / 604800)}w`;
}

export function formatDate(value: string | number | Date | undefined | null): string {
  if (!value) return '—';
  const date = toDate(value as any);
  if (!date) return String(value);
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

export function getWordCount(text: string): number {
  return text.trim().split(/\s+/).filter((word) => word.length > 0).length;
}
