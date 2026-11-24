// Tiger and Goat: the hunt game on two boards at once. Rules page: docs/games/tiger-and-goat.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
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

const glint = (x: number, y: number, r: number) => `M${x},${y - r} ${x + r},${y} ${x},${y + r} ${x - r},${y}Z`;

/**
 * A faceted (low-poly) tiger or goat head in the side's colour, lit from the top left (like the
 * animals of Animal Tile Auction): white and black overlays make the facets, `eye` shapes are cut
 * in the page's near-black. Drawn in a 40-unit box, scaled to a board point at (x, y).
 */
function Head({ tiger, x, y }: { tiger: boolean; x: number; y: number }) {
  return (
    <g transform={`translate(${x - 0.37} ${y - 0.37}) scale(0.0185)`}>
      <title>{tiger ? 'tiger' : 'goat'}</title>
      {tiger ? (
        <>
          <path d="M5,4 14,9 20,8 26,9 35,4 34,14 37,20 36,27 30,31 26,35 20,37 14,35 10,31 4,27 3,20 6,14Z" />
          <path fill="#fff" fillOpacity=".38" d="M14,9 20,8 20,17 9,15Z" />
          <path fill="#fff" fillOpacity=".2" d="M5,4 14,9 9,15 6,14Z M6,14 9,15 12,22 4,27 3,20Z" />
          <path fill="#fff" fillOpacity=".12" d="M20,8 26,9 31,15 20,17Z M9,15 20,17 20,22 14,24 12,22Z" />
          <path
            fill="#000"
            fillOpacity=".12"
            d="M35,4 26,9 31,15 34,14Z M31,15 20,17 20,22 26,24 28,22Z M4,27 12,22 14,24 14,30 20,34 20,37 14,35 10,31Z"
          />
          <path fill="#000" fillOpacity=".25" d="M34,14 31,15 28,22 36,27 37,20Z" />
          <path fill="#000" fillOpacity=".3" d="M36,27 28,22 26,24 26,30 20,34 20,37 26,35 30,31Z" />
          <path fill="#000" fillOpacity=".35" d="M7.5,7 12.5,9.6 9,12.6Z M32.5,7 27.5,9.6 31,12.6Z" />
          <path fill="#fff" fillOpacity=".5" d="M14,24 20,22 20,34 14,30Z" />
          <path fill="#fff" fillOpacity=".3" d="M26,24 20,22 20,34 26,30Z" />
          <path
            className="eye"
            d="M15.5,10.2 17.8,9.4 17.2,13.8Z M19,8.6 21,8.6 20,13.2Z M22.2,9.4 24.5,10.2 22.8,13.8Z M4,18.6 9.5,20.6 4,22Z M4.6,24.4 10.5,24.3 5.2,26.6Z M36,18.6 30.5,20.6 36,22Z M35.4,24.4 29.5,24.3 34.8,26.6Z M10.5,17.2 15.8,18.4 16.6,20.6 12,20.6Z M29.5,17.2 24.2,18.4 23.4,20.6 28,20.6Z M17.4,24.2 22.6,24.2 20,27.4Z M19.4,27 20.6,27 20.6,29.4 23.6,30.6 23.2,31.4 20,30.4 16.8,31.4 16.4,30.6 19.4,29.4Z"
          />
          <path fill="#fff" d={`${glint(13.4, 19.1, 0.9)} ${glint(26.4, 19.1, 0.9)} ${glint(18.9, 24.9, 0.6)}`} />
        </>
      ) : (
        <>
          <path d="M16.8,33.5 23.2,33.5 20,39.6Z M16,10 13,4 8,1 4.5,2.5 9,4.5 11.5,9 12.5,12Z M24,10 27,4 32,1 35.5,2.5 31,4.5 28.5,9 27.5,12Z" />
          <path fill="#fff" fillOpacity=".45" d="M13,4 8,1 4.5,2.5 9,4.5Z" />
          <path
            fill="#fff"
            fillOpacity=".25"
            d="M16,10 13,4 9,4.5 11.5,9 12.5,12Z M27,4 32,1 35.5,2.5 31,4.5Z M16.8,33.5 20,33.5 20,39.6Z"
          />
          <path fill="#000" fillOpacity=".2" d="M24,10 27,4 31,4.5 28.5,9 27.5,12Z M23.2,33.5 20,33.5 20,39.6Z" />
          <path d="M11,14 2,14.5 0.8,18 10.5,19Z M29,14 38,14.5 39.2,18 29.5,19Z M12,11 20,9.5 28,11 30,17 28,24 25,31 23.5,35 20,36.5 16.5,35 15,31 12,24 10,17Z" />
          <path fill="#fff" fillOpacity=".38" d="M12,11 20,9.5 20,19 10,17Z" />
          <path fill="#fff" fillOpacity=".3" d="M11,14 2,14.5 0.8,18Z" />
          <path fill="#fff" fillOpacity=".15" d="M20,9.5 28,11 30,17 20,19Z M10,17 20,19 16,26 12,24Z" />
          <path fill="#000" fillOpacity=".12" d="M29,14 38,14.5 39.2,18Z M12,24 16,26 16.5,35 15,31Z" />
          <path fill="#000" fillOpacity=".25" d="M30,17 20,19 24,26 28,24Z" />
          <path fill="#000" fillOpacity=".3" d="M29,14 39.2,18 29.5,19Z M28,24 24,26 23.5,35 25,31Z" />
          <path fill="#fff" fillOpacity=".45" d="M16,26 20,24.5 20,36.5 16.5,35Z" />
          <path fill="#fff" fillOpacity=".2" d="M24,26 20,24.5 20,36.5 23.5,35Z" />
          <path
            className="eye"
            d="M11.4,18 13.2,16.8 16,17 16.6,18.6 14.6,19.6 12.4,19.3Z M28.6,18 26.8,16.8 24,17 23.4,18.6 25.4,19.6 27.6,19.3Z M16.8,31 18.6,30.4 19,32.2Z M23.2,31 21.4,30.4 21,32.2Z"
          />
          <path fill="#fff" d={`${glint(13.6, 17.8, 0.8)} ${glint(25.6, 17.8, 0.8)} ${glint(8.5, 2.6, 0.8)}`} />
        </>
      )}
    </g>
  );
}

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
      top={<GameTop title="Tiger and Goat" step={`Step ${view.step} · ${them} ${view.theirScore}`} score={view.score} />}
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
                {c === 0 ? (
                  <circle cx={x} cy={y} r={0.08} className="stone" />
                ) : (
                  <g className="stone">
                    <Head tiger={c === 1} x={x} y={y} />
                  </g>
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
