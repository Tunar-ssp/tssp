import { api } from '../api';
import { uploadQueue } from '../stores/uploadQueue';

const CHUNK_SIZE = 262_144; // 256 KB
const MAX_RETRIES = 5;
const BASE_RETRY_DELAY_MS = 500;
const MAX_CONCURRENT = 3;

interface FileForUpload {
  id: string;
  file: File;
  folder: string;
}

let uploadingCount = 0;
let uploadQueue_: FileForUpload[] = [];

async function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function getRetryDelay(retryCount: number): number {
  // Exponential backoff: 500ms, 1s, 2s, 4s, 8s
  return BASE_RETRY_DELAY_MS * Math.pow(2, Math.min(retryCount, 4));
}

export async function startChunkedUpload(
  uploadId: string,
  file: File,
  folder: string
): Promise<string | null> {
  try {
    const data = await api.startUpload(file.name, file.size, folder);
    return data.session_id;
  } catch (err) {
    console.error('Error starting upload:', err);
    await uploadQueue.setStatus(uploadId, 'failed', String(err));
    return null;
  }
}

async function uploadChunk(
  uploadId: string,
  sessionId: string,
  chunkIndex: number,
  chunk: Blob,
  retryCount: number = 0
): Promise<boolean> {
  try {
    await api.uploadChunk(sessionId, chunkIndex, chunk);
    const uploadedBytes = (chunkIndex + 1) * CHUNK_SIZE;
    await uploadQueue.updateProgress(uploadId, uploadedBytes, chunkIndex);

    return true;
  } catch (err) {
    if (retryCount < MAX_RETRIES) {
      await delay(getRetryDelay(retryCount));
      return uploadChunk(uploadId, sessionId, chunkIndex, chunk, retryCount + 1);
    }
    console.error(`Failed to upload chunk ${chunkIndex}:`, err);
    await uploadQueue.setStatus(uploadId, 'failed', `Chunk ${chunkIndex} upload failed`);
    return false;
  }
}

async function completeUpload(
  uploadId: string,
  sessionId: string,
): Promise<boolean> {
  try {
    await api.completeUpload(sessionId);

    await uploadQueue.setStatus(uploadId, 'completed');
    return true;
  } catch (err) {
    console.error('Error completing upload:', err);
    await uploadQueue.setStatus(uploadId, 'failed', String(err));
    return false;
  }
}

async function cancelUpload(sessionId: string): Promise<void> {
  try {
    await api.cancelUpload(sessionId);
  } catch (err) {
    console.error('Error canceling upload:', err);
  }
}

async function uploadFile(
  uploadId: string,
  file: File,
  folder: string
): Promise<boolean> {
  try {
    // Set status to uploading
    await uploadQueue.setStatus(uploadId, 'uploading');

    // Start upload session
    const sessionId = await startChunkedUpload(uploadId, file, folder);
    if (!sessionId) {
      return false;
    }

    // Split file into chunks
    const chunks: Blob[] = [];
    let offset = 0;
    while (offset < file.size) {
      const chunkSize = Math.min(CHUNK_SIZE, file.size - offset);
      chunks.push(file.slice(offset, offset + chunkSize));
      offset += chunkSize;
    }

    // Upload all chunks
    for (let i = 0; i < chunks.length; i++) {
      const success = await uploadChunk(uploadId, sessionId, i, chunks[i]);
      if (!success) {
        await cancelUpload(sessionId);
        return false;
      }
    }

    // Complete the upload
    const completed = await completeUpload(uploadId, sessionId);
    return completed;
  } catch (err) {
    console.error('Error uploading file:', err);
    await uploadQueue.setStatus(uploadId, 'failed', String(err));
    return false;
  }
}

export async function processUploadQueue(): Promise<void> {
  // Load current queue state
  let queueState: any = null;
  uploadQueue.subscribe((state) => {
    queueState = state;
  })();

  while (true) {
    // Get pending and uploading items
    const pending = queueState.items.filter((i) => i.status === 'pending');
    const uploading = queueState.items.filter((i) => i.status === 'uploading');

    // Don't exceed max concurrent uploads
    if (uploading.length >= MAX_CONCURRENT || pending.length === 0) {
      // All done or at capacity
      break;
    }

    // Get next file to upload
    const nextItem = pending[0];

    // TODO: Get actual file from browser's File API
    // For now, this is a placeholder that will be integrated with the UI layer
    // Real implementation needs to track File objects separately from upload metadata

    await delay(100);
  }
}

export async function retryFailedUploads(): Promise<void> {
  let queueState: any = null;
  uploadQueue.subscribe((state) => {
    queueState = state;
  })();

  const failed = queueState.items.filter((i) => i.status === 'failed');
  for (const item of failed) {
    if (item.retries < MAX_RETRIES) {
      await uploadQueue.retryItem(item.id);
    }
  }
}
