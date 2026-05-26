/**
 * Workspace Filesystem Integration Tests
 *
 * Tests path validation, normalization, error handling, and edge cases
 */

import { describe, it, expect } from 'vitest';
import {
  validateFilePath,
  normalizePath,
  getFileExtension,
  getFileName,
  getDirectoryPath,
  isBinaryFile,
  detectLanguage,
  formatFileSize,
  isFileTooLarge,
  suggestUniquePath,
  pathsEqual,
  isChildPath,
} from '$lib/utils/workspaceFS';

describe('Path Validation', () => {
  it('should reject empty paths', () => {
    const result = validateFilePath('');
    expect(result.valid).toBe(false);
    expect(result.error).toContain('empty');
  });

  it('should reject absolute paths', () => {
    const result = validateFilePath('/absolute/path');
    expect(result.valid).toBe(false);
    expect(result.error).toContain('relative');
  });

  it('should reject path traversal attempts', () => {
    const cases = ['../etc/passwd', 'a/../../b', 'folder/..'];
    cases.forEach(path => {
      const result = validateFilePath(path);
      expect(result.valid).toBe(false);
    });
  });

  it('should reject reserved names', () => {
    const result = validateFilePath('.git/config');
    expect(result.valid).toBe(false);
  });

  it('should accept valid relative paths', () => {
    const cases = ['file.txt', 'folder/file.txt', 'a/b/c/file.txt'];
    cases.forEach(path => {
      const result = validateFilePath(path);
      expect(result.valid).toBe(true);
    });
  });

  it('should reject paths with control characters', () => {
    const result = validateFilePath('file\x00.txt');
    expect(result.valid).toBe(false);
  });

  it('should reject paths with double slashes', () => {
    const result = validateFilePath('folder//file.txt');
    expect(result.valid).toBe(false);
  });
});

describe('Path Normalization', () => {
  it('should normalize multiple slashes', () => {
    expect(normalizePath('a//b///c')).toBe('a/b/c');
  });

  it('should remove trailing slashes', () => {
    expect(normalizePath('folder/')).toBe('folder');
    expect(normalizePath('folder///')).toBe('folder');
  });

  it('should remove leading slashes', () => {
    expect(normalizePath('/a/b')).toBe('a/b');
  });

  it('should trim whitespace', () => {
    expect(normalizePath('  a/b  ')).toBe('a/b');
  });
});

describe('File Operations', () => {
  it('should extract file extension', () => {
    expect(getFileExtension('file.txt')).toBe('.txt');
    expect(getFileExtension('archive.tar.gz')).toBe('.gz');
    expect(getFileExtension('no-extension')).toBe('');
  });

  it('should get filename from path', () => {
    expect(getFileName('folder/file.txt')).toBe('file.txt');
    expect(getFileName('a/b/c/file')).toBe('file');
    expect(getFileName('file.txt')).toBe('file.txt');
  });

  it('should get directory path', () => {
    expect(getDirectoryPath('folder/file.txt')).toBe('folder');
    expect(getDirectoryPath('a/b/c/file.txt')).toBe('a/b/c');
    expect(getDirectoryPath('file.txt')).toBe('');
  });
});

describe('File Type Detection', () => {
  it('should detect binary files', () => {
    const binaryFiles = ['image.jpg', 'video.mp4', 'archive.zip', 'executable.exe'];
    binaryFiles.forEach(file => {
      expect(isBinaryFile(file)).toBe(true);
    });
  });

  it('should detect text files', () => {
    const textFiles = ['code.js', 'config.json', 'note.md', 'script.sh'];
    textFiles.forEach(file => {
      expect(isBinaryFile(file)).toBe(false);
    });
  });

  it('should detect language from extension', () => {
    expect(detectLanguage('file.js')).toBe('javascript');
    expect(detectLanguage('file.ts')).toBe('typescript');
    expect(detectLanguage('file.py')).toBe('python');
    expect(detectLanguage('file.rs')).toBe('rust');
    expect(detectLanguage('file.md')).toBe('markdown');
    expect(detectLanguage('unknown.xyz')).toBe('plaintext');
  });
});

describe('File Size Handling', () => {
  it('should format file sizes correctly', () => {
    expect(formatFileSize(0)).toBe('0 B');
    expect(formatFileSize(1024)).toBe('1.0 KB');
    expect(formatFileSize(1024 * 1024)).toBe('1.0 MB');
    expect(formatFileSize(1024 * 1024 * 1024)).toBe('1.0 GB');
  });

  it('should detect files that are too large for editing', () => {
    expect(isFileTooLarge(5 * 1024 * 1024)).toBe(false);
    expect(isFileTooLarge(15 * 1024 * 1024)).toBe(true);
    expect(isFileTooLarge(undefined)).toBe(false);
  });
});

describe('Path Manipulation', () => {
  it('should suggest unique paths for conflicts', () => {
    const unique = suggestUniquePath('folder/file.txt');
    expect(unique).toContain('(1)');
    expect(unique).toMatch(/\.txt$/);
  });

  it('should compare paths correctly', () => {
    expect(pathsEqual('a/b/c', 'a/b/c')).toBe(true);
    expect(pathsEqual('a/b/c', 'a/b/c/')).toBe(true);
    expect(pathsEqual('a/b/c', 'a/b/d')).toBe(false);
  });

  it('should check child paths', () => {
    expect(isChildPath('a/b/c/file.txt', 'a/b')).toBe(true);
    expect(isChildPath('a/b', 'a/b')).toBe(true);
    expect(isChildPath('a/c', 'a/b')).toBe(false);
  });
});

describe('Edge Cases', () => {
  it('should handle deeply nested paths', () => {
    const deepPath = 'a/b/c/d/e/f/g/h/i/j/file.txt';
    const validation = validateFilePath(deepPath);
    expect(validation.valid).toBe(true);
  });

  it('should handle special characters in filenames', () => {
    const cases = ['file-with-dashes.txt', 'file_with_underscores.txt', 'file (with spaces).txt'];
    cases.forEach(path => {
      const validation = validateFilePath(path);
      expect(validation.valid).toBe(true);
    });
  });

  it('should handle unicode filenames', () => {
    const result = validateFilePath('文件.txt');
    expect(result.valid).toBe(true);

    const result2 = validateFilePath('файл.txt');
    expect(result2.valid).toBe(true);
  });

  it('should handle very long filenames', () => {
    const longName = 'a'.repeat(255) + '.txt';
    const result = validateFilePath(longName);
    // Path validation doesn't check length, that's backend's job
    expect(result.valid).toBe(true);
  });
});
