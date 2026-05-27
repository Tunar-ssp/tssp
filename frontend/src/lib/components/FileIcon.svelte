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

    // Check MIME type first (most reliable)
    if (mimePrefix === 'image') return Icons.Image;
    if (mimePrefix === 'video') return Icons.Film;
    if (mimePrefix === 'audio') return Icons.Music;
    if (mimeType === 'application/pdf') return Icons.FileText;
    if (mimeType.includes('text') || mimeType.includes('code') || mimeType.includes('json') || mimeType.includes('xml')) return Icons.Code;

    // Fallback to extension-based detection
    switch (ext) {
      // Code files
      case 'txt':
      case 'md':
      case 'rs':
      case 'ts':
      case 'tsx':
      case 'js':
      case 'jsx':
      case 'py':
      case 'go':
      case 'c':
      case 'cpp':
      case 'h':
      case 'java':
      case 'rb':
      case 'php':
      case 'sql':
      case 'sh':
      case 'json':
      case 'yaml':
      case 'yml':
      case 'xml':
      case 'html':
      case 'css':
      case 'scss':
        return Icons.Code;
      // Archives
      case 'zip':
      case 'tar':
      case 'gz':
      case 'rar':
      case '7z':
      case 'bz2':
        return Icons.Archive;
      // Documents
      case 'doc':
      case 'docx':
      case 'odt':
      case 'txt':
      case 'rtf':
        return Icons.FileText;
      // Spreadsheets
      case 'xls':
      case 'xlsx':
      case 'csv':
      case 'ods':
        return Icons.Table;
      // Presentations
      case 'ppt':
      case 'pptx':
      case 'odp':
        return Icons.Presentation;
      // Images
      case 'jpg':
      case 'jpeg':
      case 'png':
      case 'gif':
      case 'svg':
      case 'webp':
      case 'bmp':
      case 'tiff':
        return Icons.Image;
      // Videos
      case 'mp4':
      case 'webm':
      case 'mov':
      case 'mkv':
      case 'avi':
      case 'flv':
      case 'wmv':
        return Icons.Film;
      // Audio
      case 'mp3':
      case 'wav':
      case 'flac':
      case 'aac':
      case 'm4a':
      case 'ogg':
        return Icons.Music;
      default:
        return Icons.File;
    }
  }

  let Icon = $derived(getIcon());
</script>

<Icon {size} strokeWidth={1.5} />
