// Piece Puzzle: fill the shape with the pieces, a race. Rules page: docs/games/piece-puzzle.md
// RaceBar (the racers and the last round) is also used by Fence Building and Symmetry Puzzle.

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './PiecePuzzle.css';

type Cell = [number, number];

interface RaceFinish {
  id: string;
  timeMs: number | null;
  place: number | null;
  points: number;
}

/** The race part every race view carries. */
export interface RaceView {
  round: number;
  rounds: number;
  timeLeftMs: number;
  roundTimeMs: number;
  racers: { id: string; score: number; done: boolean }[];
  last: RaceFinish[] | null;
  over: boolean;
}

interface PieceView extends RaceView {
  width: number;
  height: number;
  target: boolean[];
  pieces: { shape: Cell[]; placed: Cell[] | null }[];
}

const ordinal = (n: number) => `${n}${n === 1 ? 'st' : n === 2 ? 'nd' : n === 3 ? 'rd' : 'th'}`;

/** Racers with their scores (a tick when done this round) and how the last round went. */
export function RaceBar({ view }: { view: RaceView }) {
  const mine = view.last?.find((f) => f.id === 'you');
  return (
    <>
      {view.racers.length > 1 && (
        <div className="race-bar">
          {view.racers.map((r) => (
            <span key={r.id} className={`race-chip${r.done ? ' done' : ''}${r.id === 'you' ? ' me' : ''}`}>
              {nameOf(r.id)} <strong>{r.score}</strong>
              {r.done && ' ✓'}
            </span>
          ))}
        </div>
      )}
      {mine && (
        <p className={`last ${mine.place ? 'ok' : 'bad'}`}>
          Last round: {mine.place ? `${view.racers.length > 1 ? `${ordinal(mine.place)} place, ` : ''}+${mine.points}` : 'not finished'}
          {view.racers.length > 1 &&
            ` · ${view.last
              ?.filter((f) => f.id !== 'you')
              .map((f) => `${nameOf(f.id)} ${f.place ? ordinal(f.place) : '–'}`)
              .join(', ')}`}
        </p>
      )}
    </>
  );
}

const PIECE_COLOURS = [
  'var(--g-yellow)',
  'var(--g-blue)',
  'var(--g-green)',
  'var(--g-red)',
  'var(--g-purple)',
  'var(--g-orange)',
  'var(--g-cyan)',
];

function normalize(cells: Cell[]): Cell[] {
  const mx = Math.min(...cells.map((c) => c[0]));
  const my = Math.min(...cells.map((c) => c[1]));
  return cells.map(([x, y]) => [x - mx, y - my] as Cell).sort((a, b) => a[1] - b[1] || a[0] - b[0]);
}

const turn = (cells: Cell[]) => normalize(cells.map(([x, y]) => [-y, x]));
const flip = (cells: Cell[]) => normalize(cells.map(([x, y]) => [-x, y]));

/** The cell nearest the middle of the shape: it goes where you tap. */
function anchorOf(cells: Cell[]): Cell {
  const cx = cells.reduce((s, c) => s + c[0], 0) / cells.length;
  const cy = cells.reduce((s, c) => s + c[1], 0) / cells.length;
  return cells.reduce((best, c) => (Math.hypot(c[0] - cx, c[1] - cy) < Math.hypot(best[0] - cx, best[1] - cy) ? c : best));
}

function ShapeIcon({ cells, colour, mark }: { cells: Cell[]; colour: string; mark: boolean }) {
  const w = Math.max(...cells.map((c) => c[0])) + 1;
  const h = Math.max(...cells.map((c) => c[1])) + 1;
  const size = Math.max(w, h, 3);
  const a = anchorOf(cells);
  return (
    <svg viewBox={`${-(size - w) / 2} ${-(size - h) / 2} ${size} ${size}`} className="pp-icon" aria-hidden="true">
      {cells.map(([x, y]) => (
        <rect key={`${x},${y}`} x={x + 0.05} y={y + 0.05} width={0.9} height={0.9} rx={0.12} style={{ fill: colour }} />
      ))}
      {mark && <circle cx={a[0] + 0.5} cy={a[1] + 0.5} r={0.16} className="pp-anchor" />}
    </svg>
  );
}

export default function PiecePuzzle({ view: raw }: ScreenProps) {
  const view = raw as PieceView;
  const [sel, setSel] = useState<{ piece: number; shape: Cell[] } | null>(null);
  const [forRound, setForRound] = useState(view.round);
  if (forRound !== view.round) {
    setForRound(view.round);
    setSel(null);
  }
  const { width: w, height: h } = view;
  const owner = new Map<string, number>();
  view.pieces.forEach((p, k) => {
    for (const [x, y] of p.placed ?? []) owner.set(`${x},${y}`, k);
  });
  const pick = (k: number) => {
    const p = view.pieces[k];
    if (!p) return;
    setSel(sel?.piece === k ? null : { piece: k, shape: normalize(p.placed ?? p.shape) });
  };
  const tapCell = (x: number, y: number) => {
    const here = owner.get(`${x},${y}`);
    if (sel) {
      const [ax, ay] = anchorOf(sel.shape);
      const cells = sel.shape.map(([cx, cy]) => [cx - ax + x, cy - ay + y] as Cell);
      const fits = cells.every(([cx, cy]) => {
        const o = owner.get(`${cx},${cy}`);
        return cx >= 0 && cy >= 0 && cx < w && cy < h && view.target[cy * w + cx] && (o === undefined || o === sel.piece);
      });
      if (fits) {
        move({ type: 'place', piece: sel.piece, cells });
        setSel(null);
        return;
      }
    }
    if (here !== undefined) {
      // Pick a placed piece up again, as it lies.
      move({ type: 'remove', piece: here });
      setSel({ piece: here, shape: normalize(view.pieces[here]?.placed ?? []) });
    }
  };
  const doTurn = () => sel && setSel({ ...sel, shape: turn(sel.shape) });
  const doFlip = () => sel && setSel({ ...sel, shape: flip(sel.shape) });
  useKeys((key) => {
    const k = Number(key) - 1;
    if (/^[1-9]$/.test(key) && k < view.pieces.length) pick(k);
    else if (key === 'r' || key === 'R') doTurn();
    else if (key === 'f' || key === 'F') doFlip();
    else if (key === 'Escape') setSel(null);
    else return false;
    return true;
  });
  return (
    <Play
      top={
        <>
          <GameTop title="Piece Puzzle" step={`Round ${view.round} of ${view.rounds}`} score={view.racers[0]?.score ?? 0} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.roundTimeMs} />
          <RaceBar view={view} />
        </>
      }
      stage={
        <svg className="drawing pp-board" viewBox={`-0.2 -0.2 ${w + 0.4} ${h + 0.4}`} role="img" aria-label="Target shape">
          {view.target.map((t, i) => {
            if (!t) return null;
            const x = i % w;
            const y = Math.floor(i / w);
            const k = owner.get(`${x},${y}`);
            return (
              // biome-ignore lint/a11y/useSemanticElements: an SVG cell cannot be a <button>
              <rect
                key={`${x},${y}`}
                x={x + 0.04}
                y={y + 0.04}
                width={0.92}
                height={0.92}
                rx={0.1}
                className={`pp-cell${k !== undefined ? ' filled' : ''}`}
                style={k !== undefined ? { fill: PIECE_COLOURS[k % PIECE_COLOURS.length] } : undefined}
                role="button"
                tabIndex={-1}
                onClick={() => tapCell(x, y)}
                onKeyDown={() => {}}
              />
            );
          })}
        </svg>
      }
      controls={
        <>
          <div className="pp-tray">
            {view.pieces.map((p, k) => (
              <button
                // biome-ignore lint/suspicious/noArrayIndexKey: pieces are positional
                key={k}
                type="button"
                className={`pp-piece${sel?.piece === k ? ' sel' : ''}${p.placed ? ' placed' : ''}`}
                onClick={() => pick(k)}
                aria-label={`Piece ${k + 1}`}
              >
                <ShapeIcon
                  cells={sel?.piece === k ? sel.shape : normalize(p.placed ?? p.shape)}
                  colour={PIECE_COLOURS[k % PIECE_COLOURS.length] ?? 'var(--muted)'}
                  mark={sel?.piece === k}
                />
              </button>
            ))}
          </div>
          <p className="muted small center">
            {sel
              ? 'Tap the shape where the dotted cell should go.'
              : 'Pick a piece, turn or flip it, then tap the shape. Tap a placed piece to pick it up.'}
          </p>
          <div className="row">
            <button type="button" className="btn" disabled={!sel} onClick={doTurn}>
              ⟳ Turn
            </button>
            <button type="button" className="btn" disabled={!sel} onClick={doFlip}>
              ⇋ Flip
            </button>
          </div>
          <button type="button" className="btn ghost" onClick={() => move({ type: 'giveUp' })}>
            Give up this round
          </button>
          <KeyHint>Keys: 1–{view.pieces.length} pick a piece, R turns, F flips, Esc drops it.</KeyHint>
        </>
      }
    />
  );
}
