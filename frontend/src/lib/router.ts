import { writable } from "svelte/store";

export type AppId = "drive" | "knowledge" | "workspace" | "operations";

export type DriveLens = "all" | "images" | "videos" | "documents";
export type OpsSection =
  | "overview"
  | "access"
  | "files"
  | "storage"
  | "maintenance"
  | "console";

export interface NavItem {
  id: AppId;
  label: string;
  subtitle: string;
}

export const apps: NavItem[] = [
  { id: "drive", label: "Cloud Drive", subtitle: "Files, folders, sharing" },
  { id: "knowledge", label: "Knowledge", subtitle: "Notes and capture" },
  { id: "workspace", label: "Workspace", subtitle: "Projects and editor" },
  { id: "operations", label: "Operations", subtitle: "Admin and diagnostics" },
];

function parseHash(): { app: AppId; driveLens: DriveLens; ops: OpsSection } {
  if (typeof window === "undefined") {
    return { app: "drive", driveLens: "all", ops: "overview" };
  }
  const raw = window.location.hash.replace(/^#/, "").trim().toLowerCase();
  const [appPart, subPart] = raw.split("/");
  const alias: Record<string, AppId> = {
    drive: "drive",
    objects: "drive",
    images: "drive",
    videos: "drive",
    documents: "drive",
    notes: "knowledge",
    knowledge: "knowledge",
    workspace: "workspace",
    workspaces: "workspace",
    editor: "workspace",
    operations: "operations",
    admin: "operations",
    ops: "operations",
  };
  const lensAlias: Record<string, DriveLens> = {
    images: "images",
    videos: "videos",
    documents: "documents",
  };
  const app = alias[appPart] || "drive";
  let driveLens: DriveLens = "all";
  if (appPart in lensAlias) driveLens = lensAlias[appPart];
  else if (subPart && subPart in lensAlias) driveLens = lensAlias[subPart];
  const opsSections = new Set<OpsSection>([
    "overview",
    "access",
    "files",
    "storage",
    "maintenance",
    "console",
  ]);
  const ops = opsSections.has(subPart as OpsSection) ? (subPart as OpsSection) : "overview";
  return { app, driveLens, ops };
}

export const appRoute = writable<AppId>("drive");
export const driveLensRoute = writable<DriveLens>("all");
export const opsSection = writable<OpsSection>("overview");

function syncFromHash() {
  const parsed = parseHash();
  appRoute.set(parsed.app);
  driveLensRoute.set(parsed.driveLens);
  opsSection.set(parsed.ops);
}

if (typeof window !== "undefined") {
  syncFromHash();
  window.addEventListener("hashchange", syncFromHash);
}

export function navigateApp(app: AppId, sub?: string) {
  if (typeof window === "undefined") return;
  const hash = sub ? `${app}/${sub}` : app;
  window.location.hash = hash;
  syncFromHash();
}

export function navigateDriveLens(lens: DriveLens) {
  navigateApp("drive", lens === "all" ? undefined : lens);
  driveLensRoute.set(lens);
}

export function navigateOps(section: OpsSection) {
  navigateApp("operations", section);
  opsSection.set(section);
}
