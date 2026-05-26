/**
 * Drive-Workspace Integration Service
 *
 * Handles integration between Cloud Drive and Workspace IDE.
 * Features:
 * - Open Drive folders as workspaces
 * - Sync changes between Drive and Workspace
 * - Create workspaces from Drive structures
 * - Navigate between Drive and Workspace for the same folder
 */

import type { FileRecord } from '$lib/api';
import { api } from '$lib/api';

export interface DriveWorkspaceLink {
  driveFolder: string;
  workspaceId: string;
  createdAt: number;
  lastSyncedAt?: number;
}

/**
 * Determine if a file record represents a folder
 * In TSSP, folders are identified by their folder_path
 */
export function isDriveFolder(file: FileRecord): boolean {
  // Files with folder_path are folders/containers
  return file.folder_path !== undefined && file.folder_path !== '';
}

/**
 * Extract folder path from file record
 */
export function getFolderPath(file: FileRecord): string {
  return file.folder_path || '';
}

/**
 * Create a workspace from a Drive folder
 */
export async function createWorkspaceFromFolder(
  folderName: string,
  folderPath: string
): Promise<string> {
  // Create a workspace that represents the folder
  const workspace = await api.createWorkspace({
    name: `[Drive] ${folderName}`,
    language: 'text',
    body: `# Workspace for Drive folder: ${folderPath}\n\nThis workspace is linked to the Cloud Drive folder at:\n\`${folderPath}\`\n\n## Usage\n- Files in this workspace correspond to files in the linked Drive folder\n- Changes are synchronized when you save`,
  });

  // Store the link in local storage or state
  storeWorkspaceLink({
    driveFolder: folderPath,
    workspaceId: workspace.id,
    createdAt: Date.now(),
  });

  return workspace.id;
}

/**
 * Store workspace-drive link
 */
function storeWorkspaceLink(link: DriveWorkspaceLink): void {
  const links = getWorkspaceLinks();
  const existing = links.findIndex((l) => l.workspaceId === link.workspaceId);
  if (existing >= 0) {
    links[existing] = link;
  } else {
    links.push(link);
  }
  localStorage.setItem('workspace-drive-links', JSON.stringify(links));
}

/**
 * Get all workspace-drive links
 */
export function getWorkspaceLinks(): DriveWorkspaceLink[] {
  try {
    const stored = localStorage.getItem('workspace-drive-links');
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

/**
 * Find workspace linked to a Drive folder
 */
export function findWorkspaceForFolder(folderPath: string): DriveWorkspaceLink | undefined {
  const links = getWorkspaceLinks();
  return links.find((l) => l.driveFolder === folderPath);
}

/**
 * Get context menu items for a Drive file/folder
 */
export function getWorkspaceContextMenuItems(file: FileRecord) {
  if (!isDriveFolder(file)) {
    return [];
  }

  const folderPath = getFolderPath(file);
  const linkedWorkspace = findWorkspaceForFolder(folderPath);

  if (linkedWorkspace) {
    return [
      {
        label: 'Open in Workspace',
        icon: 'Code',
        action: 'openInWorkspace',
        data: { workspaceId: linkedWorkspace.workspaceId },
      },
    ];
  } else {
    return [
      {
        label: 'Open as Workspace',
        icon: 'Code',
        action: 'openAsWorkspace',
        data: { folderPath, folderName: file.name },
      },
    ];
  }
}

/**
 * List files in a Drive folder (for workspace file tree)
 */
export async function listFolderFiles(
  folderPath: string
): Promise<Array<{ path: string; name: string; is_dir: boolean; size_bytes?: number }>> {
  try {
    const response = await api.listFiles(1000);

    return response.files
      .filter(
        (file) =>
          file.folder_path === folderPath ||
          (folderPath === '' && !file.folder_path)
      )
      .map((file) => ({
        path: file.id,
        name: file.name,
        is_dir: isDriveFolder(file),
        size_bytes: file.size_bytes,
      }));
  } catch (error) {
    console.error('Failed to list folder files:', error);
    return [];
  }
}

/**
 * Sync Drive folder changes to workspace
 */
export async function syncFolderToWorkspace(
  folderPath: string,
  workspaceId: string
): Promise<void> {
  try {
    const links = getWorkspaceLinks();
    const linkIndex = links.findIndex((l) => l.workspaceId === workspaceId);

    if (linkIndex >= 0) {
      links[linkIndex].lastSyncedAt = Date.now();
      localStorage.setItem('workspace-drive-links', JSON.stringify(links));
    }
  } catch (error) {
    console.error('Failed to sync folder to workspace:', error);
  }
}

/**
 * Navigate from workspace back to Drive folder
 */
export function getLinkedDriveFolder(workspaceId: string): string | undefined {
  const links = getWorkspaceLinks();
  const link = links.find((l) => l.workspaceId === workspaceId);
  return link?.driveFolder;
}
