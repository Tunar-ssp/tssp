/**
 * Sharing Service
 *
 * Handles all sharing-related operations: public links, visibility management, QR code generation.
 * Extracted from SharingModal.svelte and related components.
 *
 * Features:
 * - Generate public share links
 * - Revoke share links
 * - Update file visibility (public/private)
 * - QR code generation for sharing
 * - Link expiration handling
 * - Permission validation
 *
 * Error Handling:
 * - Invalid file IDs
 * - Permission denied
 * - Link generation failures
 * - Visibility update conflicts
 *
 * Edge Cases:
 * - Expired share links
 * - Multiple shares for same file
 * - Permission mismatches
 * - Concurrent visibility changes
 */

import type { FileRecord } from '$lib/api';
import { api } from '$lib/api';

function log(context: string, message: string, data?: any) {
  console.debug(`[sharingService] ${context}: ${message}`, data || '');
}

class SharingServiceError extends Error {
  constructor(
    public code: string,
    message: string,
    public originalError?: Error
  ) {
    super(message);
    this.name = 'SharingServiceError';
  }
}

/**
 * Share link representation
 */
export interface ShareLink {
  id: string;
  url: string;
  expires_at?: number;
  created_at: number;
  access_count: number;
}

/**
 * Generate a public share link for a file
 * @param fileId The ID of the file to share
 * @returns Share link with URL
 */
export async function generateShareLink(fileId: string): Promise<ShareLink> {
  log('generateShareLink', 'Starting', { fileId });

  try {
    if (!fileId?.trim()) {
      throw new SharingServiceError('VALIDATION_ERROR', 'File ID required');
    }

    // Generate link via API
    const result = await api.generateShareLink(fileId) as any;

    if (!result?.url) {
      throw new SharingServiceError(
        'INVALID_RESPONSE',
        'Server returned invalid link data'
      );
    }

    const link: ShareLink = {
      id: result.id || generateLinkId(),
      url: result.url,
      expires_at: result.expires_at || undefined,
      created_at: Math.floor(Date.now() / 1000),
      access_count: 0,
    };

    log('generateShareLink', 'Success', { fileId, linkId: link.id });
    return link;
  } catch (err) {
    const message = err instanceof SharingServiceError
      ? err.message
      : 'Failed to generate share link';

    log('generateShareLink', 'Error', { error: message, fileId });
    throw new SharingServiceError(
      'LINK_GENERATION_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Revoke a share link
 */
export async function revokeShareLink(fileId: string, linkId: string): Promise<void> {
  log('revokeShareLink', 'Starting', { fileId, linkId });

  try {
    if (!fileId?.trim()) {
      throw new SharingServiceError('VALIDATION_ERROR', 'File ID required');
    }

    if (!linkId?.trim()) {
      throw new SharingServiceError('VALIDATION_ERROR', 'Link ID required');
    }

    await api.revokeShareLink(fileId, linkId);

    log('revokeShareLink', 'Success', { fileId, linkId });
  } catch (err) {
    const message = err instanceof SharingServiceError
      ? err.message
      : 'Failed to revoke share link';

    log('revokeShareLink', 'Error', { error: message, fileId, linkId });
    throw new SharingServiceError(
      'REVOKE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Update file visibility (public/private)
 * @param fileId File to update
 * @param isPublic True for public, false for private
 */
export async function updateVisibility(
  fileId: string,
  isPublic: boolean
): Promise<FileRecord> {
  log('updateVisibility', 'Starting', { fileId, isPublic });

  try {
    if (!fileId?.trim()) {
      throw new SharingServiceError('VALIDATION_ERROR', 'File ID required');
    }

    const file = await api.setFileVisibility(fileId, isPublic) as any;

    if (!file?.id) {
      throw new SharingServiceError(
        'INVALID_RESPONSE',
        'Server returned invalid file data'
      );
    }

    log('updateVisibility', 'Success', { fileId, isPublic });
    return file as FileRecord;
  } catch (err) {
    const message = err instanceof SharingServiceError
      ? err.message
      : 'Failed to update visibility';

    log('updateVisibility', 'Error', { error: message, fileId });
    throw new SharingServiceError(
      'VISIBILITY_UPDATE_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Get list of share links for a file
 */
export async function getShareLinks(fileId: string): Promise<ShareLink[]> {
  log('getShareLinks', 'Starting', { fileId });

  try {
    if (!fileId?.trim()) {
      throw new SharingServiceError('VALIDATION_ERROR', 'File ID required');
    }

    const result = await api.listShareLinks(fileId) as any;
    const links: ShareLink[] = (result.links || []).map((link: any) => ({
      id: link.id || '',
      url: link.url || '',
      expires_at: link.expires_at,
      created_at: link.created_at || Math.floor(Date.now() / 1000),
      access_count: link.access_count || 0,
    }));

    log('getShareLinks', 'Success', { fileId, count: links.length });
    return links;
  } catch (err) {
    log('getShareLinks', 'Error', { fileId });
    return []; // Return empty instead of throwing
  }
}

/**
 * Copy share link to clipboard
 * Handles browser clipboard API with fallbacks
 */
export async function copyLinkToClipboard(url: string): Promise<boolean> {
  log('copyLinkToClipboard', 'Starting', { urlLength: url.length });

  try {
    if (!url || url.length === 0) {
      throw new SharingServiceError('VALIDATION_ERROR', 'URL required');
    }

    // Try modern Clipboard API first
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(url);
      log('copyLinkToClipboard', 'Success via Clipboard API');
      return true;
    }

    // Fallback: Create temporary textarea
    const textarea = document.createElement('textarea');
    textarea.value = url;
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    document.body.appendChild(textarea);

    textarea.select();
    const success = document.execCommand('copy');
    document.body.removeChild(textarea);

    if (success) {
      log('copyLinkToClipboard', 'Success via execCommand');
      return true;
    }

    throw new SharingServiceError(
      'CLIPBOARD_ACCESS_DENIED',
      'Could not copy to clipboard'
    );
  } catch (err) {
    log('copyLinkToClipboard', 'Error');
    return false;
  }
}

/**
 * Generate QR code data URL for a share link
 * Returns a data URL that can be displayed as an image
 */
export async function generateQRCode(url: string): Promise<string> {
  log('generateQRCode', 'Starting', { urlLength: url.length });

  try {
    if (!url || url.length === 0) {
      throw new SharingServiceError('VALIDATION_ERROR', 'URL required');
    }

    // Dynamic import of qrcode library to keep bundle small
    const QRCode = (await import('qrcode')).default;

    // Generate QR code as data URL
    const dataUrl: string = await (QRCode.toDataURL(url, {
      errorCorrectionLevel: 'H',
      margin: 1,
      width: 300,
      color: { dark: '#000000', light: '#FFFFFF' },
    }) as Promise<string>);

    log('generateQRCode', 'Success');
    return dataUrl;
  } catch (err) {
    const message = 'Failed to generate QR code';
    log('generateQRCode', 'Error', { message });
    throw new SharingServiceError(
      'QR_GENERATION_FAILED',
      message,
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Format a share URL for display
 * Removes protocol and trailing slashes
 */
export function formatShareUrl(url: string): string {
  try {
    const urlObj = new URL(url);
    return urlObj.host + urlObj.pathname;
  } catch {
    // If URL is invalid, return as-is
    return url.replace(/^https?:\/\//, '').replace(/\/$/, '');
  }
}

/**
 * Check if a share link has expired
 */
export function isLinkExpired(link: ShareLink): boolean {
  if (!link.expires_at) return false;

  const now = Math.floor(Date.now() / 1000);
  return now > link.expires_at;
}

/**
 * Calculate time remaining until link expiration
 * Returns human-readable string
 */
export function getTimeUntilExpiration(link: ShareLink): string {
  if (!link.expires_at) return 'Never';

  const now = Math.floor(Date.now() / 1000);
  const remaining = link.expires_at - now;

  if (remaining <= 0) return 'Expired';

  const seconds = remaining;
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days}d`;
  if (hours > 0) return `${hours}h`;
  if (minutes > 0) return `${minutes}m`;
  return `${seconds}s`;
}

/**
 * Generate unique link ID for client-side tracking
 * (Server will generate its own ID)
 */
function generateLinkId(): string {
  return `link_${Date.now()}_${Math.random().toString(36).slice(2)}`;
}

/**
 * Get share information for display
 * Combines file info with share stats
 */
export interface ShareInfo {
  fileId: string;
  fileName: string;
  isPublic: boolean;
  links: ShareLink[];
  totalViews: number;
  createdAt: number;
}

export { SharingServiceError };
