function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function inlineMarkdown(value: string): string {
  return escapeHtml(value)
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(.+?)\*/g, "<em>$1</em>")
    .replace(/`([^`]+?)`/g, "<code>$1</code>")
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noreferrer">$1</a>');
}

export interface MarkdownHeading {
  level: number;
  text: string;
  slug: string;
}

export function extractHeadings(markdown: string): MarkdownHeading[] {
  return markdown
    .split(/\r?\n/)
    .map((line) => line.match(/^(#{1,6})\s+(.+)$/))
    .filter((match): match is RegExpMatchArray => Boolean(match))
    .map((match) => {
      const text = match[2].trim();
      return {
        level: match[1].length,
        text,
        slug: text
          .toLowerCase()
          .replace(/[^a-z0-9\s-]/g, "")
          .trim()
          .replace(/\s+/g, "-"),
      };
    });
}

export function renderMarkdown(markdown: string): string {
  const lines = markdown.replace(/\r\n/g, "\n").split("\n");
  const html: string[] = [];
  let inCode = false;
  let inList = false;

  const closeList = () => {
    if (inList) {
      html.push("</ul>");
      inList = false;
    }
  };

  for (const rawLine of lines) {
    const line = rawLine.trimEnd();

    if (line.startsWith("```")) {
      closeList();
      if (!inCode) {
        inCode = true;
        html.push('<pre class="md-code"><code>');
      } else {
        inCode = false;
        html.push("</code></pre>");
      }
      continue;
    }

    if (inCode) {
      html.push(`${escapeHtml(rawLine)}\n`);
      continue;
    }

    if (!line.trim()) {
      closeList();
      html.push('<div class="md-gap"></div>');
      continue;
    }

    const heading = line.match(/^(#{1,6})\s+(.+)$/);
    if (heading) {
      closeList();
      const level = Math.min(6, heading[1].length);
      const text = inlineMarkdown(heading[2].trim());
      html.push(`<h${level} class="md-h${level}">${text}</h${level}>`);
      continue;
    }

    const checklist = line.match(/^- \[( |x)\] (.+)$/i);
    if (checklist) {
      if (!inList) {
        html.push('<ul class="md-list md-checklist">');
        inList = true;
      }
      const checked = checklist[1].toLowerCase() === "x";
      html.push(
        `<li class="md-check-item"><span class="md-check ${checked ? "checked" : ""}"></span><span>${inlineMarkdown(checklist[2])}</span></li>`,
      );
      continue;
    }

    const bullet = line.match(/^[-*]\s+(.+)$/);
    if (bullet) {
      if (!inList) {
        html.push('<ul class="md-list">');
        inList = true;
      }
      html.push(`<li>${inlineMarkdown(bullet[1])}</li>`);
      continue;
    }

    const quote = line.match(/^>\s+(.+)$/);
    if (quote) {
      closeList();
      html.push(`<blockquote class="md-callout">${inlineMarkdown(quote[1])}</blockquote>`);
      continue;
    }

    if (line === "---") {
      closeList();
      html.push('<hr class="md-rule" />');
      continue;
    }

    closeList();
    html.push(`<p>${inlineMarkdown(line)}</p>`);
  }

  closeList();
  if (inCode) {
    html.push("</code></pre>");
  }
  return html.join("");
}
