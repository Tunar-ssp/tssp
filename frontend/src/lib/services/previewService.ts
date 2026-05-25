/**
 * Preview Service
 *
 * Handles file preview generation for different content types.
 * Manages image thumbnails, text preview, code syntax highlighting, etc.
 *
 * Features:
 * - Image preview and thumbnail generation
 * - Text file preview with truncation
 * - Code syntax detection
 * - Document preview fallbacks
 * - Lazy loading and caching
 * - Error recovery
 *
 * Edge Cases:
 * - Corrupted/invalid files
 * - Unsupported file types
 * - Very large files (preview only)
 * - Missing preview data
 * - Network timeouts
 */

import type { FileRecord } from '$lib/api';
import { api } from '$lib/api';

function log(context: string, message: string, data?: any) {
  console.debug(`[previewService] ${context}: ${message}`, data || '');
}

class PreviewServiceError extends Error {
  constructor(
    public code: string,
    message: string,
    public originalError?: Error
  ) {
    super(message);
    this.name = 'PreviewServiceError';
  }
}

// Configuration
const MAX_PREVIEW_TEXT_LENGTH = 10_000; // Show first 10KB of text
const TEXT_PREVIEW_TIMEOUT_MS = 3000;
const THUMBNAIL_SIZE = 300; // pixels

/**
 * Preview data for a file
 */
export interface FilePreview {
  type: 'image' | 'text' | 'code' | 'document' | 'unknown';
  data?: string; // Base64 for images, text content for text
  language?: string; // For code files
  size: number; // Original file size
  truncated: boolean; // Whether preview was truncated
  error?: string; // Error message if preview failed
}

/**
 * Determine file type from MIME type
 */
function getFileType(mimeType: string): string {
  if (!mimeType) return 'unknown';

  if (mimeType.startsWith('image/')) return 'image';
  if (mimeType.startsWith('text/')) return 'text';
  if (mimeType.includes('json') || mimeType.includes('xml')) return 'code';
  if (mimeType.includes('javascript') || mimeType.includes('typescript')) return 'code';
  if (mimeType.includes('pdf') || mimeType.includes('document')) return 'document';

  return 'unknown';
}

/**
 * Detect code language from MIME type or filename
 */
function detectLanguage(mimeType: string, filename: string): string | undefined {
  const ext = filename.split('.').pop()?.toLowerCase();

  const mimeMap: Record<string, string> = {
    'javascript': 'javascript',
    'typescript': 'typescript',
    'python': 'python',
    'rust': 'rust',
    'go': 'go',
    'bash': 'bash',
    'shell': 'bash',
    'json': 'json',
    'xml': 'xml',
    'html': 'html',
    'css': 'css',
    'sql': 'sql',
    'yaml': 'yaml',
    'markdown': 'markdown',
  };

  // Check MIME type
  for (const [key, lang] of Object.entries(mimeMap)) {
    if (mimeType.includes(key)) return lang;
  }

  // Check file extension
  if (ext && mimeMap[ext]) {
    return mimeMap[ext];
  }

  return undefined;
}

/**
 * Generate preview for an image file
 * For now, returns URL to load as image
 */
async function previewImage(
  file: FileRecord
): Promise<FilePreview> {
  log('previewImage', 'Starting', { id: file.id, size: file.size_bytes });

  try {
    // For images, we just need the URL
    // The UI will load it directly from the file download URL
    return {
      type: 'image',
      data: await api.getFileDownloadUrl(file.id),
      size: file.size_bytes || 0,
      truncated: false,
    };
  } catch (err) {
    log('previewImage', 'Error', { error: err });
    throw new PreviewServiceError(
      'IMAGE_PREVIEW_FAILED',
      'Could not generate image preview',
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Generate preview for a text file
 * Fetches and truncates content
 */
async function previewText(
  file: FileRecord
): Promise<FilePreview> {
  log('previewText', 'Starting', { id: file.id, size: file.size_bytes });

  try {
    // Fetch file content
    const content = await api.getFileContent(file.id);

    if (!content) {
      return {
        type: 'text',
        data: '(empty file)',
        size: file.size_bytes || 0,
        truncated: false,
      };
    }

    // Detect if it's code
    const language = detectLanguage(file.mime_type, file.name);
    const type = language ? 'code' : 'text';

    // Truncate if needed
    const truncated = content.length > MAX_PREVIEW_TEXT_LENGTH;
    const preview = truncated
      ? content.slice(0, MAX_PREVIEW_TEXT_LENGTH) + '\n...(truncated)'
      : content;

    return {
      type,
      data: preview,
      language,
      size: file.size_bytes || 0,
      truncated,
    };
  } catch (err) {
    log('previewText', 'Error', { error: err });
    throw new PreviewServiceError(
      'TEXT_PREVIEW_FAILED',
      'Could not fetch text preview',
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Generate preview for code file
 * Similar to text but with language detection
 */
async function previewCode(
  file: FileRecord
): Promise<FilePreview> {
  log('previewCode', 'Starting', { id: file.id });

  try {
    const language = detectLanguage(file.mime_type, file.name) || 'plaintext';

    const content = await api.getFileContent(file.id);

    if (!content) {
      return {
        type: 'code',
        data: '(empty file)',
        language,
        size: file.size_bytes || 0,
        truncated: false,
      };
    }

    const truncated = content.length > MAX_PREVIEW_TEXT_LENGTH;
    const preview = truncated
      ? content.slice(0, MAX_PREVIEW_TEXT_LENGTH) + '\n...(truncated)'
      : content;

    return {
      type: 'code',
      data: preview,
      language,
      size: file.size_bytes || 0,
      truncated,
    };
  } catch (err) {
    log('previewCode', 'Error', { error: err });
    throw new PreviewServiceError(
      'CODE_PREVIEW_FAILED',
      'Could not fetch code preview',
      err instanceof Error ? err : undefined
    );
  }
}

/**
 * Generate preview for document
 * Returns metadata only (actual preview requires document parser)
 */
async function previewDocument(
  file: FileRecord
): Promise<FilePreview> {
  log('previewDocument', 'Starting', { id: file.id, mime: file.mime_type });

  // For now, return document metadata
  // Full document preview would require specialized parser
  return {
    type: 'document',
    data: `${file.mime_type} document\n\nSize: ${formatBytes(file.size_bytes || 0)}\nUploaded: ${new Date(
      (file.uploaded_at || 0) * 1000
    ).toLocaleDateString()}`,
    size: file.size_bytes || 0,
    truncated: false,
  };
}

/**
 * Generate preview for unknown file type
 */
function previewUnknown(file: FileRecord): FilePreview {
  return {
    type: 'unknown',
    data: `File type not supported for preview\n\n${file.name}\n${file.mime_type || 'Unknown type'}\nSize: ${formatBytes(file.size_bytes || 0)}`,
    size: file.size_bytes || 0,
    truncated: false,
  };
}

/**
 * Main preview generation function
 * Routes to appropriate preview handler based on file type
 */
export async function generatePreview(file: FileRecord): Promise<FilePreview> {
  log('generatePreview', 'Starting', { id: file.id, mime: file.mime_type });

  try {
    if (!file?.id) {
      throw new PreviewServiceError('VALIDATION_ERROR', 'File data required');
    }

    const fileType = getFileType(file.mime_type);

    switch (fileType) {
      case 'image':
        return await previewImage(file);

      case 'text':
        return await previewText(file);

      case 'code':
        return await previewCode(file);

      case 'document':
        return await previewDocument(file);

      default:
        return previewUnknown(file);
    }
  } catch (err) {
    const message = err instanceof PreviewServiceError
      ? err.message
      : 'Failed to generate preview';

    log('generatePreview', 'Error', { error: message, fileId: file?.id });

    // Return error preview instead of throwing
    return {
      type: 'unknown',
      data: message,
      size: file?.size_bytes || 0,
      truncated: false,
      error: message,
    };
  }
}

/**
 * Check if a file type is previewable
 */
export function isPreviewable(file: FileRecord): boolean {
  const type = getFileType(file.mime_type);
  return type !== 'unknown';
}

/**
 * Get preview icon for file type
 */
export function getPreviewIcon(type: string): string {
  switch (type) {
    case 'image':
      return '🖼️';
    case 'text':
      return '📄';
    case 'code':
      return '</>';
    case 'document':
      return '📑';
    default:
      return '📎';
  }
}

/**
 * Format bytes to human-readable string
 */
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

export { PreviewServiceError };
