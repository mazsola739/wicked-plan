// Count the Cubes, Count the Rectangles and Equation Pyramid: rounds of one question.

import { useState } from 'react';
import type { CubesPuzzle, EquationAnswer, EquationPuzzle, Op, QuizView, RectanglesPuzzle } from '../types';
import { GameTop, KeyHint, move, NumberPad, Play, RoundDots, TimerBar, useKeys } from './common';

function QuizFrame<P, A>({
  title,
  view,
  stage,
  controls,
}: {
  title: string;
  view: QuizView<P, A>;
  stage: React.ReactNode;
  controls: React.ReactNode;
}) {
  return (
    <Play
      top={
        <>
          <GameTop title={title} step={`Round ${view.round} of ${view.rounds}`} score={view.score} />
          <RoundDots results={view.history.map((h) => h.correct)} total={view.rounds} />
          {view.roundTimeMs > 0 && <TimerBar leftMs={view.timeLeftMs} totalMs={view.roundTimeMs} />}
        </>
      }
      stage={stage}
      controls={
        <>
          {controls}
          <LastRound history={view.history} />
        </>
      }
    />
  );
}

function LastRound<P, A>({ history }: { history: QuizView<P, A>['history'] }) {
  const last = history[history.length - 1];
  if (!last) return null;
  const show = (a: A | null) => (a === null ? 'no answer' : typeof a === 'number' ? String(a) : formatEquation(a as EquationAnswer));
  return (
    <p className={`last ${last.correct ? 'ok' : 'bad'}`}>
      Last round: {last.correct ? `right, +${last.points}` : `${show(last.given)} – the answer: ${show(last.solution)}`}
    </p>
  );
}

// --- Count the Cubes -------------------------------------------------------

const A = 26; // half tile width
const B = 13; // half tile height
const C = 30; // cube height

function iso(x: number, y: number, z: number): [number, number] {
  return [(x - y) * A, (x + y) * B - z * C];
}

function poly(points: [number, number, number][]): string {
  return points.map(([x, y, z]) => iso(x, y, z).join(',')).join(' ');
}

function CubesDrawing({ puzzle }: { puzzle: CubesPuzzle }) {
  const cubes: [number, number, number][] = [];
  for (let y = 0; y < puzzle.rows; y++) {
    for (let x = 0; x < puzzle.cols; x++) {
      const h = puzzle.heights[y * puzzle.cols + x] ?? 0;
      for (let z = 0; z < h; z++) cubes.push([x, y, z]);
    }
  }
  cubes.sort((p, q) => p[0] + p[1] - (q[0] + q[1]) || p[2] - q[2]);
  const maxH = Math.max(1, ...puzzle.heights);
  const left = -puzzle.rows * A - 4;
  const right = puzzle.cols * A + 4;
  const top = -maxH * C - 4;
  const bottom = (puzzle.cols + puzzle.rows) * B + 4;
  return (
    <svg className="drawing" viewBox={`${left} ${top} ${right - left} ${bottom - top}`} role="img" aria-label="Cubes">
      {cubes.map(([x, y, z]) => (
        <g key={`${x}-${y}-${z}`} className="cube">
          <polygon
            className="top"
            points={poly([
              [x, y, z + 1],
              [x + 1, y, z + 1],
              [x + 1, y + 1, z + 1],
              [x, y + 1, z + 1],
            ])}
          />
          <polygon
            className="right"
            points={poly([
              [x + 1, y, z],
              [x + 1, y + 1, z],
              [x + 1, y + 1, z + 1],
              [x + 1, y, z + 1],
            ])}
          />
          <polygon
            className="left"
            points={poly([
              [x, y + 1, z],
              [x + 1, y + 1, z],
              [x + 1, y + 1, z + 1],
              [x, y + 1, z + 1],
            ])}
          />
        </g>
      ))}
    </svg>
  );
}

export function CubesGame({ view }: { view: QuizView<CubesPuzzle, number> }) {
  return (
    <QuizFrame
      title="Count the Cubes"
      view={view}
      stage={view.puzzle && <CubesDrawing puzzle={view.puzzle} />}
      controls={
        <>
          <p className="ask">How many cubes are there?</p>
          <NumberPad round={view.round} onSubmit={(value) => move({ type: 'answer', value })} />
        </>
      }
    />
  );
}

// --- Count the Rectangles --------------------------------------------------

const U = 60; // lattice step

function RectanglesDrawing({ puzzle }: { puzzle: RectanglesPuzzle }) {
  const lines: [number, number, number, number][] = [];
  puzzle.h.forEach((row, y) => {
    row.forEach((on, x) => {
      if (on) lines.push([x, y, x + 1, y]);
    });
  });
  puzzle.v.forEach((row, y) => {
    row.forEach((on, x) => {
      if (on) lines.push([x, y, x, y + 1]);
    });
  });
  const pad = 6;
  return (
    <svg
      className="drawing"
      viewBox={`${-pad} ${-pad} ${puzzle.cols * U + 2 * pad} ${puzzle.rows * U + 2 * pad}`}
      role="img"
      aria-label="Figure"
    >
      {lines.map(([x1, y1, x2, y2]) => (
        <line key={`${x1}-${y1}-${x2}-${y2}`} className="segment" x1={x1 * U} y1={y1 * U} x2={x2 * U} y2={y2 * U} />
      ))}
    </svg>
  );
}

export function RectanglesGame({ view }: { view: QuizView<RectanglesPuzzle, number> }) {
  return (
    <QuizFrame
      title="Count the Rectangles"
      view={view}
      stage={view.puzzle && <RectanglesDrawing puzzle={view.puzzle} />}
      controls={
        <>
          <p className="ask">How many rectangles are there? (Squares count too.)</p>
          <NumberPad round={view.round} onSubmit={(value) => move({ type: 'answer', value })} />
        </>
      }
    />
  );
}

// --- Equation Pyramid -------------------------------------------------------

const OPS: Op[] = ['+', '-', '*'];
const OP_SIGN: Record<Op, string> = { '+': '+', '-': '−', '*': '×' };

function formatEquation(a: EquationAnswer, cards?: number[]): string {
  const n = (i: number) => (cards ? String(cards[i]) : String.fromCharCode(65 + i));
  return `(${n(a.picks[0])} ${OP_SIGN[a.ops[0]]} ${n(a.picks[1])}) ${OP_SIGN[a.ops[1]]} ${n(a.picks[2])}`;
}

export function EquationGame({ view }: { view: QuizView<EquationPuzzle, EquationAnswer> }) {
  const [picks, setPicks] = useState<number[]>([]);
  const [ops, setOps] = useState<[Op, Op]>(['+', '+']);
  const [forRound, setForRound] = useState(view.round);
  if (forRound !== view.round) {
    setForRound(view.round);
    setPicks([]);
    setOps(['+', '+']);
  }
  const p = view.puzzle;
  const toggle = (i: number) => setPicks((cur) => (cur.includes(i) ? cur.filter((c) => c !== i) : cur.length < 3 ? [...cur, i] : cur));
  const cycle = (k: 0 | 1) =>
    setOps((o) => (k === 0 ? [OPS[(OPS.indexOf(o[0]) + 1) % 3]!, o[1]] : [o[0], OPS[(OPS.indexOf(o[1]) + 1) % 3]!]));
  const slot = (k: number) => (p && picks[k] !== undefined ? p.cards[picks[k]!] : '?');
  const submit = () => picks.length === 3 && move({ type: 'answer', value: { picks, ops } });
  // Desktop: keys 1–9 and 0 pick the cards in order, + - * set the next operator, Backspace undoes, Enter submits.
  useKeys((key) => {
    const card = '1234567890'.indexOf(key);
    if (card >= 0) toggle(card);
    else if (key === '+' || key === '-' || key === '*') setOps((o) => (picks.length < 2 ? [key, o[1]] : [o[0], key]));
    else if (key === 'Backspace') setPicks((cur) => cur.slice(0, -1));
    else if (key === 'Enter') submit();
    else return false;
    return true;
  });
  return (
    <QuizFrame
      title="Equation Pyramid"
      view={view}
      stage={
        p && (
          <div className="equation">
            <div className="target">
              Target: <strong>{p.target}</strong>
            </div>
            <div className="expr">
              <span className="paren">(</span>
              <span className="slot">{slot(0)}</span>
              <button type="button" className="op" onClick={() => cycle(0)}>
                {OP_SIGN[ops[0]]}
              </button>
              <span className="slot">{slot(1)}</span>
              <span className="paren">)</span>
              <button type="button" className="op" onClick={() => cycle(1)}>
                {OP_SIGN[ops[1]]}
              </button>
              <span className="slot">{slot(2)}</span>
            </div>
          </div>
        )
      }
      controls={
        p && (
          <>
            <p className="muted small center">Tap three cards; tap an operator to change it.</p>
            <div className="cards">
              {p.cards.map((c, i) => (
                <button
                  // biome-ignore lint/suspicious/noArrayIndexKey: cards are positional
                  key={i}
                  type="button"
                  className={`card-btn${picks.includes(i) ? ' picked' : ''}`}
                  onClick={() => toggle(i)}
                >
                  {c}
                  {picks.includes(i) && <small>{picks.indexOf(i) + 1}</small>}
                </button>
              ))}
            </div>
            <button type="button" className="btn primary" disabled={picks.length !== 3} onClick={submit}>
              Submit
            </button>
            <KeyHint>Keys: 1–9, 0 pick the cards in order, + − * set the operators, Enter submits.</KeyHint>
          </>
        )
      }
    />
  );
}
