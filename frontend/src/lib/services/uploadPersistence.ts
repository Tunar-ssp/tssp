/**
 * Upload Persistence Service
 *
 * Handles IndexedDB operations for upload queue persistence.
 * Manages session recovery and chunk tracking.
 */

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

const DB_NAME = 'tssp_uploads';
const DB_VERSION = 1;
const STORE_NAME = 'upload_queue';

let db: IDBDatabase | null = null;

export async function initDatabase(): Promise<IDBDatabase> {
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

export async function loadQueue(): Promise<UploadQueueItem[]> {
  const database = await initDatabase();
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

export async function saveItem(item: UploadQueueItem): Promise<void> {
  const database = await initDatabase();
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

export async function deleteItem(id: string): Promise<void> {
  const database = await initDatabase();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.delete(id);

    request.onsuccess = () => resolve();
    request.onerror = () => reject(new Error('Failed to delete upload item'));
  });
}

export async function clearQueue(): Promise<void> {
  const database = await initDatabase();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.clear();

    request.onsuccess = () => resolve();
    request.onerror = () => reject(new Error('Failed to clear upload queue'));
  });
}

export function markChunkComplete(item: UploadQueueItem, chunkIndex: number): UploadQueueItem {
  return {
    ...item,
    uploadedChunks: new Set([...item.uploadedChunks, chunkIndex]),
    uploadedBytes: item.uploadedBytes + Math.min(262_144, item.fileSize - item.uploadedBytes),
  };
}

export function getLastSuccessfulChunk(item: UploadQueueItem): number {
  const chunks = Array.from(item.uploadedChunks).sort((a, b) => a - b);
  return chunks.length > 0 ? Math.max(...chunks) : -1;
}
