/**
 * Unified utility exports
 * Consolidates all utility functions for easier imports
 */

// Formatting utilities
export { formatBytes, formatRelative, formatRelativeDate, formatAbsoluteDate } from './format';

// Markdown utilities
export { renderMarkdownLite } from './markdown';

// File utilities
export {
  matchesDriveLens,
  fileKindLabel,
  fileIconLabel,
  buildFolderEntries,
  inferFilePreviewUrl,
  type DriveLens,
  type FolderEntry,
} from './files';

// Keyboard utilities
export { registerShortcuts } from './keyboard';

// Workspace utilities
export type { WorkspaceTreeNode } from './workspace';
export { inferLanguageFromPath, buildWorkspaceTree } from './workspace';


export function getWordCount(text: string): number {
  return text.trim().split(/\s+/).filter((word) => word.length > 0).length;
}

export function getLineCount(text: string): number {
  return text.split('\n').length;
}

export function getCharCount(text: string): number {
  return text.length;
}

export function truncateString(str: string, maxLength: number, suffix = '...'): string {
  if (str.length <= maxLength) return str;
  return str.slice(0, maxLength - suffix.length) + suffix;
}

export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | null = null;

  return function executedFunction(...args: Parameters<T>) {
    const later = () => {
      timeout = null;
      func(...args);
    };

    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean = false;

  return function executedFunction(...args: Parameters<T>) {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => {
        inThrottle = false;
      }, limit);
    }
  };
}
