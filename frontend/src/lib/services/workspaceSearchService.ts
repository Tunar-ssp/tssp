/**
 * Workspace Search Service
 *
 * Handles searching and replacing content across workspace files.
 * Features:
 * - Search with regex or literal strings
 * - Case-sensitive and whole-word matching
 * - Find/replace operations
 * - Search result aggregation
 */

export interface SearchOptions {
  matchCase: boolean;
  wholeWord: boolean;
  regex: boolean;
}

export interface SearchResult {
  filePath: string;
  matches: Match[];
  totalMatches: number;
}

export interface Match {
  line: number;
  column: number;
  text: string;
  lineContent: string;
  matchStart: number;
  matchEnd: number;
}

/**
 * Find all matches of a pattern in text
 */
export function findMatches(
  content: string,
  pattern: string,
  options: SearchOptions
): Match[] {
  const matches: Match[] = [];

  try {
    let regex: RegExp;

    if (options.regex) {
      const flags = options.matchCase ? 'g' : 'gi';
      regex = new RegExp(pattern, flags);
    } else {
      // Escape special regex characters
      const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      let finalPattern = escaped;

      if (options.wholeWord) {
        finalPattern = `\\b${escaped}\\b`;
      }

      const flags = options.matchCase ? 'g' : 'gi';
      regex = new RegExp(finalPattern, flags);
    }

    const lines = content.split('\n');
    let charIndex = 0;

    lines.forEach((line, lineIndex) => {
      let lineMatches;
      const lineRegex = new RegExp(regex.source, regex.flags);

      while ((lineMatches = lineRegex.exec(line)) !== null) {
        const column = lineMatches.index;
        const matchText = lineMatches[0];

        matches.push({
          line: lineIndex + 1,
          column: column + 1,
          text: matchText,
          lineContent: line,
          matchStart: charIndex + column,
          matchEnd: charIndex + column + matchText.length,
        });
      }

      charIndex += line.length + 1; // +1 for newline
    });
  } catch (error) {
    console.error('Regex error:', error);
  }

  return matches;
}

/**
 * Replace all matches in text
 */
export function replaceMatches(
  content: string,
  pattern: string,
  replacement: string,
  options: SearchOptions
): { content: string; replacementCount: number } {
  try {
    let regex: RegExp;

    if (options.regex) {
      const flags = options.matchCase ? 'g' : 'gi';
      regex = new RegExp(pattern, flags);
    } else {
      const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      let finalPattern = escaped;

      if (options.wholeWord) {
        finalPattern = `\\b${escaped}\\b`;
      }

      const flags = options.matchCase ? 'g' : 'gi';
      regex = new RegExp(finalPattern, flags);
    }

    const matches = content.match(regex);
    const newContent = content.replace(regex, replacement);

    return {
      content: newContent,
      replacementCount: matches ? matches.length : 0,
    };
  } catch (error) {
    console.error('Replace error:', error);
    return {
      content,
      replacementCount: 0,
    };
  }
}

/**
 * Replace a single match at a specific position
 */
export function replaceSingleMatch(
  content: string,
  matchStart: number,
  matchEnd: number,
  replacement: string
): string {
  return content.substring(0, matchStart) + replacement + content.substring(matchEnd);
}

/**
 * Validate regex pattern
 */
export function isValidRegex(pattern: string): boolean {
  try {
    new RegExp(pattern);
    return true;
  } catch {
    return false;
  }
}

/**
 * Highlight matches in text (for display)
 */
export function highlightMatches(
  content: string,
  pattern: string,
  options: SearchOptions,
  maxResults = 1000
): string {
  if (!pattern || !options.regex) {
    return escapeHtml(content);
  }

  const matches = findMatches(content, pattern, options);
  if (matches.length === 0) {
    return escapeHtml(content);
  }

  const limited = matches.slice(0, maxResults);
  let html = '';
  let lastIndex = 0;

  limited.forEach((match) => {
    html += escapeHtml(content.substring(lastIndex, match.matchStart));
    html += `<mark>${escapeHtml(match.text)}</mark>`;
    lastIndex = match.matchEnd;
  });

  html += escapeHtml(content.substring(lastIndex));
  return html;
}

function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

/**
 * Search across multiple files
 */
export function searchFiles(
  files: Array<{ path: string; content: string }>,
  pattern: string,
  options: SearchOptions
): SearchResult[] {
  return files
    .map((file) => ({
      filePath: file.path,
      matches: findMatches(file.content, pattern, options),
      totalMatches: findMatches(file.content, pattern, options).length,
    }))
    .filter((result) => result.totalMatches > 0);
}
