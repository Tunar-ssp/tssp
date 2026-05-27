/**
 * Image Thumbnail Service
 * Generates image thumbnails client-side using Canvas API
 * Caches results as blob URLs for efficient reuse
 */

const THUMBNAIL_SIZES = {
  small: 80,
  medium: 160,
  large: 300,
};

type ThumbnailSize = keyof typeof THUMBNAIL_SIZES;

interface CachedThumbnail {
  url: string;
  timestamp: number;
}

const thumbnailCache = new Map<string, Map<ThumbnailSize, CachedThumbnail>>();
const CACHE_TTL = 1 * 60 * 60 * 1000; // 1 hour

function getCacheKey(fileId: string): Map<ThumbnailSize, CachedThumbnail> {
  if (!thumbnailCache.has(fileId)) {
    thumbnailCache.set(fileId, new Map());
  }
  return thumbnailCache.get(fileId)!;
}

function cleanupExpiredCache(fileId: string) {
  const cache = getCacheKey(fileId);
  const now = Date.now();

  for (const [size, cached] of cache.entries()) {
    if (now - cached.timestamp > CACHE_TTL) {
      URL.revokeObjectURL(cached.url);
      cache.delete(size);
    }
  }
}

export async function generateImageThumbnail(
  fileId: string,
  imageUrl: string,
  size: ThumbnailSize = 'medium'
): Promise<string> {
  const cache = getCacheKey(fileId);
  cleanupExpiredCache(fileId);

  // Return cached version if available
  const cached = cache.get(size);
  if (cached) {
    return cached.url;
  }

  try {
    const thumbnailSize = THUMBNAIL_SIZES[size];
    const img = new Image();

    img.crossOrigin = 'same-origin';
    img.src = imageUrl;

    return new Promise((resolve, reject) => {
      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');

          if (!ctx) {
            reject(new Error('Could not create canvas context'));
            return;
          }

          // Calculate dimensions maintaining aspect ratio
          let width = img.width;
          let height = img.height;
          const ratio = width / height;

          if (width > height) {
            width = thumbnailSize;
            height = Math.round(width / ratio);
          } else {
            height = thumbnailSize;
            width = Math.round(height * ratio);
          }

          canvas.width = width;
          canvas.height = height;

          ctx.drawImage(img, 0, 0, width, height);

          canvas.toBlob(
            (blob) => {
              if (!blob) {
                reject(new Error('Failed to create blob'));
                return;
              }

              const url = URL.createObjectURL(blob);
              cache.set(size, { url, timestamp: Date.now() });
              resolve(url);
            },
            'image/jpeg',
            0.85 // 85% quality balance between size and quality
          );
        } catch (error) {
          reject(error);
        }
      };

      img.onerror = () => {
        reject(new Error(`Failed to load image: ${imageUrl}`));
      };

      img.onabort = () => {
        reject(new Error('Image load aborted'));
      };
    });
  } catch (error) {
    console.error('[imageThumbnailService] Error generating thumbnail:', error);
    throw error;
  }
}

export function getImageThumbnailUrl(fileId: string, size: ThumbnailSize = 'medium'): string | null {
  const cache = getCacheKey(fileId);
  const cached = cache.get(size);

  if (cached) {
    const age = Date.now() - cached.timestamp;
    if (age < CACHE_TTL) {
      return cached.url;
    } else {
      // Cleanup expired cache
      URL.revokeObjectURL(cached.url);
      cache.delete(size);
    }
  }

  return null;
}

export function isImageFile(mimeType: string): boolean {
  return mimeType?.startsWith('image/') ?? false;
}

export function clearThumbnailCache(fileId?: string): void {
  if (fileId) {
    const cache = getCacheKey(fileId);
    for (const [, cached] of cache.entries()) {
      URL.revokeObjectURL(cached.url);
    }
    thumbnailCache.delete(fileId);
  } else {
    // Clear all
    for (const [, sizes] of thumbnailCache.entries()) {
      for (const [, cached] of sizes.entries()) {
        URL.revokeObjectURL(cached.url);
      }
    }
    thumbnailCache.clear();
  }
}
