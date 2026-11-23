// Tiger and Goat: the hunt game on two boards at once. Rules page: docs/games/tiger-and-goat.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './TigerAndGoat.css';

type Side = 'tiger' | 'goat';

interface BoardView {
  cells: number[];
  side: Side;
  toMove: Side;
  yourTurn: boolean;
  hand: number;
  captured: number;
  turns: number;
  maxTurns: number;
  ended: boolean;
  winner: Side | null;
  moves: [number, number][];
  last: [number, number] | null;
  points: number;
}

interface TigerView {
  boards: BoardView[];
  opponent: string;
  step: number;
  score: number;
  theirScore: number;
  timeLeftMs: number;
  stepMs: number;
  over: boolean;
}

const NAMES = ['A', 'B'];
const LINES: [number, number, number, number][] = [
  ...[0, 1, 2, 3, 4].flatMap((k): [number, number, number, number][] => [
    [0, k, 4, k],
    [k, 0, k, 4],
  ]),
  [0, 0, 4, 4],
  [4, 0, 0, 4],
  [2, 0, 4, 2],
  [4, 2, 2, 4],
  [2, 4, 0, 2],
  [0, 2, 2, 0],
];

export default function TigerAndGoat({ view: raw }: ScreenProps) {
  const view = raw as TigerView;
  const active = view.boards.findIndex((b) => b.yourTurn);
  const [picked, setPicked] = useState<number | null>(null);
  const [sel, setSel] = useState<number | null>(null);
  const shown = picked ?? Math.max(active, 0);
  const board = view.boards[shown] as BoardView;
  const them = nameOf(view.opponent);
  const placing = board.yourTurn && board.side === 'goat' && board.hand > 0;
  const movable = new Set(board.moves.filter(([f]) => f >= 0).map(([f]) => f));
  const targets = new Set(board.moves.filter(([f]) => (placing ? f === -1 : f === sel)).map(([, t]) => t));
  if (sel !== null && !movable.has(sel)) setSel(null);
  // Follow the board you have to move on, unless you chose to look at the other one.
  const [forStep, setForStep] = useState(view.step);
  if (forStep !== view.step) {
    setForStep(view.step);
    setPicked(null);
  }

  const tap = (i: number) => {
    if (placing && targets.has(i)) move({ type: 'place', board: shown, to: i });
    else if (movable.has(i)) setSel(sel === i ? null : i);
    else if (sel !== null && targets.has(i)) {
      move({ type: 'move', board: shown, from: sel, to: i });
      setSel(null);
    }
  };

  useKeys((key) => {
    if (key === 'Escape') setSel(null);
    else if (key === 'v' || key === 'V') setPicked(1 - shown);
    else return false;
    return true;
  });

  const status = (b: BoardView) =>
    b.ended
      ? b.winner
        ? `${b.winner === b.side ? 'You' : them} won (${b.winner}s)`
        : 'Ended: turn limit'
      : b.yourTurn
        ? `Your move (${b.side}s)`
        : b.toMove === b.side
          ? 'Waiting for the next step'
          : `${them} is moving`;

  return (
    <Play
      top={
        <>
          <GameTop title="Tiger and Goat" step={`Step ${view.step} · ${them} ${view.theirScore}`} score={view.score} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.stepMs} />
        </>
      }
      stage={
        <svg className="drawing tiger-and-goat-board" viewBox="-0.6 -0.6 5.2 5.2" role="img" aria-label={`Board ${NAMES[shown]}`}>
          {LINES.map(([x1, y1, x2, y2]) => (
            <line key={`${x1}${y1}${x2}${y2}`} x1={x1} y1={y1} x2={x2} y2={y2} className="tiger-and-goat-line" />
          ))}
          {board.cells.map((c, i) => {
            const x = i % 5;
            const y = Math.floor(i / 5);
            const can = targets.has(i) || (!placing && movable.has(i));
            // Your side's pieces take your seat colour, the opponent's theirs.
            const mine = c !== 0 && (c === 1) === (board.side === 'tiger');
            const cls = `tiger-and-goat-point${c === 1 ? ' tiger' : c === 2 ? ' goat' : ''}${mine ? ' mine' : ''}${can ? ' can' : ''}${
              sel === i ? ' sel' : ''
            }${board.last?.[1] === i ? ' last' : ''}`;
            return (
              // biome-ignore lint/a11y/useSemanticElements: an SVG point cannot be a <button>
              <g key={`${x},${y}`} className={cls} onClick={() => tap(i)} role="button" tabIndex={-1} onKeyDown={() => {}}>
                <circle cx={x} cy={y} r={0.42} className="hit" />
                <circle cx={x} cy={y} r={c === 0 ? 0.08 : 0.3} className="stone" />
                {c !== 0 && (
                  <text x={x} y={y}>
                    {c === 1 ? 'T' : 'G'}
                  </text>
                )}
              </g>
            );
          })}
        </svg>
      }
      controls={
        <>
          <div className="tiger-and-goat-tabs">
            {view.boards.map((b, i) => (
              <button
                key={NAMES[i]}
                type="button"
                className={`tiger-and-goat-tab${i === shown ? ' on' : ''}${b.yourTurn ? ' turn' : ''}`}
                onClick={() => setPicked(i)}
              >
                <strong>
                  Board {NAMES[i]} · you are {b.side}s
                </strong>
                <span>{status(b)}</span>
                <span className="muted small">
                  Captured {b.captured}/5 · goats in hand {b.hand} · turn {b.turns}/{b.maxTurns} · {b.points} pts
                </span>
              </button>
            ))}
          </div>
          <p className="muted small center">
            {board.side === 'tiger'
              ? 'Tigers: slide along a line, or jump over a goat to capture it. Five captures win.'
              : 'Goats: place all 20, then slide. Trap every tiger to win; never leave a goat open to a jump.'}
          </p>
          <KeyHint>Click a piece, then where it goes. V switches boards, Esc cancels.</KeyHint>
        </>
      }
    />
  );
}
