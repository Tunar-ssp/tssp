function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

function renderInline(source: string): string {
  return escapeHtml(source)
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noreferrer">$1</a>');
}

export function renderMarkdownLite(source: string): string {
  const lines = source.replace(/\r\n/g, '\n').split('\n');
  const html: string[] = [];
  let paragraph: string[] = [];
  let listMode: 'ul' | 'ol' | null = null;
  let inCodeFence = false;
  let codeFenceLines: string[] = [];

  function flushParagraph() {
    if (!paragraph.length) return;
    html.push(`<p>${renderInline(paragraph.join(' '))}</p>`);
    paragraph = [];
  }

  function closeList() {
    if (!listMode) return;
    html.push(`</${listMode}>`);
    listMode = null;
  }

  function flushCodeFence() {
    if (!inCodeFence) return;
    html.push(`<pre><code>${escapeHtml(codeFenceLines.join('\n'))}</code></pre>`);
    inCodeFence = false;
    codeFenceLines = [];
  }

  for (const line of lines) {
    if (line.trim().startsWith('```')) {
      flushParagraph();
      closeList();
      if (inCodeFence) {
        flushCodeFence();
      } else {
        inCodeFence = true;
      }
      continue;
    }

    if (inCodeFence) {
      codeFenceLines.push(line);
      continue;
    }

    const trimmed = line.trim();
    if (!trimmed) {
      flushParagraph();
      closeList();
      continue;
    }

    const headingMatch = trimmed.match(/^(#{1,6})\s+(.+)$/);
    if (headingMatch) {
      flushParagraph();
      closeList();
      const level = headingMatch[1].length;
      html.push(`<h${level}>${renderInline(headingMatch[2])}</h${level}>`);
      continue;
    }

    const checklistMatch = trimmed.match(/^- \[( |x)\]\s+(.+)$/i);
    if (checklistMatch) {
      flushParagraph();
      if (listMode !== 'ul') {
        closeList();
        listMode = 'ul';
        html.push('<ul class="task-list">');
      }
      const checked = checklistMatch[1].toLowerCase() === 'x';
      html.push(
        `<li class="task-item"><input type="checkbox" disabled ${checked ? 'checked' : ''}>` +
          `<span>${renderInline(checklistMatch[2])}</span></li>`
      );
      continue;
    }

    const ulMatch = trimmed.match(/^[-*]\s+(.+)$/);
    if (ulMatch) {
      flushParagraph();
      if (listMode !== 'ul') {
        closeList();
        listMode = 'ul';
        html.push('<ul>');
      }
      html.push(`<li>${renderInline(ulMatch[1])}</li>`);
      continue;
    }

    const olMatch = trimmed.match(/^\d+\.\s+(.+)$/);
    if (olMatch) {
      flushParagraph();
      if (listMode !== 'ol') {
        closeList();
        listMode = 'ol';
        html.push('<ol>');
      }
      html.push(`<li>${renderInline(olMatch[1])}</li>`);
      continue;
    }

    const quoteMatch = trimmed.match(/^>\s+(.+)$/);
    if (quoteMatch) {
      flushParagraph();
      closeList();
      html.push(`<blockquote><p>${renderInline(quoteMatch[1])}</p></blockquote>`);
      continue;
    }

    paragraph.push(trimmed);
  }

  flushParagraph();
  closeList();
  flushCodeFence();

  return html.join('\n');
}

export function estimateBlockCount(source: string): number {
  return source
    .split(/\n\s*\n/g)
    .map((section) => section.trim())
    .filter(Boolean).length;
}

