// The "How to play" text of each game comes from its rules page, docs/games/<id>.md,
// so the docs stay the one source. The section uses only paragraphs, "- " bullets and **bold**.
// It is shown as a transmission from the Observer (docs/design.md, Broadcast feel): a live feed,
// the Goal line typing itself out, then the rest; a tap or any key shows everything at once.

import { type ReactNode, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Emblem, reducedMotion } from './Emblem';
import { GameIcon } from './GameIcon';

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

const GOAL = /^\*\*Goal:\*\*\s*/;
const TYPE_MS = 22;

/**
 * The Goal line typing itself out. The untyped rest is already laid out (invisibly), so nothing
 * jumps; the plain text types, and the **bold** comes back when it is done.
 */
function TypedGoal({ goal, skip, onDone }: { goal: string; skip: boolean; onDone: () => void }) {
  const plain = goal.replace(/\*\*/g, '');
  const [shown, setShown] = useState(() => (reducedMotion() ? plain.length : 0));
  useEffect(() => {
    if (reducedMotion()) return;
    const timer = window.setInterval(() => {
      setShown((n) => {
        if (n >= plain.length) window.clearInterval(timer);
        return Math.min(n + 1, plain.length);
      });
    }, TYPE_MS);
    return () => window.clearInterval(timer);
  }, [plain]);
  const done = skip || shown >= plain.length;
  useEffect(() => {
    if (done) onDone();
  }, [done, onDone]);
  return (
    <p className="tx-goal">
      <span className="sr-only">Goal: {plain}</span>
      <span aria-hidden="true">
        <strong>Goal:</strong>{' '}
        {done ? (
          inline(goal)
        ) : (
          <>
            {plain.slice(0, shown)}
            <span className="tx-caret" />
            <span className="tx-pending">{plain.slice(shown)}</span>
          </>
        )}
      </span>
    </p>
  );
}

// "Don't show again for this game": its rules stop opening by themselves on this browser (the How
// to play button still opens them). Storage may be blocked; then the rules just open every time.
const hideKey = (id: string) => `wp-hide-rules-${id}`;

export function rulesHidden(id: string): boolean {
  try {
    return localStorage.getItem(hideKey(id)) === '1';
  } catch {
    return false;
  }
}

function setRulesHidden(id: string, hide: boolean): void {
  try {
    if (hide) localStorage.setItem(hideKey(id), '1');
    else localStorage.removeItem(hideKey(id));
  } catch {}
}

export function HowToPlay({ id, name, onClose }: { id: string; name: string; onClose: () => void }) {
  const list = useMemo(() => blocks(id), [id]);
  const [hide, setHide] = useState(() => rulesHidden(id));
  const first = list[0];
  const goal = first && !first.list && GOAL.test(first.lines[0] ?? '') ? (first.lines[0] ?? '').replace(GOAL, '') : '';
  const rest = goal ? list.slice(1) : list;
  const [skip, setSkip] = useState(false);
  const [done, setDone] = useState(!goal);
  const finish = useCallback(() => setDone(true), []);

  // Focus moves to the heading when the transmission opens (keyboard and screen-reader users land
  // on it); a tap or any key inside shows the whole text at once.
  const box = useRef<HTMLElement>(null);
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    heading.current?.focus({ preventScroll: true });
    const el = box.current;
    const reveal = () => setSkip(true);
    el?.addEventListener('pointerdown', reveal);
    el?.addEventListener('keydown', reveal);
    return () => {
      el?.removeEventListener('pointerdown', reveal);
      el?.removeEventListener('keydown', reveal);
    };
  }, []);

  let step = 0;
  return (
    <section ref={box} className="card howto transmission">
      <header className="tx-bar">
        <span className="tx-live">Live</span>
        <span className="label">From the Observer</span>
        <span className="tx-channel">CH 08</span>
      </header>
      {/* The monitor sits in the top-left corner; the text flows around it, to its right and below. */}
      <div className="tx-body">
        {/* The Observer and the game's icon take turns on the monitor, switching with a flicker. */}
        <div className="tx-screen" aria-hidden="true">
          <Emblem className="tx-emblem" watching decorative />
          <GameIcon id={id} className="tx-icon" neon />
        </div>
        <div className="label">How to play</div>
        <h2 ref={heading} tabIndex={-1} className="tx-title">
          {name}
        </h2>
        {goal && <TypedGoal goal={goal} skip={skip} onDone={finish} />}
        <div className={done ? 'tx-rest' : 'tx-rest waiting'}>
          {rest.map((b) =>
            b.list ? (
              <ul key={b.lines[0]}>
                {b.lines.map((l) => (
                  <li key={l} className="tx-in" style={{ animationDelay: `${step++ * 90}ms` }}>
                    {inline(l)}
                  </li>
                ))}
              </ul>
            ) : (
              <p key={b.lines[0]} className="tx-in" style={{ animationDelay: `${step++ * 90}ms` }}>
                {inline(b.lines[0] ?? '')}
              </p>
            ),
          )}
        </div>
      </div>
      <label className="tx-hide">
        <input
          type="checkbox"
          checked={hide}
          onChange={(e) => {
            setHide(e.target.checked);
            setRulesHidden(id, e.target.checked);
          }}
        />
        Don't show again for this game
      </label>
      <button type="button" className="btn primary" onClick={onClose}>
        Play
      </button>
    </section>
  );
}
