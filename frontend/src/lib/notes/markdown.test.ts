import { describe, it, expect } from 'vitest';
import { markdownToHtml, docToMarkdown, type PMNode } from './markdown';

describe('markdownToHtml', () => {
  it('renders headings', () => {
    expect(markdownToHtml('# Title')).toBe('<h1>Title</h1>');
    expect(markdownToHtml('### Sub')).toBe('<h3>Sub</h3>');
  });

  it('renders inline marks', () => {
    expect(markdownToHtml('**bold** and *italic* and `code`')).toBe(
      '<p><strong>bold</strong> and <em>italic</em> and <code>code</code></p>',
    );
  });

  it('keeps code span contents literal', () => {
    expect(markdownToHtml('`**not bold**`')).toBe('<p><code>**not bold**</code></p>');
  });

  it('renders links', () => {
    expect(markdownToHtml('[TSSP](https://example.com)')).toBe(
      '<p><a href="https://example.com">TSSP</a></p>',
    );
  });

  it('renders bulleted lists', () => {
    expect(markdownToHtml('- one\n- two')).toBe('<ul>\n<li>one</li>\n<li>two</li>\n</ul>');
  });

  it('renders ordered lists', () => {
    expect(markdownToHtml('1. one\n2. two')).toBe('<ol>\n<li>one</li>\n<li>two</li>\n</ol>');
  });

  it('renders task lists', () => {
    expect(markdownToHtml('- [ ] todo\n- [x] done')).toBe(
      '<ul data-type="taskList">\n' +
        '<li data-type="taskItem" data-checked="false"><p>todo</p></li>\n' +
        '<li data-type="taskItem" data-checked="true"><p>done</p></li>\n' +
        '</ul>',
    );
  });

  it('renders fenced code blocks with language', () => {
    expect(markdownToHtml('```rust\nfn main() {}\n```')).toBe(
      '<pre><code class="language-rust">fn main() {}</code></pre>',
    );
  });

  it('renders pipe tables', () => {
    const html = markdownToHtml('| A | B |\n| --- | --- |\n| 1 | 2 |');
    expect(html).toBe(
      '<table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></table>',
    );
  });

  it('renders blockquotes and horizontal rules', () => {
    expect(markdownToHtml('> quoted')).toBe('<blockquote><p>quoted</p></blockquote>');
    expect(markdownToHtml('---')).toBe('<hr>');
  });
});

describe('docToMarkdown', () => {
  const doc = (content: PMNode[]): PMNode => ({ type: 'doc', content });
  const text = (t: string, marks?: { type: string; attrs?: Record<string, unknown> }[]): PMNode => ({
    type: 'text',
    text: t,
    marks,
  });

  it('serializes headings and paragraphs', () => {
    const d = doc([
      { type: 'heading', attrs: { level: 2 }, content: [text('Hello')] },
      { type: 'paragraph', content: [text('world')] },
    ]);
    expect(docToMarkdown(d)).toBe('## Hello\n\nworld');
  });

  it('serializes inline marks', () => {
    const d = doc([
      {
        type: 'paragraph',
        content: [text('b', [{ type: 'bold' }]), text(' '), text('i', [{ type: 'italic' }])],
      },
    ]);
    expect(docToMarkdown(d)).toBe('**b** *i*');
  });

  it('serializes links and code', () => {
    const d = doc([
      {
        type: 'paragraph',
        content: [
          text('site', [{ type: 'link', attrs: { href: 'https://x.io' } }]),
          text(' '),
          text('x()', [{ type: 'code' }]),
        ],
      },
    ]);
    expect(docToMarkdown(d)).toBe('[site](https://x.io) `x()`');
  });

  it('serializes task lists', () => {
    const d = doc([
      {
        type: 'taskList',
        content: [
          { type: 'taskItem', attrs: { checked: false }, content: [{ type: 'paragraph', content: [text('a')] }] },
          { type: 'taskItem', attrs: { checked: true }, content: [{ type: 'paragraph', content: [text('b')] }] },
        ],
      },
    ]);
    expect(docToMarkdown(d)).toBe('- [ ] a\n- [x] b');
  });

  it('serializes tables', () => {
    const cell = (t: string): PMNode => ({ type: 'tableCell', content: [{ type: 'paragraph', content: [text(t)] }] });
    const header = (t: string): PMNode => ({ type: 'tableHeader', content: [{ type: 'paragraph', content: [text(t)] }] });
    const d = doc([
      {
        type: 'table',
        content: [
          { type: 'tableRow', content: [header('A'), header('B')] },
          { type: 'tableRow', content: [cell('1'), cell('2')] },
        ],
      },
    ]);
    expect(docToMarkdown(d)).toBe('| A | B |\n| --- | --- |\n| 1 | 2 |');
  });

  it('serializes code blocks', () => {
    const d = doc([
      { type: 'codeBlock', attrs: { language: 'js' }, content: [text('const a = 1;')] },
    ]);
    expect(docToMarkdown(d)).toBe('```js\nconst a = 1;\n```');
  });
});

describe('round-trip stability', () => {
  it('markdown survives html->(simulated)->markdown for common shapes', () => {
    // We can only assert html generation here (DOM parsing happens in Tiptap),
    // but verify the html is well-formed for the key shapes.
    const md = '# Title\n\nSome **bold** text.\n\n- a\n- b\n\n> quote';
    const html = markdownToHtml(md);
    expect(html).toContain('<h1>Title</h1>');
    expect(html).toContain('<strong>bold</strong>');
    expect(html).toContain('<ul>');
    expect(html).toContain('<blockquote>');
  });
});
