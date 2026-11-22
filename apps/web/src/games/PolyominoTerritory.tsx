// Polyomino Territory: grow your territory with polyomino pieces; key squares give encores and leaps.
// Rules page: docs/games/polyomino-territory.md

import { useState } from 'react';
import './PolyominoTerritory.css';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';

type Cell = [number, number];

interface Key {
  x: number;
  y: number;
  kind: 'encore' | 'leap';
}

interface Placement {
  seat: number;
  cells: Cell[];
  leap: boolean;
  keys: string[];
}

interface PolyView {
  players: string[];
  you: number | null;
  size: number;
  board: number[];
  keys: Key[];
  hand: Cell[][];
  leaps: number[];
  squares: number[];
  turn: number;
  yourTurn: boolean;
  encore: boolean;
  turnsLeft: number;
  timeLeftMs: number;
  turnMs: number;
  last: Placement | null;
  over: boolean;
}

const C = 30;
const DIRS: Cell[] = [
  [1, 0],
  [-1, 0],
  [0, 1],
  [0, -1],
];

/** The same orientation as the rules: turned k % 4 quarter turns, mirrored when k >= 4, shifted to 0. */
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

function MiniPiece({ cells }: { cells: Cell[] }) {
  const w = Math.max(...cells.map((c) => c[0])) + 1;
  const h = Math.max(...cells.map((c) => c[1])) + 1;
  const s = Math.max(w, h, 3);
  return (
    <svg className="pt-mini" viewBox={`${(w - s) / 2} ${(h - s) / 2} ${s} ${s}`} aria-hidden="true">
      {cells.map(([x, y]) => (
        <rect key={`${x},${y}`} x={x + 0.06} y={y + 0.06} width={0.88} height={0.88} rx={0.15} />
      ))}
    </svg>
  );
}

export default function PolyominoTerritory({ view: raw }: ScreenProps) {
  const view = raw as PolyView;
  const you = view.you ?? 0;
  const n = view.size;
  const [piece, setPiece] = useState(0);
  const [k, setK] = useState(0);
  const [at, setAt] = useState<Cell | null>(null);
  const [leap, setLeap] = useState(false);
  const lastStamp = view.last ? `${view.last.seat}:${view.last.cells.join(';')}` : '';
  const [forLast, setForLast] = useState(lastStamp);
  if (forLast !== lastStamp) {
    setForLast(lastStamp);
    if (view.last?.seat === you) {
      // Our piece went down: start fresh for the next one.
      setAt(null);
      setLeap(false);
    }
  }
  const shape = view.hand[piece] ? orient(view.hand[piece], k) : [];
  const w = Math.max(0, ...shape.map((c) => c[0])) + 1;
  const h = Math.max(0, ...shape.map((c) => c[1])) + 1;
  const ghost: Cell[] = at
    ? (() => {
        const ox = Math.max(0, Math.min(n - w, at[0] - Math.floor(w / 2)));
        const oy = Math.max(0, Math.min(n - h, at[1] - Math.floor(h / 2)));
        return shape.map(([x, y]) => [x + ox, y + oy] as Cell);
      })()
    : [];
  const owner = (x: number, y: number) => (x >= 0 && y >= 0 && x < n && y < n ? (view.board[y * n + x] ?? -2) : -2);
  const free = ghost.every(([x, y]) => owner(x, y) === -1);
  const touching = ghost.some(([x, y]) => DIRS.some(([dx, dy]) => owner(x + dx, y + dy) === you));
  const useLeap = leap && (view.leaps[you] ?? 0) > 0;
  const ok = ghost.length > 0 && free && (touching || useLeap);
  const place = () => view.yourTurn && ok && move({ type: 'place', piece, cells: ghost, leap: useLeap && !touching });
  const turn = () => setK((v) => (((v % 4) + 1) % 4) + (v >= 4 ? 4 : 0));
  const flip = () => setK((v) => (v >= 4 ? v - 4 : v + 4));
  useKeys((key) => {
    const d = ({ ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] } as Record<string, Cell>)[key];
    if (/^[1-3]$/.test(key)) setPiece(Number(key) - 1);
    else if (key === 'r' || key === 'R') turn();
    else if (key === 'f' || key === 'F') flip();
    else if (key === 'l' || key === 'L') setLeap((v) => !v);
    else if (d) setAt((c) => [Math.max(0, Math.min(n - 1, (c?.[0] ?? 6) + d[0])), Math.max(0, Math.min(n - 1, (c?.[1] ?? 6) + d[1]))]);
    else if (key === 'Enter') place();
    else return false;
    return true;
  });
  const lastCells = new Set(view.last?.cells.map(([x, y]) => `${x},${y}`));
  const cells = Array.from({ length: n * n }, (_, i) => ({ x: i % n, y: Math.floor(i / n), o: view.board[i] ?? -1 }));
  const who = (s: number) => (s === you ? 'You' : nameOf(view.players[s]!));
  return (
    <Play
      top={
        <>
          <GameTop
            title="Polyomino Territory"
            step={view.yourTurn ? (view.encore ? 'Encore: place again!' : 'Your turn') : `${who(view.turn)} to move`}
            score={(view.squares[you] ?? 0) * 10}
          />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.turnMs} />
        </>
      }
      stage={
        <svg className="drawing pt-board" viewBox={`-2 -2 ${n * C + 4} ${n * C + 4}`} role="img" aria-label="Territory board">
          {cells.map((c) => (
            // biome-ignore lint/a11y/useSemanticElements: an SVG cell cannot be a <button>
            <rect
              key={`${c.x},${c.y}`}
              className={`pt-cell s${c.o}${lastCells.has(`${c.x},${c.y}`) ? ' last' : ''}`}
              x={c.x * C + 1}
              y={c.y * C + 1}
              width={C - 2}
              height={C - 2}
              rx={4}
              role="button"
              tabIndex={-1}
              onClick={() => setAt([c.x, c.y])}
              onKeyDown={() => {}}
            />
          ))}
          {view.keys.map((key) => (
            <text key={`${key.x},${key.y}`} className={`pt-key ${key.kind}`} x={key.x * C + C / 2} y={key.y * C + C / 2}>
              {key.kind === 'encore' ? '⟳' : '↗'}
            </text>
          ))}
          {view.yourTurn &&
            ghost.map(([x, y]) => (
              <rect
                key={`g${x},${y}`}
                className={`pt-ghost${ok ? '' : ' bad'}`}
                x={x * C + 3}
                y={y * C + 3}
                width={C - 6}
                height={C - 6}
                rx={4}
              />
            ))}
        </svg>
      }
      controls={
        <>
          <p className="ask">
            {view.yourTurn ? 'Pick a piece, tap the board to put it there, then Place.' : `${who(view.turn)} is placing…`}
          </p>
          <div className="pt-hand">
            {view.hand.map((p, i) => (
              <button
                // biome-ignore lint/suspicious/noArrayIndexKey: hand slots are positional
                key={i}
                type="button"
                className={`pt-piece${i === piece ? ' on' : ''}`}
                onClick={() => setPiece(i)}
              >
                <MiniPiece cells={i === piece ? orient(p, k) : p} />
              </button>
            ))}
          </div>
          <div className="row">
            <button type="button" className="btn" onClick={turn}>
              ↻ Turn
            </button>
            <button type="button" className="btn" onClick={flip}>
              ⇋ Flip
            </button>
            {(view.leaps[you] ?? 0) > 0 && (
              <button type="button" className={`btn${leap ? ' pt-leap-on' : ''}`} onClick={() => setLeap((v) => !v)}>
                ↗ Leap ({view.leaps[you]})
              </button>
            )}
          </div>
          <button type="button" className="btn primary" disabled={!view.yourTurn || !ok} onClick={place}>
            {at && !ok && view.yourTurn ? (free ? 'Must touch your squares' : 'Squares taken') : 'Place'}
          </button>
          <ul className="list">
            {view.players.map((p, s) => (
              <li key={p} className={`${s === you ? 'me' : ''}${s === view.turn && !view.over ? ' pt-now' : ''}`}>
                <span>
                  <span className={`pt-swatch s${s}`} /> {who(s)}
                  {(view.leaps[s] ?? 0) > 0 && <span className="muted small"> · ↗{view.leaps[s]}</span>}
                </span>
                <span className="score">{view.squares[s]} sq</span>
              </li>
            ))}
          </ul>
          <p className="muted small center">⟳ encore: place again · ↗ leap: one later piece anywhere. 10 points per square.</p>
          <KeyHint>Keys: 1–3 pick, R turns, F flips, arrows move, L leap, Enter places.</KeyHint>
        </>
      }
    />
  );
}
