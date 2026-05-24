import { writable, derived } from "svelte/store";
import type { FileRecord, FolderEntry, ShareInfo } from "../api";

export type DriveLens = "all" | "images" | "videos" | "documents";
export type DriveViewMode = "grid" | "table";
export type DriveSort = "name" | "date" | "size";

export const driveLens = writable<DriveLens>("all");
export const driveFolder = writable("");
export const driveViewMode = writable<DriveViewMode>("grid");
export const driveSort = writable<DriveSort>("date");
export const driveQuery = writable("");
export const driveFiles = writable<FileRecord[]>([]);
export const driveFolders = writable<FolderEntry[]>([]);
export const driveLoading = writable(false);
export const driveError = writable("");
export const selectedIds = writable<Set<string>>(new Set());
export const focusedFileId = writable<string | null>(null);
export const previewFileId = writable<string | null>(null);
export const shareInfo = writable<ShareInfo | null>(null);
export const uploadDragOver = writable(false);

export const selectedCount = derived(selectedIds, ($s) => $s.size);
export const focusedFile = derived(
  [driveFiles, focusedFileId],
  ([$files, $id]) => $files.find((f) => f.id === $id) || null,
);

export function toggleSelection(id: string, checked: boolean) {
  selectedIds.update((set) => {
    const next = new Set(set);
    if (checked) next.add(id);
    else next.delete(id);
    return next;
  });
}

export function clearSelection() {
  selectedIds.set(new Set());
}

export function selectAll(ids: string[]) {
  selectedIds.set(new Set(ids));
}
