/**
 * Upload Chunk Service
 *
 * Handles file chunking, chunk hashing, and retry logic for uploads.
 */

const CHUNK_SIZE = 262_144; // 256KB
const MAX_RETRIES = 5;
const INITIAL_RETRY_DELAY = 1000; // 1 second

export function splitFileIntoChunks(file: File): Blob[] {
  const chunks: Blob[] = [];
  for (let i = 0; i < file.size; i += CHUNK_SIZE) {
    chunks.push(file.slice(i, i + CHUNK_SIZE));
  }
  return chunks;
}

export function calculateChunkCount(fileSize: number): number {
  return Math.ceil(fileSize / CHUNK_SIZE);
}

export function getChunkSize(index: number, totalSize: number): number {
  const start = index * CHUNK_SIZE;
  const end = Math.min(start + CHUNK_SIZE, totalSize);
  return end - start;
}

export async function calculateChunkHash(chunk: Blob): Promise<string> {
  const buffer = await chunk.arrayBuffer();
  const hashBuffer = await crypto.subtle.digest('SHA-256', buffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
}

export function getRetryDelay(retryCount: number): number {
  // Exponential backoff: 1s, 2s, 4s, 8s, 16s
  return INITIAL_RETRY_DELAY * Math.pow(2, Math.min(retryCount, 4));
}

export function canRetry(retryCount: number): boolean {
  return retryCount < MAX_RETRIES;
}

export function getRetryError(retryCount: number): string {
  if (retryCount >= MAX_RETRIES) {
    return `Upload failed after ${MAX_RETRIES} retries`;
  }
  return `Retry attempt ${retryCount + 1}/${MAX_RETRIES}`;
}
