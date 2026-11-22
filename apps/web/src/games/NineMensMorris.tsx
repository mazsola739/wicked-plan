// Nine Men's Morris: place, slide and fly; mills remove pieces. Rules page: docs/games/nine-mens-morris.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './NineMensMorris.css';

interface MorrisView {
  board: number[];
  hand: number;
  theirHand: number;
  onBoard: number;
  theirOnBoard: number;
  opponent: string;
  yourTurn: boolean;
  phase: 'place' | 'move' | 'fly' | 'remove' | 'wait' | 'over';
  moves: [number, number][];
  removable: number[];
  last: [number, number] | null;
  lastRemoved: number | null;
  turnNo: number;
  timeLeftMs: number;
  turnMs: number;
  over: boolean;
}

/** Point i: ring i/8 (outer first), position i%8 clockwise from the top-left corner. */
const POINTS: [number, number][] = Array.from({ length: 24 }, (_, i) => {
  const r = Math.floor(i / 8);
  const k = i % 8;
  const far = 6 - r;
  return [[r, 3, far, far, far, 3, r, r][k] ?? 0, [r, r, r, 3, far, far, far, 3][k] ?? 0];
});

const CROSS: [number, number, number, number][] = [
  [3, 0, 3, 2],
  [6, 3, 4, 3],
  [3, 6, 3, 4],
  [0, 3, 2, 3],
];

const HELP: Record<MorrisView['phase'], string> = {
  place: 'Place a piece on an empty point.',
  move: 'Pick a piece, then a neighbouring empty point.',
  fly: 'Three pieces left: a piece may fly to any empty point.',
  remove: 'Mill! Remove one of the glowing pieces.',
  wait: '',
  over: 'Game over.',
};

export default function NineMensMorris({ view: raw }: ScreenProps) {
  const view = raw as MorrisView;
  const [sel, setSel] = useState<number | null>(null);
  const them = nameOf(view.opponent);
  const movable = new Set(view.moves.filter(([f]) => f >= 0).map(([f]) => f));
  const targets = new Set(view.moves.filter(([f]) => (view.phase === 'place' ? f === -1 : f === sel)).map(([, t]) => t));
  const removable = new Set(view.removable);
  if (sel !== null && !movable.has(sel)) setSel(null);

  const tap = (i: number) => {
    if (view.phase === 'remove' && removable.has(i)) move({ type: 'remove', at: i });
    else if (view.phase === 'place' && targets.has(i)) move({ type: 'place', to: i });
    else if (movable.has(i)) setSel(sel === i ? null : i);
    else if (sel !== null && targets.has(i)) {
      move({ type: 'move', from: sel, to: i });
      setSel(null);
    }
  };

  useKeys((key) => {
    if (key !== 'Escape') return false;
    setSel(null);
    return true;
  });

  const status = view.yourTurn ? HELP[view.phase] : view.over ? HELP.over : `${them} is thinking…`;

  return (
    <Play
      top={
        <>
          <GameTop
            title="Nine Men's Morris"
            step={`Turn ${view.turnNo}${view.yourTurn ? ' · your move' : ''}`}
            score={view.onBoard + view.hand}
          />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.turnMs} />
        </>
      }
      stage={
        <svg className="drawing nine-mens-morris-board" viewBox="-0.6 -0.6 7.2 7.2" role="img" aria-label="Morris board">
          {[0, 1, 2].map((r) => (
            <rect key={r} x={r} y={r} width={6 - 2 * r} height={6 - 2 * r} className="nine-mens-morris-line" />
          ))}
          {CROSS.map(([x1, y1, x2, y2]) => (
            <line key={`${x1},${y1}`} x1={x1} y1={y1} x2={x2} y2={y2} className="nine-mens-morris-line" />
          ))}
          {POINTS.map(([x, y], i) => {
            const c = view.board[i];
            const cls = [
              'nine-mens-morris-point',
              c === 1 ? 'mine' : c === -1 ? 'theirs' : 'empty',
              sel === i ? 'sel' : '',
              targets.has(i) || removable.has(i) || (view.phase !== 'place' && movable.has(i)) ? 'can' : '',
              view.last?.[1] === i ? 'last' : '',
              view.lastRemoved === i ? 'gone' : '',
            ].join(' ');
            return (
              // biome-ignore lint/a11y/useSemanticElements: an SVG point cannot be a <button>
              <g key={`${x},${y}`} className={cls} onClick={() => tap(i)} role="button" tabIndex={-1} onKeyDown={() => {}}>
                <circle cx={x} cy={y} r={0.45} className="hit" />
                <circle cx={x} cy={y} r={c === 0 ? 0.12 : 0.32} className="stone" />
              </g>
            );
          })}
        </svg>
      }
      controls={
        <>
          <div className="nine-mens-morris-side">
            <span>You</span>
            <Pieces n={view.hand} cls="mine" />
            <span className="muted small">{view.onBoard} on board</span>
          </div>
          <div className="nine-mens-morris-side">
            <span>{them}</span>
            <Pieces n={view.theirHand} cls="theirs" />
            <span className="muted small">{view.theirOnBoard} on board</span>
          </div>
          <p className={`ask${view.phase === 'remove' ? ' nine-mens-morris-mill' : ''}`}>{status}</p>
          <KeyHint>Click a piece, then where it goes. Esc cancels.</KeyHint>
        </>
      }
    />
  );
}

function Pieces({ n, cls }: { n: number; cls: string }) {
  return (
    <span className="nine-mens-morris-hand">
      {Array.from({ length: n }, (_, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: identical pieces
        <i key={i} className={cls} />
      ))}
    </span>
  );
}
