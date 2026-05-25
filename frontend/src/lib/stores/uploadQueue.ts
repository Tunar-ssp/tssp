import { writable } from 'svelte/store';
import { api } from '../api';

export interface UploadQueueItem {
  id: string;
  filename: string;
  fileSize: number;
  folder: string;
  sessionId?: string;
  uploadedBytes: number;
  totalChunks: number;
  uploadedChunks: Set<number>;
  status: 'pending' | 'uploading' | 'paused' | 'completed' | 'failed';
  retries: number;
  lastError?: string;
  createdAt: number;
  startedAt?: number;
  completedAt?: number;
}

interface UploadQueueStore {
  items: UploadQueueItem[];
  totalUploadingCount: number;
}

const FALLBACK_CHUNK_SIZE = 262_144;
const MAX_CONCURRENT_UPLOADS = 3;
const MAX_RETRIES = 5;
const DB_NAME = 'tssp_uploads';
const DB_VERSION = 1;
const STORE_NAME = 'upload_queue';

let db: IDBDatabase | null = null;
let stateCache: UploadQueueStore = { items: [], totalUploadingCount: 0 };
let isProcessing = false;
const fileMap = new Map<string, File>();

async function initDb(): Promise<IDBDatabase> {
  if (db) return db;

  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onupgradeneeded = () => {
      const database = request.result;
      if (!database.objectStoreNames.contains(STORE_NAME)) {
        database.createObjectStore(STORE_NAME, { keyPath: 'id' });
      }
    };

    request.onsuccess = () => {
      db = request.result;
      resolve(db);
    };

    request.onerror = () => reject(new Error('Failed to open IndexedDB'));
  });
}

async function getQueueFromDb(): Promise<UploadQueueItem[]> {
  const database = await initDb();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, 'readonly');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.getAll();

    request.onsuccess = () => {
      const items = (request.result || []).map((item: any) => ({
        ...item,
        uploadedChunks: new Set(item.uploadedChunks || []),
      }));
      resolve(items);
    };

    request.onerror = () => reject(new Error('Failed to read queue from IndexedDB'));
  });
}

async function saveItemToDb(item: UploadQueueItem): Promise<void> {
  const database = await initDb();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.put({
      ...item,
      uploadedChunks: Array.from(item.uploadedChunks),
    });

    request.onsuccess = () => resolve();
    request.onerror = () => reject(new Error('Failed to save upload item'));
  });
}

async function deleteItemFromDb(id: string): Promise<void> {
  const database = await initDb();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.delete(id);

    request.onsuccess = () => resolve();
    request.onerror = () => reject(new Error('Failed to delete upload item'));
  });
}

function createUploadQueueItem(file: File, folder: string): UploadQueueItem {
  const now = Date.now();
  return {
    id: `${file.name}-${file.size}-${now}`,
    filename: file.name,
    fileSize: file.size,
    folder,
    uploadedBytes: 0,
    totalChunks: Math.ceil(file.size / FALLBACK_CHUNK_SIZE),
    uploadedChunks: new Set(),
    status: 'pending',
    retries: 0,
    createdAt: now,
  };
}

function refreshTotals(items: UploadQueueItem[]): UploadQueueStore {
  return {
    items,
    totalUploadingCount: items.filter((item) => item.status === 'uploading').length,
  };
}

function updateItemLocally(
  id: string,
  updater: (item: UploadQueueItem) => UploadQueueItem | void,
) {
  uploadQueueState.update((current) => {
    const items = current.items.map((item) => {
      if (item.id !== id) return item;
      const next = updater(item);
      const merged = next ? next : item;
      void saveItemToDb(merged).catch((err) => {
        console.error('Failed to persist upload item:', err);
      });
      return merged;
    });
    return refreshTotals(items);
  });
}

const uploadQueueState = writable<UploadQueueStore>({ items: [], totalUploadingCount: 0 });
uploadQueueState.subscribe((value) => {
  stateCache = value;
});

async function uploadSingleItem(itemId: string) {
  const item = stateCache.items.find((entry) => entry.id === itemId);
  const file = fileMap.get(itemId);

  if (!item) return;
  if (!file) {
    updateItemLocally(itemId, (current) => ({
      ...current,
      status: 'failed',
      lastError: 'File data is unavailable after reload. Re-add the file to continue.',
    }));
    return;
  }

  try {
    updateItemLocally(itemId, (current) => ({
      ...current,
      status: 'uploading',
      startedAt: current.startedAt ?? Date.now(),
      lastError: undefined,
    }));

    const { session_id, chunk_size } = await api.startUpload(item.filename, item.fileSize, item.folder);
    const effectiveChunkSize = chunk_size || FALLBACK_CHUNK_SIZE;
    const totalChunks = Math.ceil(file.size / effectiveChunkSize);

    updateItemLocally(itemId, (current) => ({
      ...current,
      sessionId: session_id,
      totalChunks,
      uploadedBytes: 0,
      uploadedChunks: new Set(),
    }));

    for (let index = 0; index < totalChunks; index += 1) {
      const start = index * effectiveChunkSize;
      const end = Math.min(start + effectiveChunkSize, file.size);
      const chunk = file.slice(start, end);
      await api.uploadChunk(session_id, index, chunk);

      updateItemLocally(itemId, (current) => {
        const uploadedChunks = new Set(current.uploadedChunks);
        uploadedChunks.add(index);
        return {
          ...current,
          uploadedChunks,
          uploadedBytes: Math.min(end, file.size),
        };
      });
    }

    await api.completeUpload(session_id, [
      {
        name: file.name,
        mime_type: file.type || 'application/octet-stream',
      },
    ]);

    updateItemLocally(itemId, (current) => ({
      ...current,
      status: 'completed',
      completedAt: Date.now(),
      uploadedBytes: current.fileSize,
    }));
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Upload failed';
    updateItemLocally(itemId, (current) => ({
      ...current,
      status: 'failed',
      lastError: message,
    }));
  }
}

async function processQueue() {
  if (isProcessing) return;
  isProcessing = true;

  try {
    while (true) {
      const uploading = stateCache.items.filter((item) => item.status === 'uploading').length;
      const availableSlots = Math.max(MAX_CONCURRENT_UPLOADS - uploading, 0);
      const nextItems = stateCache.items
        .filter((item) => item.status === 'pending')
        .slice(0, availableSlots);

      if (!nextItems.length) break;
      await Promise.all(nextItems.map((item) => uploadSingleItem(item.id)));
    }
  } finally {
    isProcessing = false;
    if (stateCache.items.some((item) => item.status === 'pending')) {
      queueMicrotask(() => {
        void processQueue();
      });
    }
  }
}

getQueueFromDb()
  .then((items) => {
    const recovered = items.map((item) =>
      item.status === 'uploading' || item.status === 'pending'
        ? {
            ...item,
            status: 'failed' as const,
            lastError: 'Recovered pending upload. Re-add the file to continue.',
          }
        : item,
    );
    uploadQueueState.set(refreshTotals(recovered));
  })
  .catch((err) => {
    console.error('Failed to load upload queue from IndexedDB:', err);
  });

export const uploadQueue = {
  subscribe: uploadQueueState.subscribe,

  async addFiles(files: FileList, folder = '') {
    const newItems: UploadQueueItem[] = [];

    for (const file of Array.from(files)) {
      const item = createUploadQueueItem(file, folder);
      fileMap.set(item.id, file);
      newItems.push(item);
      await saveItemToDb(item).catch((err) => {
        console.error('Failed to persist upload item:', err);
      });
    }

    uploadQueueState.update((current) => refreshTotals([...current.items, ...newItems]));
    void processQueue();
  },

  async setStatus(id: string, status: UploadQueueItem['status'], lastError?: string) {
    updateItemLocally(id, (item) => ({
      ...item,
      status,
      lastError,
      startedAt: status === 'uploading' ? item.startedAt ?? Date.now() : item.startedAt,
      completedAt: status === 'completed' ? item.completedAt ?? Date.now() : item.completedAt,
    }));
  },

  async updateProgress(id: string, uploadedBytes: number, chunkIndex: number) {
    updateItemLocally(id, (item) => {
      const uploadedChunks = new Set(item.uploadedChunks);
      uploadedChunks.add(chunkIndex);
      return {
        ...item,
        uploadedBytes,
        uploadedChunks,
      };
    });
  },

  async removeItem(id: string) {
    fileMap.delete(id);
    await deleteItemFromDb(id).catch((err) => {
      console.error('Failed to delete upload item:', err);
    });
    uploadQueueState.update((current) => refreshTotals(current.items.filter((item) => item.id !== id)));
  },

  async clear() {
    const items = [...stateCache.items];
    await Promise.all(items.map((item) => deleteItemFromDb(item.id).catch(() => undefined)));
    items.forEach((item) => fileMap.delete(item.id));
    uploadQueueState.set(refreshTotals([]));
  },

  async retryItem(id: string) {
    updateItemLocally(id, (item) => ({
      ...item,
      status: 'pending',
      retries: Math.min(item.retries + 1, MAX_RETRIES),
      lastError: undefined,
      uploadedBytes: 0,
      uploadedChunks: new Set(),
    }));
    void processQueue();
  },
};
