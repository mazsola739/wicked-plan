// Lay the Grass: everyone lays the same piece each round. Rules page: docs/games/lay-the-grass.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './LayTheGrass.css';

type Cell = [number, number];

interface LayTheGrassView {
  size: number;
  cells: number[];
  seats: { id: string; patch: number; square: number; score: number }[];
  you: number;
  round: number;
  rounds: number;
  piece: Cell[] | null;
  next: Cell[] | null;
  turn: string | null;
  last: Cell[];
  over: boolean;
}

/** Grass in each seat's colour, by seat index (you first). */
const GRASS = ['var(--seat-0)', 'var(--seat-1)', 'var(--seat-2)', 'var(--seat-3)'];

function normalize(cells: Cell[]): Cell[] {
  const mx = Math.min(...cells.map((c) => c[0]));
  const my = Math.min(...cells.map((c) => c[1]));
  return cells.map(([x, y]) => [x - mx, y - my] as Cell).sort((a, b) => a[1] - b[1] || a[0] - b[0]);
}

/** The piece turned `rot` quarter turns, mirrored first when `flip`. */
function orient(piece: Cell[], rot: number, flip: boolean): Cell[] {
  let cur = piece.map(([x, y]) => [flip ? -x : x, y] as Cell);
  for (let i = 0; i < rot; i++) cur = cur.map(([x, y]) => [-y, x] as Cell);
  return normalize(cur);
}

function Piece({ cells, color }: { cells: Cell[]; color: string }) {
  const w = Math.max(...cells.map((c) => c[0])) + 1;
  const h = Math.max(...cells.map((c) => c[1])) + 1;
  return (
    <svg
      className="lg-piece"
      style={{ width: `${w * 1.8}em`, height: `${h * 1.8}em` }}
      viewBox={`-0.05 -0.05 ${w + 0.1} ${h + 0.1}`}
      role="img"
      aria-label="Grass piece"
    >
      {cells.map(([x, y]) => (
        <rect key={`${x},${y}`} x={x + 0.05} y={y + 0.05} width="0.9" height="0.9" rx="0.15" style={{ fill: color }} />
      ))}
    </svg>
  );
}

export default function LayTheGrass({ view: raw }: ScreenProps) {
  const view = raw as LayTheGrassView;
  const n = view.size;
  const [rot, setRot] = useState(0);
  const [flip, setFlip] = useState(false);
  const [at, setAt] = useState<Cell | null>(null);
  const turnKey = `${view.round}/${view.turn}`;
  const [forTurn, setForTurn] = useState(turnKey);
  if (forTurn !== turnKey) {
    setForTurn(turnKey);
    setAt(null);
    setRot(0);
    setFlip(false);
  }
  const mine = !view.over && view.turn === view.seats[view.you]?.id;
  const shape = view.piece ? orient(view.piece, rot, flip) : [];
  const anchor = shape[0] ?? [0, 0];
  const ghost: Cell[] = at ? shape.map(([x, y]) => [x - anchor[0] + at[0], y - anchor[1] + at[1]]) : [];
  const fits = ghost.length > 0 && ghost.every(([x, y]) => x >= 0 && y >= 0 && x < n && y < n && view.cells[y * n + x] === -1);
  const ghostSet = new Set(ghost.map(([x, y]) => y * n + x));
  const lastSet = new Set(view.last.map(([x, y]) => y * n + x));
  const place = () => mine && fits && move({ type: 'place', cells: ghost });
  const tap = (x: number, y: number) => {
    if (!mine) return;
    if (at && at[0] === x && at[1] === y) place();
    else setAt([x, y]);
  };
  useKeys((key) => {
    const d = ({ ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] } as Record<string, Cell>)[key];
    if (d) setAt((a) => [Math.min(n - 1, Math.max(0, (a?.[0] ?? n >> 1) + d[0])), Math.min(n - 1, Math.max(0, (a?.[1] ?? n >> 1) + d[1]))]);
    else if (key === 'r' || key === 'R') setRot((r) => (r + 1) % 4);
    else if (key === 'f' || key === 'F') setFlip((f) => !f);
    else if (key === 'Enter' || key === ' ') place();
    else return false;
    return true;
  });
  const me = view.seats[view.you];
  return (
    <Play
      top={<GameTop title="Lay the Grass" step={`Round ${view.round} of ${view.rounds}`} score={me?.score ?? 0} />}
      stage={
        <div className="lg-field" style={{ gridTemplateColumns: `repeat(${n}, 1fr)`, gridTemplateRows: `repeat(${n}, 1fr)` }}>
          {view.cells.map((c, i) => {
            const x = i % n;
            const y = Math.floor(i / n);
            const g = ghostSet.has(i);
            return (
              <button
                key={`${x},${y}`}
                type="button"
                aria-label={`Square ${x + 1}, ${y + 1}`}
                className={`lg-sq${c === -2 ? ' rock' : ''}${g ? (fits ? ' ghost' : ' ghost bad') : ''}${lastSet.has(i) ? ' last' : ''}`}
                style={c >= 0 ? { background: `color-mix(in srgb, ${GRASS[c]} 80%, #15121f)`, borderColor: GRASS[c] } : undefined}
                disabled={!mine}
                onClick={() => tap(x, y)}
              />
            );
          })}
        </div>
      }
      controls={
        <>
          <p className="last">
            {view.over
              ? 'The field is full.'
              : mine
                ? 'Your turn: tap where the piece goes, tap again to lay it.'
                : `${nameOf(view.turn ?? '')} is laying grass…`}
          </p>
          {view.piece && (
            <div className="lg-pieces">
              <div className="lg-now">
                <Piece cells={shape} color={GRASS[view.you] ?? 'var(--seat-0)'} />
                <span className="muted small">this round</span>
              </div>
              {view.next && (
                <div className="lg-next">
                  <Piece cells={normalize(view.next)} color="var(--muted)" />
                  <span className="muted small">next</span>
                </div>
              )}
            </div>
          )}
          <div className="row">
            <button type="button" className="btn" disabled={!mine} onClick={() => setRot((r) => (r + 1) % 4)}>
              ↻ Turn
            </button>
            <button type="button" className="btn" disabled={!mine} onClick={() => setFlip((f) => !f)}>
              ⇋ Flip
            </button>
            <button type="button" className="btn primary" disabled={!mine || !fits} onClick={place}>
              Lay
            </button>
          </div>
          <ul className="list lg-scores">
            {view.seats.map((s, i) => (
              <li key={s.id} className={i === view.you ? 'me' : ''}>
                <span className="lg-who">
                  <span className="lg-swatch" style={{ background: GRASS[i] }} />
                  {nameOf(s.id)}
                  {view.turn === s.id && <span className="muted small"> · laying</span>}
                </span>
                <span className="muted small">
                  patch {s.patch} · square {s.square}×{s.square}
                </span>
                <span className="score">{s.score}</span>
              </li>
            ))}
          </ul>
          <p className="muted small center">Score: 10 per square of your largest patch + 20 × side² of your largest grass square.</p>
          <KeyHint>Keys: arrows move, R turns, F flips, Enter lays.</KeyHint>
        </>
      }
    />
  );
}
