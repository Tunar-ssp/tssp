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

  // Returns the lucide icon plus a type color so files read at a glance,
  // like a real file explorer instead of a wall of identical gray glyphs.
  function getMeta(): { icon: any; color: string } {
    const ext = name.split('.').pop()?.toLowerCase() || '';
    const mimePrefix = mimeType.split('/')[0];

    const image = { icon: Icons.Image, color: '#5be39a' };
    const video = { icon: Icons.Film, color: '#ff5fa2' };
    const audio = { icon: Icons.Music, color: '#a394ff' };
    const code = { icon: Icons.FileCode2, color: '#5b8cff' };
    const doc = { icon: Icons.FileText, color: '#ff8a3d' };
    const pdf = { icon: Icons.FileText, color: '#ff6b6b' };
    const sheet = { icon: Icons.Sheet, color: '#34d399' };
    const slides = { icon: Icons.Presentation, color: '#fbbf24' };
    const archive = { icon: Icons.FileArchive, color: '#fbbf24' };
    const text = { icon: Icons.FileText, color: '#b9bdc7' };

    switch (ext) {
      case 'jpg': case 'jpeg': case 'png': case 'gif': case 'svg':
      case 'webp': case 'bmp': case 'tiff': case 'ico': case 'heif': case 'heic': case 'avif':
        return image;
      case 'mp4': case 'webm': case 'mov': case 'mkv': case 'avi':
      case 'flv': case 'wmv': case 'mts': case 'm2ts': case 'm4v': case 'mxf':
        return video;
      case 'mp3': case 'wav': case 'flac': case 'aac': case 'm4a':
      case 'ogg': case 'opus': case 'aiff': case 'wma': case 'alac':
        return audio;
      case 'rs': case 'tsx': case 'jsx': case 'py': case 'go': case 'c':
      case 'cpp': case 'h': case 'java': case 'rb': case 'php': case 'sql':
      case 'sh': case 'json': case 'yaml': case 'yml': case 'xml': case 'html':
      case 'css': case 'scss': case 'svelte': case 'vue': case 'toml':
        return code;
      case 'ts': case 'js': case 'mjs': case 'cjs':
        return code;
      case 'txt': case 'log': case 'rtf':
        return text;
      case 'md': case 'mdx':
        return { icon: Icons.FileText, color: '#58d6e0' };
      case 'zip': case 'tar': case 'gz': case 'rar': case '7z': case 'bz2': case 'xz':
        return archive;
      case 'pdf':
        return pdf;
      case 'doc': case 'docx': case 'odt':
        return doc;
      case 'xls': case 'xlsx': case 'csv': case 'ods': case 'numbers':
        return sheet;
      case 'ppt': case 'pptx': case 'odp': case 'key':
        return slides;
    }

    if (mimePrefix === 'image') return image;
    if (mimePrefix === 'video') return video;
    if (mimePrefix === 'audio') return audio;
    if (mimeType === 'application/pdf') return pdf;
    if (mimeType.includes('json') || mimeType.includes('xml') || mimeType.includes('code')) return code;
    if (mimeType.includes('text')) return text;

    return { icon: Icons.File, color: '#7c8190' };
  }

  let meta = $derived(getMeta());
  let Glyph = $derived(meta.icon);
</script>

<Glyph {size} strokeWidth={1.75} color={meta.color} />
