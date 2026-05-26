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

  let Icon = $derived(getIcon());
</script>

<Icon {size} strokeWidth={1.5} />
