import type { FileRecord } from "../api";

export type DriveLens = "drive" | "images" | "videos" | "documents" | "all";

export interface FolderEntry {
  path: string;
  label: string;
  depth: number;
  count: number;
}

export function matchesDriveLens(file: FileRecord, lens: DriveLens): boolean {
  const mime = file.mime_type || "";
  if (lens === "drive" || lens === "all") return true;
  if (lens === "images") return mime.startsWith("image/");
  if (lens === "videos") return mime.startsWith("video/");
  if (lens === "documents") {
    return (
      mime.startsWith("text/") ||
      mime === "application/pdf" ||
      mime.includes("json") ||
      mime.includes("xml") ||
      mime.includes("word") ||
      mime.includes("sheet")
    );
  }
  return true;
}

export function fileKindLabel(file: FileRecord): string {
  const mime = file.mime_type || "";
  if (mime.startsWith("image/")) return "Image";
  if (mime.startsWith("video/")) return "Video";
  if (mime.startsWith("audio/")) return "Audio";
  if (mime === "application/pdf") return "PDF";
  if (mime.startsWith("text/")) return "Text";
  return "Object";
}

export function fileIconLabel(file: FileRecord): string {
  const mime = file.mime_type || "";
  if (mime.startsWith("image/")) return "IMG";
  if (mime.startsWith("video/")) return "VID";
  if (mime.startsWith("audio/")) return "AUD";
  if (mime === "application/pdf") return "PDF";
  if (mime.startsWith("text/")) return "TXT";
  return "OBJ";
}

export function buildFolderEntries(files: FileRecord[]): FolderEntry[] {
  const counts = new Map<string, number>();
  counts.set("", files.length);
  for (const file of files) {
    const folder = file.folder_path || "";
    if (!folder) continue;
    const segments = folder.split("/").filter(Boolean);
    for (let index = 0; index < segments.length; index += 1) {
      const path = segments.slice(0, index + 1).join("/");
      counts.set(path, (counts.get(path) || 0) + 1);
    }
  }

  return Array.from(counts.entries())
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([path, count]) => {
      const segments = path.split("/").filter(Boolean);
      return {
        path,
        label: segments.at(-1) || "Bucket root",
        depth: segments.length,
        count,
      };
    });
}

export function inferFilePreviewUrl(file: FileRecord): string {
  return `/api/v1/files/${encodeURIComponent(file.id)}/content?disposition=inline`;
}
