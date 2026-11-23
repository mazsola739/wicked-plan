// Hexagon, Knight's Tour and Magic Square.

import { useRef, useState } from 'react';
import type { HexView, KnightView, MagicView } from '../types';
import { GameTop, KeyHint, move, Play, RoundDots, TimerBar, useKeys } from './common';

// --- Hexagon ---------------------------------------------------------------

/** Axial (q, r) of the 19 cells, the same order as the server. */
const HEX_CELLS: [number, number][] = [];
for (let r = -2; r <= 2; r++) {
  for (let q = Math.max(-2, -2 - r); q <= Math.min(2, 2 - r); q++) HEX_CELLS.push([q, r]);
}
const R = 30;

function hexPoints(cx: number, cy: number): string {
  return Array.from({ length: 6 }, (_, i) => {
    const a = (Math.PI / 180) * (60 * i - 30);
    return `${cx + R * Math.cos(a)},${cy + R * Math.sin(a)}`;
  }).join(' ');
}

export function HexagonGame({ view }: { view: HexView }) {
  const [picked, setPicked] = useState<number[]>([]);
  const [forRound, setForRound] = useState(view.round);
  if (forRound !== view.round) {
    setForRound(view.round);
    setPicked([]);
  }
  const toggle = (i: number) =>
    !view.studying && setPicked((cur) => (cur.includes(i) ? cur.filter((c) => c !== i) : cur.length < 3 ? [...cur, i] : cur));
  const w = Math.sqrt(3) * R;
  const last = view.history[view.history.length - 1];
  const submit = () => picked.length === 3 && move({ type: 'answer', cells: picked });
  // Desktop: letters A–S pick cells, Backspace undoes, Enter submits (or ends the study time).
  useKeys((key) => {
    const cell = key.toUpperCase().charCodeAt(0) - 65;
    if (key.length === 1 && cell >= 0 && cell < HEX_CELLS.length) toggle(cell);
    else if (key === 'Backspace') setPicked((cur) => cur.slice(0, -1));
    else if (key === 'Enter') view.studying ? move({ type: 'ready' }) : submit();
    else return false;
    return true;
  });
  return (
    <Play
      top={
        <>
          <GameTop title="Hexagon" step={view.studying ? 'Memorise!' : `Round ${view.round} of ${view.rounds}`} score={view.score} />
          <RoundDots results={view.history.map((h) => h.correct)} total={view.rounds} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.phaseTimeMs} />
        </>
      }
      stage={
        <svg
          className="drawing hex"
          viewBox={`${-3 * w} ${-3.2 * R * 1.5} ${6 * w} ${6.4 * R * 1.5}`}
          role="img"
          aria-label="Hexagon board"
        >
          {HEX_CELLS.map(([q, r], i) => {
            const cx = w * (q + r / 2);
            const cy = 1.5 * R * r;
            const number = view.numbers?.[i];
            return (
              // biome-ignore lint/a11y/useSemanticElements: an SVG cell cannot be a <button>
              <g
                key={`${q},${r}`}
                className={`hex-cell${picked.includes(i) ? ' picked' : ''}`}
                onClick={() => toggle(i)}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => e.key === 'Enter' && toggle(i)}
              >
                <polygon points={hexPoints(cx, cy)} />
                <text x={cx} y={cy} className={number === undefined ? 'letter' : 'number'}>
                  {number ?? String.fromCharCode(65 + i)}
                </text>
              </g>
            );
          })}
        </svg>
      }
      controls={
        <>
          {view.target !== null && (
            <div className="target">
              Target: <strong>{view.target}</strong>
            </div>
          )}
          {view.studying ? (
            <button type="button" className="btn primary" onClick={() => move({ type: 'ready' })}>
              Ready
            </button>
          ) : (
            <button type="button" className="btn primary" disabled={picked.length !== 3} onClick={submit}>
              Submit
            </button>
          )}
          <p className="muted small center">Three neighbouring cells in a straight line that add up to the target.</p>
          {last && (
            <p className={`last ${last.correct ? 'ok' : 'bad'}`}>
              Last round:{' '}
              {last.correct ? `right, +${last.points}` : `the answer: ${last.solution.map((i) => String.fromCharCode(65 + i)).join('')}`}
            </p>
          )}
          <KeyHint>Keys: letters A–S pick cells, Enter submits.</KeyHint>
        </>
      }
    />
  );
}

// --- Knight's Tour ---------------------------------------------------------

/**
 * The knight, a faceted horse head in the square's colour (docs/design.md, "Icons are faceted"):
 * white facets lit from the top left, black ones down right, a dark eye and a white glint.
 */
function Knight() {
  return (
    <svg
      viewBox="0 0 40 40"
      width="1.2em"
      height="1.2em"
      fill="currentColor"
      role="img"
      aria-label="knight"
      style={{ display: 'block', margin: 'auto' }}
    >
      <path d="M22,2.5 24.5,6 27,9 30.5,16 31.5,25 30,32 11,32 13.5,26.5 15,21.5 10.5,23.5 6,22.5 5.5,18.5 12,11 18.5,6.5Z" />
      <path d="M9.5,32 30.5,32 33,34.5 33,38 7,38 7,34.5Z" />
      <path fill="#fff" fillOpacity=".45" d="M18.5,6.5 12,11 5.5,18.5 11,17Z M9.5,32 30.5,32 33,34.5 7,34.5Z" />
      <path fill="#fff" fillOpacity=".3" d="M22,2.5 24.5,6 18.5,6.5Z M5.5,18.5 11,17 10.5,23.5 6,22.5Z M13.5,26.5 22,24 20,32 11,32Z" />
      <path fill="#fff" fillOpacity=".15" d="M18.5,6.5 24.5,6 19,14 11,17Z M7,34.5 20,34.5 20,38 7,38Z" />
      <path fill="#fff" fillOpacity=".1" d="M11,17 19,14 15,21.5 10.5,23.5Z M19,14 22,24 13.5,26.5 15,21.5Z" />
      <path fill="#000" fillOpacity=".14" d="M24.5,6 27,9 30.5,16 19,14Z" />
      <path fill="#000" fillOpacity=".26" d="M30.5,16 31.5,25 22,24 19,14Z M20,34.5 33,34.5 33,38 20,38Z" />
      <path
        fill="#000"
        fillOpacity=".35"
        d="M22,24 31.5,25 30,32 20,32Z M24.5,6 27,9 30.5,16 31.5,25 29,23.5 28.8,19.8 26.6,18 26.7,14.4 24.5,12.6 24.4,9.4 22.6,8.2Z M21.4,4.8 22.6,6.4 20.6,6.6Z"
      />
      <path style={{ fill: 'var(--bg)' }} d="M13.4,12.6 15.8,10.8 17.8,11.4 15.6,13.6Z M6.8,19.8 8.8,19 9,20.6Z" />
      <path fill="#fff" d="M15.6,11.3 16.2,11.9 15.6,12.5 15,11.9Z M10,32.8 10.6,33.4 10,34 9.4,33.4Z" />
    </svg>
  );
}

export function KnightGame({ view }: { view: KnightView }) {
  const order = new Map(view.path.map(([x, y], i) => [`${x},${y}`, i + 1]));
  const targets = new Set(view.targets.map(([x, y]) => `${x},${y}`));
  const at = view.path[view.path.length - 1];
  return (
    <Play
      top={
        <>
          <GameTop title="Knight's Tour" step={`Board ${view.board} of ${view.boards} · ${view.size}×${view.size}`} score={view.score} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.totalTimeMs} />
        </>
      }
      stage={
        <div className="board" style={{ gridTemplateColumns: `repeat(${view.size}, 1fr)` }}>
          {Array.from({ length: view.size * view.size }, (_, i) => {
            const x = i % view.size;
            const y = Math.floor(i / view.size);
            const key = `${x},${y}`;
            const n = order.get(key);
            const here = at && at[0] === x && at[1] === y;
            return (
              <button
                key={key}
                type="button"
                className={`sq${(x + y) % 2 ? ' dark' : ''}${n ? ' seen' : ''}${here ? ' here' : ''}${targets.has(key) && view.path.length > 0 ? ' can' : ''}`}
                disabled={!targets.has(key)}
                onClick={() => move({ type: 'step', x, y })}
              >
                {here ? <Knight /> : (n ?? '')}
              </button>
            );
          })}
        </div>
      }
      controls={
        <>
          <p className="muted small center">
            {view.path.length === 0 ? 'Put the knight on any square.' : `${view.path.length} / ${view.size * view.size} squares`}
          </p>
          {view.stuck && <p className="last bad">Stuck: no free jump.</p>}
          <button
            type="button"
            className={`btn${view.stuck ? ' primary' : ''}`}
            disabled={view.path.length === 0}
            onClick={() => move({ type: 'restart' })}
          >
            Restart this board
          </button>
        </>
      }
    />
  );
}

// --- Magic Square ----------------------------------------------------------

export function MagicGame({ view }: { view: MagicView }) {
  const [sel, setSel] = useState<number | null>(null);
  const [forBoard, setForBoard] = useState(view.board);
  if (forBoard !== view.board) {
    setForBoard(view.board);
    setSel(null);
  }
  const n = view.size;
  const max = n * n;
  const used = new Set(view.cells.filter((v) => v > 0));
  const set = (value: number) => {
    if (sel === null) return;
    move({ type: 'set', cell: sel, value });
    // Jump to the next empty cell.
    const next = view.cells.findIndex((v, i) => i > sel && v === 0 && !view.given[i]);
    setSel(next >= 0 ? next : sel);
  };
  const full = view.cells.every((v) => v !== 0);
  // Desktop: type a number (two digits when needed), arrows move, Backspace clears, Enter checks.
  const typed = useRef({ digits: '', timer: 0 });
  useKeys((key) => {
    const t = typed.current;
    if (/^[0-9]$/.test(key) && sel !== null) {
      window.clearTimeout(t.timer);
      t.digits += key;
      const value = Number(t.digits);
      if (t.digits.length === 2 || value * 10 > max) {
        t.digits = '';
        if (value >= 1 && value <= max) set(value);
      } else {
        t.timer = window.setTimeout(() => {
          t.digits = '';
          if (value >= 1) set(value);
        }, 700);
      }
    } else if (key === 'Backspace' || key === 'Delete') set(0);
    else if (key.startsWith('Arrow')) {
      const d = ({ ArrowLeft: -1, ArrowRight: 1, ArrowUp: -n, ArrowDown: n } as Record<string, number>)[key] ?? 0;
      setSel((cur) => Math.min(max - 1, Math.max(0, (cur ?? -d) + d)));
    } else if (key === 'Enter' && full) move({ type: 'submit' });
    else return false;
    return true;
  });
  return (
    <Play
      top={
        <>
          <GameTop title="Magic Square" step={`Board ${view.board} of ${view.boards} · ${n}×${n}`} score={view.score} />
          <RoundDots results={view.solved} total={view.boards} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.boardTimeMs} />
        </>
      }
      stage={
        <div className="board magic" style={{ gridTemplateColumns: `repeat(${n}, 1fr)` }}>
          {view.cells.map((v, i) => (
            <button
              // biome-ignore lint/suspicious/noArrayIndexKey: cells are positional
              key={i}
              type="button"
              className={`sq${view.given[i] ? ' given' : ''}${sel === i ? ' here' : ''}`}
              disabled={view.given[i]}
              onClick={() => setSel(i)}
            >
              {v || ''}
            </button>
          ))}
        </div>
      }
      controls={
        <>
          <p className="muted small center">
            Use 1–{max} once each; every row, column and diagonal adds up to <strong>{view.magicSum}</strong>.
          </p>
          <div className="numbers" style={{ gridTemplateColumns: `repeat(${Math.min(n + 2, 7)}, 1fr)` }}>
            {Array.from({ length: max }, (_, k) => k + 1).map((v) => (
              <button key={v} type="button" className={`key${used.has(v) ? ' used' : ''}`} disabled={sel === null} onClick={() => set(v)}>
                {v}
              </button>
            ))}
            <button type="button" className="key" disabled={sel === null} onClick={() => set(0)}>
              ⌫
            </button>
          </div>
          {view.wrongTries > 0 && <p className="last bad">Not right yet ({view.wrongTries}×).</p>}
          <div className="row">
            <button type="button" className="btn ghost" onClick={() => move({ type: 'skip' })}>
              Give up
            </button>
            <button type="button" className="btn primary" disabled={!full} onClick={() => move({ type: 'submit' })}>
              Check
            </button>
          </div>
          <KeyHint>Keys: type a number, arrows move, Backspace clears, Enter checks.</KeyHint>
        </>
      }
    />
  );
}
