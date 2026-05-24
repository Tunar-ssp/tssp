import { writable } from "svelte/store";
import type { WorkspaceRecord } from "../api";

export interface OpenWorkspaceTab {
  id: string;
  label: string;
  dirty: boolean;
}

export const workspaceList = writable<WorkspaceRecord[]>([]);
export const openWorkspaceTabs = writable<OpenWorkspaceTab[]>([]);
export const activeWorkspaceTabId = writable<string | null>(null);
