import { writable } from 'svelte/store';

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

const CHUNK_SIZE = 262_144; // 256 KB
const MAX_CONCURRENT_UPLOADS = 3;
const MAX_RETRIES = 5;
const RETRY_DELAY_MS = 500;
const DB_NAME = 'tssp_uploads';
const DB_VERSION = 1;
const STORE_NAME = 'upload_queue';

let db: IDBDatabase | null = null;

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

    request.onerror = () => {
      reject(new Error('Failed to open IndexedDB'));
    };
  });
}

async function getQueueFromDb(): Promise<UploadQueueItem[]> {
  const database = await initDb();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, 'readonly');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.getAll();

    request.onsuccess = () => {
      const items = request.result || [];
      items.forEach((item: any) => {
        item.uploadedChunks = new Set(item.uploadedChunks || []);
      });
      resolve(items);
    };

    request.onerror = () => {
      reject(new Error('Failed to read queue from IndexedDB'));
    };
  });
}

async function saveItemToDb(item: UploadQueueItem): Promise<void> {
  const database = await initDb();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const itemToStore = {
      ...item,
      uploadedChunks: Array.from(item.uploadedChunks),
    };
    const request = store.put(itemToStore);

    request.onsuccess = () => {
      resolve();
    };

    request.onerror = () => {
      reject(new Error('Failed to save upload item to IndexedDB'));
    };
  });
}

async function deleteItemFromDb(id: string): Promise<void> {
  const database = await initDb();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.delete(id);

    request.onsuccess = () => {
      resolve();
    };

    request.onerror = () => {
      reject(new Error('Failed to delete upload item from IndexedDB'));
    };
  });
}

function calculateTotalChunks(fileSize: number): number {
  return Math.ceil(fileSize / CHUNK_SIZE);
}

function createUploadQueueItem(file: File, folder: string): UploadQueueItem {
  return {
    id: `${file.name}_${Date.now()}`,
    filename: file.name,
    fileSize: file.size,
    folder,
    uploadedBytes: 0,
    totalChunks: calculateTotalChunks(file.size),
    uploadedChunks: new Set(),
    status: 'pending',
    retries: 0,
    createdAt: Date.now(),
  };
}

function createStoreInstance() {
  const { subscribe, set, update } = writable<UploadQueueStore>({
    items: [],
    totalUploadingCount: 0,
  });

  // Load persisted queue on startup
  getQueueFromDb()
    .then((items) => {
      set({
        items,
        totalUploadingCount: items.filter((i) => i.status === 'uploading').length,
      });
    })
    .catch((err) => {
      console.error('Failed to load upload queue from IndexedDB:', err);
    });

  return {
    subscribe,
    addFiles: async (files: FileList, folder: string = '') => {
      const newItems: UploadQueueItem[] = [];

      // Create items and register files
      for (const file of files) {
        const item = createUploadQueueItem(file, folder);
        registerFileForUpload(item.id, file);
        newItems.push(item);

        try {
          await saveItemToDb(item);
        } catch (err) {
          console.error('Failed to persist upload item:', err);
        }
      }

      update((store) => ({
        ...store,
        items: [...store.items, ...newItems],
      }));

      // Start processing queue
      processQueue();
    },

    setStatus: async (
      id: string,
      status: UploadQueueItem['status'],
      error?: string
    ) => {
      update((store) => {
        const item = store.items.find((i) => i.id === id);
        if (item) {
          item.status = status;
          if (error) item.lastError = error;
          if (status === 'uploading' && !item.startedAt) {
            item.startedAt = Date.now();
          }
          if (status === 'completed' && !item.completedAt) {
            item.completedAt = Date.now();
          }
          saveItemToDb(item).catch((err) =>
            console.error('Failed to persist status change:', err)
          );
        }
        return store;
      });
    },

    updateProgress: async (id: string, uploadedBytes: number, chunkIndex: number) => {
      update((store) => {
        const item = store.items.find((i) => i.id === id);
        if (item) {
          item.uploadedBytes = uploadedBytes;
          item.uploadedChunks.add(chunkIndex);
          saveItemToDb(item).catch((err) =>
            console.error('Failed to persist progress:', err)
          );
        }
        return store;
      });
    },

    removeItem: async (id: string) => {
      try {
        await deleteItemFromDb(id);
      } catch (err) {
        console.error('Failed to delete item from IndexedDB:', err);
      }

      unregisterFileForUpload(id);

      update((store) => ({
        ...store,
        items: store.items.filter((i) => i.id !== id),
      }));
    },

    clear: async () => {
      const database = await initDb();
      return new Promise((resolve) => {
        const transaction = database.transaction(STORE_NAME, 'readwrite');
        const store = transaction.objectStore(STORE_NAME);
        const request = store.clear();

        request.onsuccess = () => {
          set({ items: [], totalUploadingCount: 0 });
          resolve(undefined);
        };

        request.onerror = () => {
          console.error('Failed to clear upload queue');
          resolve(undefined);
        };
      });
    },

    retryItem: async (id: string) => {
      update((store) => {
        const item = store.items.find((i) => i.id === id);
        if (item && item.retries < MAX_RETRIES) {
          item.status = 'pending';
          item.retries++;
          item.lastError = undefined;
          saveItemToDb(item).catch((err) =>
            console.error('Failed to persist retry:', err)
          );
        }
        return store;
      });
      processQueue();
    },
  };
}

export const uploadQueue = createStoreInstance();

let fileMap = new Map<string, File>();

function registerFileForUpload(uploadId: string, file: File) {
  fileMap.set(uploadId, file);
}

function getFileForUpload(uploadId: string): File | undefined {
  return fileMap.get(uploadId);
}

function unregisterFileForUpload(uploadId: string) {
  fileMap.delete(uploadId);
}

async function processQueue() {
  // Process uploads in background: up to MAX_CONCURRENT_UPLOADS at a time
  // This will be called after files are added to the queue
  const { startChunkedUpload, uploadChunk, completeUpload } = await import('../services/chunkedUploadService');

  const processNextBatch = async () => {
    let queueState: UploadQueueStore | null = null;
    const unsubscribe = subscribe((state) => {
      queueState = state;
    });

    while (queueState && queueState.items.some((i) => i.status === 'pending')) {
      const pending = queueState.items.filter((i) => i.status === 'pending');
      const uploading = queueState.items.filter((i) => i.status === 'uploading');

      if (uploading.length >= MAX_CONCURRENT_UPLOADS || pending.length === 0) {
        break;
      }

      const uploadItem = pending[0];
      const file = getFileForUpload(uploadItem.id);

      if (!file) {
        // File not available (page refresh), mark as failed
        const unsubscribe2 = subscribe((state) => {
          const item = state.items.find((i) => i.id === uploadItem.id);
          if (item && item.status === 'pending') {
            item.status = 'failed';
            item.lastError = 'File not available (recovered upload)';
          }
        });
        unsubscribe2();
        continue;
      }

      // Start the upload
      const subscription = subscribe((state) => {
        queueState = state;
      });

      const sessionId = await startChunkedUpload(uploadItem.id, file, uploadItem.folder);
      if (!sessionId) continue;

      // Upload all chunks
      let success = true;
      for (let i = 0; i < uploadItem.totalChunks; i++) {
        const start = i * CHUNK_SIZE;
        const end = Math.min(start + CHUNK_SIZE, file.size);
        const chunk = file.slice(start, end);

        const uploaded = await uploadChunk(uploadItem.id, sessionId, i, chunk);
        if (!uploaded) {
          success = false;
          break;
        }
      }

      if (success) {
        await completeUpload(uploadItem.id, sessionId);
      }

      subscription();
    }

    unsubscribe();

    // Continue processing if there are more pending items
    if (queueState && queueState.items.some((i) => i.status === 'pending')) {
      setTimeout(processNextBatch, 100);
    }
  };

  processNextBatch().catch((err) => {
    console.error('Upload queue processing error:', err);
  });
}
