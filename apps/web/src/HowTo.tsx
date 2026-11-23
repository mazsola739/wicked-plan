// The "How to play" text of each game comes from its rules page, docs/games/<id>.md,
// so the docs stay the one source. The section uses only paragraphs, "- " bullets and **bold**.

import type { ReactNode } from 'react';

const PAGES = import.meta.glob('../../../docs/games/*.md', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

type Block = { list: boolean; lines: string[] };

function blocks(id: string): Block[] {
  const md = PAGES[`../../../docs/games/${id}.md`] ?? '';
  const section = md.split(/^## How to play\s*$/m)[1]?.split(/^## /m)[0] ?? '';
  const out: Block[] = [];
  let cur: Block | null = null;
  for (const raw of section.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) {
      cur = null;
    } else if (line.startsWith('- ')) {
      if (!cur?.list) {
        cur = { list: true, lines: [] };
        out.push(cur);
      }
      cur.lines.push(line.slice(2));
    } else if (cur && (!cur.list || raw.startsWith(' '))) {
      // A wrapped line continues the bullet or paragraph above it.
      cur.lines[cur.lines.length - 1] += ` ${line}`;
    } else {
      cur = { list: false, lines: [line] };
      out.push(cur);
    }
  }
  return out;
}

function inline(text: string): ReactNode[] {
  return text.split(/\*\*(.+?)\*\*/).map((part, i) => (i % 2 ? <strong key={part}>{part}</strong> : part));
}

export function HowToPlay({ id, name, onClose }: { id: string; name: string; onClose: () => void }) {
  const list = blocks(id);
  return (
    <section className="card howto">
      <div className="label">{name}</div>
      <h2>How to play</h2>
      {list.length ? (
        list.map((b) =>
          b.list ? (
            <ul key={b.lines[0]}>
              {b.lines.map((l) => (
                <li key={l}>{inline(l)}</li>
              ))}
            </ul>
          ) : (
            <p key={b.lines[0]}>{inline(b.lines[0] ?? '')}</p>
          ),
        )
      ) : (
        <p className="muted">No rules text for this game yet.</p>
      )}
      <button type="button" className="btn primary" onClick={onClose}>
        Play
      </button>
    </section>
  );
}
