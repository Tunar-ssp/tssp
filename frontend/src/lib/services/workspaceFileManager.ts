/**
 * Workspace File Manager
 *
 * Manages files within workspaces including:
 * - File tree structure
 * - File operations (create, delete, rename, read, write)
 * - File content caching
 * - Dirty state tracking
 */

export interface WorkspaceFile {
  id: string;
  path: string;
  name: string;
  isDir: boolean;
  content?: string;
  isDirty?: boolean;
  language?: string;
  sizeBytes?: number;
}

export interface FileTreeNode extends WorkspaceFile {
  children?: FileTreeNode[];
}

export interface FileOperation {
  type: 'create' | 'delete' | 'rename' | 'write';
  path: string;
  oldPath?: string;
  content?: string;
  timestamp: number;
}

const fileCache = new Map<string, WorkspaceFile>();
const operationHistory: FileOperation[] = [];

export function createFile(
  path: string,
  name: string,
  content: string = '',
  language: string = 'text'
): WorkspaceFile {
  const file: WorkspaceFile = {
    id: `file-${Date.now()}-${Math.random().toString(36).slice(2)}`,
    path,
    name,
    isDir: false,
    content,
    language,
    isDirty: true,
  };

  fileCache.set(file.id, file);
  operationHistory.push({
    type: 'create',
    path,
    content,
    timestamp: Date.now(),
  });

  return file;
}

export function deleteFile(fileId: string): boolean {
  const file = fileCache.get(fileId);
  if (!file) return false;

  operationHistory.push({
    type: 'delete',
    path: file.path,
    timestamp: Date.now(),
  });

  fileCache.delete(fileId);
  return true;
}

export function renameFile(fileId: string, newName: string): WorkspaceFile | null {
  const file = fileCache.get(fileId);
  if (!file) return null;

  const oldPath = file.path;
  const newPath = file.path.split('/').slice(0, -1).concat(newName).join('/');

  const updated: WorkspaceFile = {
    ...file,
    name: newName,
    path: newPath,
    isDirty: true,
  };

  fileCache.set(fileId, updated);
  operationHistory.push({
    type: 'rename',
    path: newPath,
    oldPath,
    timestamp: Date.now(),
  });

  return updated;
}

export function writeFile(fileId: string, content: string): WorkspaceFile | null {
  const file = fileCache.get(fileId);
  if (!file) return null;

  const updated: WorkspaceFile = {
    ...file,
    content,
    isDirty: true,
  };

  fileCache.set(fileId, updated);
  operationHistory.push({
    type: 'write',
    path: file.path,
    content,
    timestamp: Date.now(),
  });

  return updated;
}

export function readFile(fileId: string): string | null {
  const file = fileCache.get(fileId);
  return file?.content || null;
}

export function getFile(fileId: string): WorkspaceFile | null {
  return fileCache.get(fileId) || null;
}

export function markFileSaved(fileId: string): void {
  const file = fileCache.get(fileId);
  if (file) {
    file.isDirty = false;
  }
}

export function buildFileTree(files: WorkspaceFile[]): FileTreeNode[] {
  const roots: Map<string, FileTreeNode> = new Map();
  const nodeMap: Map<string, FileTreeNode> = new Map();

  files.forEach((file) => {
    const node: FileTreeNode = { ...file };
    nodeMap.set(file.id, node);
  });

  files.forEach((file) => {
    const parts = file.path.split('/').filter(Boolean);
    if (parts.length === 0) {
      roots.set(file.id, nodeMap.get(file.id)!);
    }
  });

  return Array.from(roots.values());
}

export function getAllFiles(): WorkspaceFile[] {
  return Array.from(fileCache.values());
}

export function clearCache(): void {
  fileCache.clear();
  operationHistory.length = 0;
}

export function getOperationHistory(): FileOperation[] {
  return [...operationHistory];
}
