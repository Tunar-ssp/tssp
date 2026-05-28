<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    mimeType?: string;
    size?: number;
    name?: string;
  }

  let {
    mimeType = 'application/octet-stream',
    size = 24,
    name = '',
  }: $$Props = $props();

  function getIcon() {
    const ext = name.split('.').pop()?.toLowerCase() || '';
    const mimePrefix = mimeType.split('/')[0];

    // PRIORITY 1: Check extension first (backend MIME detection is broken)
    // This is the most reliable method given current backend issues
    switch (ext) {
      // Image files
      case 'jpg': case 'jpeg': case 'png': case 'gif': case 'svg':
      case 'webp': case 'bmp': case 'tiff': case 'ico': case 'heif': case 'heic':
        return Icons.Image;
      // Video files
      case 'mp4': case 'webm': case 'mov': case 'mkv': case 'avi':
      case 'flv': case 'wmv': case 'mts': case 'm2ts': case 'ts': case 'mxf':
        return Icons.Film;
      // Audio files
      case 'mp3': case 'wav': case 'flac': case 'aac': case 'm4a':
      case 'ogg': case 'opus': case 'aiff': case 'wma': case 'alac':
        return Icons.Music;
      // Code files
      case 'txt': case 'md': case 'rs': case 'ts': case 'tsx': case 'js':
      case 'jsx': case 'py': case 'go': case 'c': case 'cpp': case 'h':
      case 'java': case 'rb': case 'php': case 'sql': case 'sh': case 'json':
      case 'yaml': case 'yml': case 'xml': case 'html': case 'css': case 'scss':
        return Icons.Code;
      // Archive files
      case 'zip': case 'tar': case 'gz': case 'rar': case '7z': case 'bz2': case 'xz':
        return Icons.Archive;
      // Document files
      case 'doc': case 'docx': case 'odt': case 'rtf': case 'pdf':
        return Icons.FileText;
      // Spreadsheet files
      case 'xls': case 'xlsx': case 'csv': case 'ods': case 'numbers':
        return Icons.Table;
      // Presentation files
      case 'ppt': case 'pptx': case 'odp': case 'key':
        return Icons.Presentation;
    }

    // PRIORITY 2: Fallback to MIME type check
    if (mimePrefix === 'image') return Icons.Image;
    if (mimePrefix === 'video') return Icons.Film;
    if (mimePrefix === 'audio') return Icons.Music;
    if (mimeType === 'application/pdf') return Icons.FileText;
    if (mimeType.includes('text') || mimeType.includes('code') || mimeType.includes('json') || mimeType.includes('xml')) return Icons.Code;

    // PRIORITY 3: Default fallback
    return Icons.File;
  }

  let Icon = $derived(getIcon());
</script>

<Icon {size} strokeWidth={1.5} />
