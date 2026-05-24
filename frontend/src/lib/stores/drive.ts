import { writable } from "svelte/store";
import type { FileRecord } from "../api";

export type DriveViewMode = "grid" | "table";

export const driveFolder = writable("default");
export const driveViewMode = writable<DriveViewMode>("grid");
export const selectedFiles = writable<FileRecord[]>([]);
