import { writable } from "svelte/store";

export type RouteId =
  | "drive"
  | "images"
  | "videos"
  | "documents"
  | "sharing"
  | "notes"
  | "workspace"
  | "operations"
  | "search";

export interface NavItem {
  id: RouteId;
  label: string;
  group: string;
  subtitle: string;
}

export const navItems: NavItem[] = [
  {
    id: "drive",
    label: "Cloud Drive",
    group: "Storage",
    subtitle: "Files, folders, uploads, previews",
  },
  {
    id: "sharing",
    label: "Sharing Center",
    group: "Storage",
    subtitle: "Public links and QR access",
  },
  {
    id: "notes",
    label: "Notes",
    group: "Knowledge",
    subtitle: "Pages, capture, and structured writing",
  },
  {
    id: "workspace",
    label: "Workspace",
    group: "Build",
    subtitle: "Projects, documents, and editor tabs",
  },
  {
    id: "search",
    label: "Search",
    group: "Global",
    subtitle: "Cross-product lookup and command surface",
  },
  {
    id: "operations",
    label: "Operations",
    group: "Admin",
    subtitle: "Users, storage, diagnostics, and safe console",
  },
];

const HASH_ALIASES: Record<string, RouteId> = {
  drive: "drive",
  objects: "drive",
  images: "images",
  videos: "videos",
  documents: "documents",
  public: "sharing",
  sharing: "sharing",
  notes: "notes",
  workspaces: "workspace",
  editor: "workspace",
  search: "search",
  admin: "operations",
  overview: "operations",
  operations: "operations",
};

function normalizeRoute(hash: string): RouteId {
  const key = hash.replace(/^#/, "").trim().toLowerCase();
  return HASH_ALIASES[key] || "drive";
}

function installHashSync() {
  if (typeof window === "undefined") return;
  window.addEventListener("hashchange", () => {
    route.set(normalizeRoute(window.location.hash));
  });
}

export const route = writable<RouteId>(
  typeof window === "undefined" ? "drive" : normalizeRoute(window.location.hash),
);

installHashSync();

export function navigate(next: RouteId) {
  if (typeof window !== "undefined") {
    window.location.hash = next;
  }
  route.set(next);
}
