// Cooperative Puzzle: the wheel turns from player to player; place one piece each, finish the shapes.
// Rules page: docs/games/spinning-puzzle-wheel.md

import { useState } from 'react';
import './SpinningPuzzleWheel.css';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';

type Cell = [number, number];

interface PuzzleView {
  target: Cell[];
  pieces: Cell[][];
  placed: (Cell[] | null)[];
  done: boolean;
}

interface WheelView {
  players: string[];
  you: number | null;
  grid: number;
  puzzles: PuzzleView[];
  current: number;
  turn: number;
  yourTurn: boolean;
  turnsLeft: number;
  done: number;
  timeLeftMs: number;
  turnMs: number;
  last: string | null;
  score: number;
  over: boolean;
}

/** Same orientation as the rules: k % 4 quarter turns, mirrored when k >= 4, shifted to 0. */
function orient(cells: Cell[], k: number): Cell[] {
  const out = cells.map(([x0, y0]) => {
    let [x, y] = [x0, y0];
    for (let i = 0; i < k % 4; i++) [x, y] = [-y, x];
    return (k >= 4 ? [-x, y] : [x, y]) as Cell;
  });
  const mx = Math.min(...out.map((c) => c[0]));
  const my = Math.min(...out.map((c) => c[1]));
  return out.map(([x, y]) => [x - mx, y - my]);
}

/** A puzzle drawn on a `size` grid at (0, 0), cells of `c` units. */
function PuzzleCells({
  p,
  grid,
  c,
  ghost,
  ok,
  onCell,
}: {
  p: PuzzleView;
  grid: number;
  c: number;
  ghost?: Cell[];
  ok?: boolean;
  onCell?: (x: number, y: number) => void;
}) {
  const target = new Set(p.target.map(([x, y]) => `${x},${y}`));
  const owner = new Map<string, number>();
  p.placed.forEach((cells, i) => {
    for (const [x, y] of cells ?? []) owner.set(`${x},${y}`, i);
  });
  const cells = Array.from({ length: grid * grid }, (_, i) => ({ x: i % grid, y: Math.floor(i / grid) }));
  return (
    <>
      {cells.map(({ x, y }) => {
        const k = `${x},${y}`;
        const o = owner.get(k);
        return (
          // biome-ignore lint/a11y/useSemanticElements: an SVG cell cannot be a <button>
          <rect
            key={k}
            className={`spw-cell${target.has(k) ? ' target' : ''}${o !== undefined ? ` piece p${o}` : ''}`}
            x={x * c + c * 0.04}
            y={y * c + c * 0.04}
            width={c * 0.92}
            height={c * 0.92}
            rx={c * 0.12}
            role="button"
            tabIndex={-1}
            onClick={() => onCell?.(x, y)}
            onKeyDown={() => {}}
          />
        );
      })}
      {ghost?.map(([x, y]) => (
        <rect
          key={`g${x},${y}`}
          className={`spw-ghost${ok ? '' : ' bad'}`}
          x={x * c + c * 0.12}
          y={y * c + c * 0.12}
          width={c * 0.76}
          height={c * 0.76}
          rx={c * 0.1}
        />
      ))}
    </>
  );
}

function Wheel({ view }: { view: WheelView }) {
  const n = view.puzzles.length;
  const t = 14;
  const size = view.grid * t;
  return (
    <svg className="drawing spw-wheel" viewBox="-170 -170 340 340" role="img" aria-label="Puzzle wheel">
      <circle className="spw-rim" r={150} />
      <path className="spw-pointer" d="M -12 -168 L 12 -168 L 0 -146 Z" />
      {view.puzzles
        .map((p, i) => ({ p, i, id: `w${i}` }))
        .map(({ p, i, id }) => {
          const a = ((i - view.current) / n) * 2 * Math.PI - Math.PI / 2;
          const x = Math.cos(a) * 105 - size / 2;
          const y = Math.sin(a) * 105 - size / 2;
          return (
            <g
              key={id}
              className={`spw-thumb${p.done ? ' done' : ''}${i === view.current ? ' now' : ''}`}
              style={{ transform: `translate(${x}px, ${y}px)` }}
            >
              <rect className="spw-plate" x={-6} y={-6} width={size + 12} height={size + 12} rx={10} />
              <PuzzleCells p={p} grid={view.grid} c={t} />
            </g>
          );
        })}
      <text className="spw-center" y={-6}>
        {view.done}/{n}
      </text>
      <text className="spw-center small" y={16}>
        done
      </text>
    </svg>
  );
}

export default function SpinningPuzzleWheel({ view: raw }: ScreenProps) {
  const view = raw as WheelView;
  const you = view.you ?? 0;
  const g = view.grid;
  const p = view.puzzles[view.current]!;
  const [piece, setPiece] = useState(0);
  const [k, setK] = useState(0);
  const [at, setAt] = useState<Cell | null>(null);
  const stamp = `${view.turnsLeft}`;
  const [forStamp, setForStamp] = useState(stamp);
  if (forStamp !== stamp) {
    setForStamp(stamp);
    setAt(null);
    setK(0);
    setPiece(Math.max(0, p.placed.indexOf(null)));
  }
  const placed = p.placed[piece] !== null && p.placed[piece] !== undefined;
  const shape = p.pieces[piece] ? orient(p.pieces[piece], k) : [];
  const w = Math.max(0, ...shape.map((c) => c[0])) + 1;
  const h = Math.max(0, ...shape.map((c) => c[1])) + 1;
  const ghost: Cell[] =
    at && !placed
      ? shape.map(
          ([x, y]) =>
            [
              x + Math.max(0, Math.min(g - w, at[0] - Math.floor(w / 2))),
              y + Math.max(0, Math.min(g - h, at[1] - Math.floor(h / 2))),
            ] as Cell,
        )
      : [];
  const target = new Set(p.target.map(([x, y]) => `${x},${y}`));
  const taken = new Set(p.placed.flatMap((c) => c ?? []).map(([x, y]) => `${x},${y}`));
  const ok = ghost.length > 0 && ghost.every(([x, y]) => target.has(`${x},${y}`) && !taken.has(`${x},${y}`));
  const act = () => {
    if (!view.yourTurn) return;
    if (placed) move({ type: 'remove', piece });
    else if (ok) move({ type: 'place', piece, cells: ghost });
  };
  const turn = () => setK((v) => (((v % 4) + 1) % 4) + (v >= 4 ? 4 : 0));
  const flip = () => setK((v) => (v >= 4 ? v - 4 : v + 4));
  useKeys((key) => {
    const d = ({ ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] } as Record<string, Cell>)[key];
    if (/^[1-4]$/.test(key)) setPiece(Number(key) - 1);
    else if (key === 'r' || key === 'R') turn();
    else if (key === 'f' || key === 'F') flip();
    else if (key === 'p' || key === 'P') view.yourTurn && move({ type: 'pass' });
    else if (d) setAt((c) => [Math.max(0, Math.min(g - 1, (c?.[0] ?? 2) + d[0])), Math.max(0, Math.min(g - 1, (c?.[1] ?? 2) + d[1]))]);
    else if (key === 'Enter') act();
    else return false;
    return true;
  });
  const who = (s: number) => (s === you ? 'You' : nameOf(view.players[s]!));
  const C = 60;
  return (
    <Play
      top={
        <>
          <GameTop
            title="Cooperative Puzzle"
            step={view.yourTurn ? 'Your turn: quick!' : `${who(view.turn)}'s turn · ${view.turnsLeft} turns left`}
            score={view.score}
          />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.turnMs} />
        </>
      }
      stage={
        <div className="spw-stage">
          <Wheel view={view} />
          <svg className="drawing spw-board" viewBox={`-6 -6 ${g * C + 12} ${g * C + 12}`} role="img" aria-label="Current puzzle">
            <PuzzleCells p={p} grid={g} c={C} ghost={view.yourTurn ? ghost : []} ok={ok} onCell={(x, y) => setAt([x, y])} />
          </svg>
        </div>
      }
      controls={
        <>
          <p className="ask">
            {view.yourTurn ? 'Place one piece on the shape (or take one back).' : `${who(view.turn)} is on puzzle ${view.current + 1}…`}
          </p>
          <div className="spw-tray">
            {p.pieces.map((cells, i) => {
              const shown = i === piece && p.placed[i] === null ? orient(cells, k) : cells;
              const pw = Math.max(...shown.map((c) => c[0])) + 1;
              const ph = Math.max(...shown.map((c) => c[1])) + 1;
              const s = Math.max(pw, ph, 3);
              return (
                <button
                  // biome-ignore lint/suspicious/noArrayIndexKey: tray slots are positional
                  key={i}
                  type="button"
                  className={`spw-piece${i === piece ? ' on' : ''}${p.placed[i] ? ' used' : ''}`}
                  onClick={() => setPiece(i)}
                >
                  <svg viewBox={`${(pw - s) / 2} ${(ph - s) / 2} ${s} ${s}`} aria-hidden="true" className={`p${i}`}>
                    {shown.map(([x, y]) => (
                      <rect key={`${x},${y}`} x={x + 0.06} y={y + 0.06} width={0.88} height={0.88} rx={0.15} />
                    ))}
                  </svg>
                </button>
              );
            })}
          </div>
          <div className="row">
            <button type="button" className="btn" disabled={placed} onClick={turn}>
              ↻ Turn
            </button>
            <button type="button" className="btn" disabled={placed} onClick={flip}>
              ⇋ Flip
            </button>
          </div>
          <div className="row">
            <button type="button" className="btn ghost" disabled={!view.yourTurn} onClick={() => move({ type: 'pass' })}>
              Pass
            </button>
            <button type="button" className="btn primary" disabled={!view.yourTurn || (!placed && !ok)} onClick={act}>
              {placed ? 'Take back' : 'Place'}
            </button>
          </div>
          {view.last && (
            <p className="muted small center">
              {view.last.replace(/^(you|bot-\d+):/, (_, id: string) => `${id === 'you' ? 'You' : nameOf(id)}:`)}
            </p>
          )}
          <p className="muted small center">100 per finished shape; all six: +10 per turn left.</p>
          <KeyHint>Keys: 1–4 pick, R turns, F flips, arrows move, Enter places, P passes.</KeyHint>
        </>
      }
    />
  );
}
