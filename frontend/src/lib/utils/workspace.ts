import type { WorkspaceDocumentSummary } from "../api";

export interface WorkspaceTreeNode {
  key: string;
  name: string;
  path: string;
  type: "folder" | "file";
  children?: WorkspaceTreeNode[];
  documentId?: string;
}

export function inferLanguageFromPath(path: string): string {
  const extension = path.split(".").pop()?.toLowerCase() || "";
  switch (extension) {
    case "rs":
      return "rust";
    case "py":
      return "python";
    case "js":
      return "javascript";
    case "ts":
      return "typescript";
    case "md":
      return "markdown";
    case "json":
      return "json";
    case "yaml":
    case "yml":
      return "yaml";
    case "toml":
      return "toml";
    case "html":
      return "html";
    case "css":
      return "css";
    case "sql":
      return "sql";
    case "sh":
      return "bash";
    default:
      return "text";
  }
}

export function buildWorkspaceTree(
  documents: WorkspaceDocumentSummary[],
): WorkspaceTreeNode[] {
  const root: WorkspaceTreeNode[] = [];

  for (const document of documents) {
    const segments = document.path.split("/").filter(Boolean);
    let cursor = root;
    let currentPath = "";

    segments.forEach((segment, index) => {
      currentPath = currentPath ? `${currentPath}/${segment}` : segment;
      const isFile = index === segments.length - 1;
      let existing = cursor.find((entry) => entry.name === segment && entry.type === (isFile ? "file" : "folder"));
      if (!existing) {
        existing = {
          key: currentPath,
          name: segment,
          path: currentPath,
          type: isFile ? "file" : "folder",
          children: isFile ? undefined : [],
          documentId: isFile ? document.id : undefined,
        };
        cursor.push(existing);
        cursor.sort((left, right) => {
          if (left.type !== right.type) return left.type === "folder" ? -1 : 1;
          return left.name.localeCompare(right.name);
        });
      }
      if (!isFile) {
        existing.children ||= [];
        cursor = existing.children;
      }
    });
  }

  return root;
}
