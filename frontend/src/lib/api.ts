export interface AuthSnapshot {
  required: boolean;
  user?: {
    name: string;
    role: string;
  } | null;
}

export interface FileRecord {
  schema_version: number;
  id: string;
  name: string;
  size_bytes: number;
  content_hash: string;
  mime_type: string;
  uploaded_at: number;
  tags: string[];
  pinned: boolean;
  folder_path?: string;
  visibility: string;
  public_token?: string | null;
}

export interface NoteRecord {
  schema_version: number;
  id: string;
  title: string;
  body: string;
  tags: string[];
  created_at: number;
  updated_at: number;
  pinned_at?: number | null;
}

export interface WorkspaceRecord {
  id: string;
  owner_id: string;
  name: string;
  language: string;
  body: string;
  created_at: number;
  updated_at: number;
}

export interface WorkspaceDocumentSummary {
  id: string;
  workspace_id: string;
  owner_id: string;
  path: string;
  language: string;
  is_primary: boolean;
  updated_at: number;
  size_bytes: number;
}

export interface WorkspaceDocumentRecord extends WorkspaceDocumentSummary {
  body: string;
  created_at: number;
}

export interface SearchResult {
  type: "file" | "note" | "workspace";
  id?: string;
  title?: string;
  name?: string;
  snippet?: string;
  tags?: string[];
  visibility?: string;
  size_bytes?: number;
  folder_path?: string;
  language?: string;
  updated_at?: number;
  uploaded_at?: number;
}

export interface AdminOverview {
  file_count: number;
  note_count: number;
  workspace_count?: number;
  pinned_count: number;
  tag_count: number;
  storage_bytes_used: number;
  corrupt_file_count: number;
  version?: string;
}

export interface AdminSystem {
  hostname: string;
  os: string;
  arch: string;
  uptime_seconds?: number;
  total_memory_bytes?: number;
  available_memory_bytes?: number;
  data_dir_total_bytes?: number;
  data_dir_free_bytes?: number;
  load_average_1m?: number;
}

export interface AdminActivityItem {
  kind: string;
  id: string;
  title: string;
  detail: string;
  occurred_at: number;
  visibility?: string;
  size_bytes?: number;
  language?: string;
}

export interface AdminUser {
  id: string;
  name: string;
  role: string;
  created_at: number;
  disabled: boolean;
}

export interface AdminSession {
  token: string;
  token_preview: string;
  kind: string;
  user_id?: string | null;
  user_name?: string | null;
  role?: string | null;
  created_at: number;
  expires_at: number;
  current: boolean;
}

export interface AdminDevice {
  device_token: string;
  user_id: string;
  role: string;
  device_name: string;
  last_seen_at: number;
  created_at: number;
  last_ip?: string | null;
  last_user_agent?: string | null;
  expires_at: number;
}

export interface FolderEntry {
  path: string;
  file_count: number;
}

export interface ConsoleCommand {
  name: string;
  description: string;
  category: string;
}

export interface ConsoleOutput {
  schema_version: number;
  command: string;
  success: boolean;
  output: Record<string, unknown>;
  ran_at_ms: number;
}

export interface ShareInfo {
  schema_version: number;
  public_url: string;
  qr_terminal: string;
}

export interface UploadBatchItem {
  filename: string;
  http_status: number;
  deduplicated?: boolean;
  file?: FileRecord;
  error?: {
    code: string;
    message: string;
  };
}

export interface UploadBatchResponse {
  schema_version: number;
  results: UploadBatchItem[];
}

export interface ListFilesParams {
  limit?: number;
  folder?: string;
  name?: string;
  type?: string;
  pinned?: boolean;
  sort?: string;
}

export interface ListNotesParams {
  limit?: number;
  title?: string;
  pinned?: boolean;
  sort?: string;
  tag?: string;
}

export interface CreateWorkspaceInput {
  name: string;
  language?: string;
  body?: string;
}

export interface CreateDocumentInput {
  path: string;
  language?: string;
  body?: string;
  make_primary?: boolean;
}

const API_ROOT = "/api/v1";

async function parseErrorMessage(response: Response): Promise<string> {
  const payload = await response.json().catch(() => null);
  return payload?.error?.message || payload?.error || `${response.status} ${response.statusText}`;
}

export async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_ROOT}${path}`, {
    credentials: "same-origin",
    headers: {
      ...(init?.body instanceof FormData ? {} : { "Content-Type": "application/json" }),
      ...(init?.headers || {}),
    },
    ...init,
  });

  if (!response.ok) {
    throw new Error(await parseErrorMessage(response));
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

function withQuery(path: string, params: Record<string, string | number | boolean | undefined>) {
  const query = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value == null || value === "") continue;
    query.set(key, String(value));
  }
  const suffix = query.toString();
  return suffix ? `${path}?${suffix}` : path;
}

export async function probeAuthStatus(): Promise<AuthSnapshot> {
  const required = await apiFetch<{ required: boolean }>("/auth/required");
  if (!required.required) {
    return { required: false, user: null };
  }
  const user = await apiFetch<{ name: string; role: string }>("/auth/me");
  return { required: true, user };
}

export function listFiles(params: ListFilesParams = {}) {
  return apiFetch<{ files: FileRecord[]; next_cursor?: string }>(
    withQuery("/files", {
      limit: params.limit ?? 200,
      folder: params.folder,
      name: params.name,
      type: params.type,
      pinned: params.pinned,
      sort: params.sort,
    }),
  );
}

export function getFile(id: string) {
  return apiFetch<FileRecord>(`/files/${encodeURIComponent(id)}`);
}

export function getFileShare(id: string) {
  return apiFetch<ShareInfo>(`/files/${encodeURIComponent(id)}/share`);
}

export function setFileVisibility(id: string, visibility: "public" | "private") {
  return apiFetch<{ schema_version: number; file: FileRecord; public_url?: string | null }>(
    `/files/${encodeURIComponent(id)}/visibility`,
    {
      method: "PATCH",
      body: JSON.stringify({ visibility }),
    },
  );
}

export function moveFileToFolder(id: string, folderPath: string) {
  return apiFetch<{ schema_version: number; file: FileRecord }>(
    `/files/${encodeURIComponent(id)}/folder`,
    {
      method: "PATCH",
      body: JSON.stringify({ folder_path: folderPath }),
    },
  );
}

export async function uploadFilesBatch(
  files: File[],
  options: { folderPath?: string; tags?: string[]; pin?: boolean } = {},
) {
  const body = new FormData();
  for (const file of files) {
    body.append("file", file, file.name);
  }
  for (const tag of options.tags || []) {
    body.append("tag", tag);
  }
  if (options.folderPath) {
    body.append("folder_path", options.folderPath);
  }
  if (options.pin) {
    body.append("pin", "true");
  }
  return apiFetch<UploadBatchResponse>("/files/batch", {
    method: "POST",
    body,
  });
}

export function listNotes(params: ListNotesParams = {}) {
  return apiFetch<{ notes: NoteRecord[]; next_cursor?: string }>(
    withQuery("/notes", {
      limit: params.limit ?? 100,
      title: params.title,
      pinned: params.pinned,
      sort: params.sort,
      tag: params.tag,
    }),
  );
}

export function createNote(input: {
  title?: string;
  body: string;
  tags?: string[];
  pin?: boolean;
}) {
  return apiFetch<NoteRecord>("/notes", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function updateNote(id: string, input: { title?: string; body: string }) {
  return apiFetch<NoteRecord>(`/notes/${encodeURIComponent(id)}`, {
    method: "PUT",
    body: JSON.stringify(input),
  });
}

export function deleteNote(id: string) {
  return apiFetch<void>(`/notes/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

export function duplicateNote(id: string) {
  return apiFetch<NoteRecord>(`/notes/${encodeURIComponent(id)}/duplicate`, {
    method: "POST",
  });
}

export function listWorkspaces() {
  return apiFetch<{ workspaces: WorkspaceRecord[] }>("/workspaces");
}

export function createWorkspace(input: CreateWorkspaceInput) {
  return apiFetch<WorkspaceRecord>("/workspaces", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function updateWorkspace(
  id: string,
  input: { name: string; language: string; body: string },
) {
  return apiFetch<WorkspaceRecord>(`/workspaces/${encodeURIComponent(id)}`, {
    method: "PUT",
    body: JSON.stringify(input),
  });
}

export function getAdminWorkspaceDetail(id: string) {
  return apiFetch<{
    schema_version: number;
    workspace: WorkspaceRecord;
    documents: WorkspaceDocumentSummary[];
  }>(`/admin/editor/workspaces/${encodeURIComponent(id)}`);
}

export function getAdminWorkspaceDocument(workspaceId: string, documentId: string) {
  return apiFetch<WorkspaceDocumentRecord>(
    `/admin/editor/workspaces/${encodeURIComponent(workspaceId)}/documents/${encodeURIComponent(documentId)}`,
  );
}

export function createAdminWorkspaceDocument(
  workspaceId: string,
  input: CreateDocumentInput,
) {
  return apiFetch<WorkspaceDocumentRecord>(
    `/admin/editor/workspaces/${encodeURIComponent(workspaceId)}/documents`,
    {
      method: "POST",
      body: JSON.stringify(input),
    },
  );
}

export function updateAdminWorkspaceDocument(
  workspaceId: string,
  documentId: string,
  input: CreateDocumentInput,
) {
  return apiFetch<WorkspaceDocumentRecord>(
    `/admin/editor/workspaces/${encodeURIComponent(workspaceId)}/documents/${encodeURIComponent(documentId)}`,
    {
      method: "PUT",
      body: JSON.stringify(input),
    },
  );
}

export function deleteAdminWorkspaceDocument(workspaceId: string, documentId: string) {
  return apiFetch<void>(
    `/admin/editor/workspaces/${encodeURIComponent(workspaceId)}/documents/${encodeURIComponent(documentId)}`,
    {
      method: "DELETE",
    },
  );
}

export function getEditorExecutionState() {
  return apiFetch<{ execution_disabled: boolean; message: string }>("/admin/editor/check", {
    method: "POST",
  });
}

export function listPublicFiles() {
  return apiFetch<{ files: FileRecord[] }>("/public/files");
}

export function runSearch(
  query: string,
  filters: { kind?: string; type?: string; tag?: string } = {},
) {
  return apiFetch<{ schema_version: number; results: SearchResult[] }>(
    withQuery("/search", {
      q: query,
      limit: 24,
      kind: filters.kind,
      type: filters.type,
      tag: filters.tag,
    }),
  );
}

export function getAdminOverview() {
  return apiFetch<AdminOverview>("/admin/overview");
}

export function getAdminSystem() {
  return apiFetch<AdminSystem>("/admin/system");
}

export function getAdminActivity(limit = 12) {
  return apiFetch<{ schema_version: number; items: AdminActivityItem[] }>(
    `/admin/activity?limit=${encodeURIComponent(String(limit))}`,
  );
}

export function listAdminUsers() {
  return apiFetch<{ schema_version: number; users: AdminUser[] }>("/admin/users");
}

export function listAdminSessions(limit = 50) {
  return apiFetch<{ schema_version: number; sessions: AdminSession[] }>(
    `/admin/sessions?limit=${encodeURIComponent(String(limit))}`,
  );
}

export function listAdminDevices() {
  return apiFetch<{ schema_version: number; devices: AdminDevice[] }>("/admin/devices");
}

export function listAdminFiles(params: { limit?: number; folder?: string; mimePrefix?: string } = {}) {
  return apiFetch<{ schema_version: number; files: FileRecord[]; next_cursor?: string }>(
    withQuery("/admin/files", {
      limit: params.limit ?? 100,
      folder: params.folder,
      mime_prefix: params.mimePrefix,
    }),
  );
}

export function listAdminFolders() {
  return apiFetch<{ schema_version: number; folders: FolderEntry[] }>("/admin/folders");
}

export function listConsoleCommands() {
  return apiFetch<{ schema_version: number; commands: ConsoleCommand[] }>(
    "/admin/console/commands",
  );
}

export function runConsoleCommand(command: string) {
  return apiFetch<ConsoleOutput>("/admin/console/run", {
    method: "POST",
    body: JSON.stringify({ command }),
  });
}
