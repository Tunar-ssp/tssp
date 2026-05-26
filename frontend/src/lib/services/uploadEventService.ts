/**
 * Upload event handling service
 * Manages file upload triggers and event routing
 */

import { uploadFiles } from './fileService';

/**
 * Handle upload file selection
 */
export function handleUploadFiles(input: HTMLInputElement | null): void {
  input?.click();
}

/**
 * Register upload event listeners
 */
export function registerUploadEventHandlers(uploadInput: HTMLInputElement | null): () => void {
  const handleUploadRequest = () => {
    handleUploadFiles(uploadInput);
  };

  document.addEventListener('tssp:request-upload', handleUploadRequest as EventListener);

  return () => {
    document.removeEventListener('tssp:request-upload', handleUploadRequest as EventListener);
  };
}

/**
 * Handle file input change - parse selected files and start upload
 */
export async function handleFileInputChange(
  event: Event,
  targetFolder: string = '/'
): Promise<void> {
  const input = event.target as HTMLInputElement;
  const files = input.files;

  if (!files || files.length === 0) return;

  try {
    await uploadFiles(Array.from(files), targetFolder);
  } catch (err) {
    console.error('Upload failed:', err);
  } finally {
    // Reset input to allow re-selecting same file
    input.value = '';
  }
}
