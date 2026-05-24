<script lang="ts">
  import * as Icons from 'lucide-svelte';

  export let mimeType: string = 'application/octet-stream';
  export let size: number = 24;
  export let name: string = '';

  function getIcon() {
    const ext = name.split('.').pop()?.toLowerCase() || '';

    if (mimeType.startsWith('image/')) return Icons.Image;
    if (mimeType.startsWith('video/')) return Icons.Film;
    if (mimeType.startsWith('audio/')) return Icons.Music;
    if (mimeType === 'application/pdf') return Icons.FileText;

    switch (ext) {
      case 'txt':
      case 'md':
      case 'rs':
      case 'ts':
      case 'js':
      case 'py':
      case 'go':
      case 'c':
      case 'cpp':
      case 'java':
        return Icons.Code;
      case 'zip':
      case 'tar':
      case 'gz':
      case 'rar':
        return Icons.Archive;
      case 'doc':
      case 'docx':
      case 'odt':
        return Icons.FileText;
      case 'xls':
      case 'xlsx':
      case 'csv':
        return Icons.Table;
      case 'ppt':
      case 'pptx':
        return Icons.Presentation;
      default:
        return Icons.File;
    }
  }

  $: Icon = getIcon();
</script>

<Icon {size} strokeWidth={1.5} />
