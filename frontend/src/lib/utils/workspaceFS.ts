/**
 * Workspace Filesystem Utilities
 *
 * Handles path validation, normalization, and edge case resolution
 * with comprehensive security and usability checks.
 */

export interface ValidationResult {
  valid: boolean;
  error?: string;
}

/**
 * Validate file path for safety and usability
 * Checks for: empty paths, traversal attempts, absolute paths, invalid characters
 */
export function validateFilePath(path: string): ValidationResult {
  // Empty check
  if (!path || path.trim().length === 0) {
    return { valid: false, error: 'Path cannot be empty' };
  }

  const trimmed = path.trim();

  // Absolute path check
  if (trimmed.startsWith('/')) {
    return { valid: false, error: 'Path must be relative' };
  }

  // Path traversal check
  if (trimmed.includes('..') || trimmed.includes('~')) {
    return { valid: false, error: 'Path traversal not allowed' };
  }

  // Invalid characters
  if (/[\x00-\x1f]/.test(trimmed)) {
    return { valid: false, error: 'Path contains invalid characters' };
  }

  // Double slashes
  if (trimmed.includes('//')) {
    return { valid: false, error: 'Path contains double slashes' };
  }

  // Trailing slash (except for folders, but we validate at creation)
  if (trimmed.endsWith('/') && !trimmed.includes('/')) {
    // Single level directory is OK (e.g. "myfolder/")
  } else if (trimmed.endsWith('/')) {
    return { valid: false, error: 'Path should not end with slash' };
  }

  // Reserved names
  const reservedNames = ['.', '..', '.git', '.svn', 'node_modules'];
  const basename = trimmed.split('/')[0];
  if (reservedNames.includes(basename)) {
    return { valid: false, error: `Cannot use reserved name: ${basename}` };
  }

  return { valid: true };
}

/**
 * Normalize a path by removing redundant slashes and handling edge cases
 */
export function normalizePath(path: string): string {
  return path
    .trim()
    .replace(/\/+/g, '/') // Multiple slashes to single
    .replace(/\/$/, '') // Trailing slash
    .replace(/^\//, ''); // Leading slash
}

/**
 * Get file extension
 */
export function getFileExtension(path: string): string {
  const parts = path.split('.');
  return parts.length > 1 ? `.${parts[parts.length - 1]}` : '';
}

/**
 * Get base filename
 */
export function getFileName(path: string): string {
  return path.split('/').pop() || path;
}

/**
 * Get directory path
 */
export function getDirectoryPath(path: string): string {
  const parts = path.split('/');
  return parts.slice(0, -1).join('/');
}

/**
 * Check if path is likely binary based on extension
 */
export function isBinaryFile(path: string): boolean {
  const ext = getFileExtension(path).toLowerCase();
  const binaryExtensions = [
    '.bin', '.exe', '.dll', '.so', '.dylib',
    '.jpg', '.jpeg', '.png', '.gif', '.bmp', '.svg',
    '.mp3', '.mp4', '.wav', '.flac',
    '.zip', '.tar', '.gz', '.rar', '.7z',
    '.pdf', '.doc', '.docx', '.xls', '.xlsx',
    '.obj', '.o', '.a', '.pyc',
  ];
  return binaryExtensions.includes(ext);
}

/**
 * Get appropriate language for syntax highlighting
 */
export function detectLanguage(path: string): string {
  const ext = getFileExtension(path).toLowerCase();
  const languageMap: Record<string, string> = {
    '.js': 'javascript',
    '.ts': 'typescript',
    '.jsx': 'javascript',
    '.tsx': 'typescript',
    '.py': 'python',
    '.rs': 'rust',
    '.go': 'go',
    '.java': 'java',
    '.cs': 'csharp',
    '.cpp': 'cpp',
    '.c': 'c',
    '.h': 'c',
    '.html': 'html',
    '.htm': 'html',
    '.css': 'css',
    '.scss': 'scss',
    '.less': 'less',
    '.json': 'json',
    '.jsonc': 'json',
    '.yaml': 'yaml',
    '.yml': 'yaml',
    '.xml': 'xml',
    '.md': 'markdown',
    '.markdown': 'markdown',
    '.sql': 'sql',
    '.sh': 'bash',
    '.bash': 'bash',
    '.zsh': 'bash',
    '.fish': 'bash',
    '.pl': 'perl',
    '.rb': 'ruby',
    '.php': 'php',
    '.swift': 'swift',
    '.kt': 'kotlin',
    '.scala': 'scala',
    '.clj': 'clojure',
    '.ex': 'elixir',
    '.erl': 'erlang',
    '.lua': 'lua',
    '.vim': 'vim',
    '.diff': 'diff',
    '.patch': 'diff',
    '.txt': 'plaintext',
  };
  return languageMap[ext] || 'plaintext';
}

/**
 * Calculate file size display string
 */
export function formatFileSize(bytes: number | undefined): string {
  if (bytes === undefined || bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`;
}

/**
 * Check if file size is too large for editing in browser
 */
export function isFileTooLarge(sizeBytes: number | undefined): boolean {
  // 10MB threshold
  return (sizeBytes || 0) > 10 * 1024 * 1024;
}

/**
 * Suggest a new unique path when file exists
 */
export function suggestUniquePath(originalPath: string, index = 1): string {
  const dir = getDirectoryPath(originalPath);
  const name = getFileName(originalPath);
  const ext = getFileExtension(name);
  const base = name.slice(0, -ext.length);

  const newName = `${base} (${index})${ext}`;
  return dir ? `${dir}/${newName}` : newName;
}

/**
 * Check if two paths are equivalent (accounting for normalization)
 */
export function pathsEqual(path1: string, path2: string): boolean {
  return normalizePath(path1) === normalizePath(path2);
}

/**
 * Check if path is a child of parent path
 */
export function isChildPath(path: string, parentPath: string): boolean {
  const normalized = normalizePath(path);
  const normalizedParent = normalizePath(parentPath);
  return (
    normalized.startsWith(normalizedParent + '/') ||
    normalized === normalizedParent
  );
}

/**
 * Handle common editor edge cases and provide appropriate messages
 */
export function getEdgeWarning(
  filePath: string | null,
  fileSize: number | undefined,
  isDirty: boolean
): string | null {
  if (!filePath) return null;

  // Binary file warning
  if (isBinaryFile(filePath)) {
    return '⚠ Binary file - display is not supported';
  }

  // File too large warning
  if (isFileTooLarge(fileSize)) {
    return '⚠ File is very large - editing may be slow';
  }

  // Unsaved changes warning
  if (isDirty) {
    return '●  You have unsaved changes';
  }

  return null;
}

/**
 * Suggest appropriate action for error
 */
export function suggestErrorRecovery(errorCode: string): string {
  const suggestions: Record<string, string> = {
    'LOAD_FILE_TREE_FAILED': 'Try refreshing the page or checking your internet connection',
    'READ_FILE_FAILED': 'The file may have been deleted or you may not have permission',
    'WRITE_FILE_FAILED': 'Check your internet connection and available storage',
    'CREATE_FILE_FAILED': 'The file may already exist or path is invalid',
    'CREATE_DIR_FAILED': 'The directory may already exist',
    'DELETE_FILE_FAILED': 'The file may be in use or you may not have permission',
    'MOVE_FILE_FAILED': 'The destination path may be invalid or file may be in use',
    'VALIDATION_ERROR': 'Check the path for invalid characters or spaces',
  };
  return suggestions[errorCode] || 'Please try again or contact support';
}
