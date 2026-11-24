// Balance Mancala: a ring of pits, stones in three colours; your weakest colour counts.
// Rules page: docs/games/colour-mancala.md

import { type CSSProperties, useState } from 'react';
import './ColourMancala.css';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';

type Stones = [number, number, number];

interface LastMove {
  seat: number;
  hole: number;
  first: number;
  path: number[];
  captured: number;
  extra: boolean;
}

interface MancalaView {
  players: string[];
  you: number | null;
  pits: number;
  holes: Stones[];
  turn: number;
  yourTurn: boolean;
  turnsLeft: number;
  points: number[];
  last: LastMove | null;
  over: boolean;
}

const COLOURS = ['red', 'green', 'blue'];
const R = 150;
const GOLDEN = Math.PI * (3 - Math.sqrt(5));

const sum = (s: Stones) => s[0] + s[1] + s[2];

/** Stones as dots in a sunflower pattern, coloured in order. */
function Dots({ stones, cx, cy, r }: { stones: Stones; cx: number; cy: number; r: number }) {
  const n = sum(stones);
  const spread = Math.min(0.3, 0.8 / Math.sqrt(Math.max(n, 1))) * r;
  const dot = Math.min(r * 0.17, spread * 0.55);
  const dots = stones
    .flatMap((count, colour) => Array.from({ length: count }, () => colour))
    .map((colour, k) => {
      const d = spread * Math.sqrt(k + 0.5);
      return { id: `d${k}`, colour, x: cx + d * Math.cos(k * GOLDEN), y: cy + d * Math.sin(k * GOLDEN) };
    });
  return (
    <>
      {dots.map((s) => (
        <circle key={s.id} className={`cm-stone ${COLOURS[s.colour] ?? ''}`} cx={s.x} cy={s.y} r={dot} />
      ))}
    </>
  );
}

function Ring({ view }: { view: MancalaView }) {
  const w = view.pits + 1;
  const count = view.holes.length;
  const step = (2 * Math.PI) / count;
  const you = view.you ?? 0;
  const spacing = R * step;
  const pr = Math.min(30, spacing * 0.42);
  const sr = Math.min(pr * 1.25, spacing * 0.55);
  const path = new Set(view.last?.path ?? []);
  return (
    <svg className="drawing cm-ring" viewBox="-200 -200 400 400" role="img" aria-label="Mancala ring">
      <circle className="cm-track" r={R} />
      {view.holes.map((stones, h) => {
        // Counter-clockwise; your row is at the bottom, your store at its right end.
        const a = Math.PI / 2 + (you * w + (w - 1) / 2 - h) * step;
        const x = R * Math.cos(a);
        const y = R * Math.sin(a);
        const seat = Math.floor(h / w);
        const store = h % w === view.pits;
        const cls = [
          'cm-hole',
          store ? 'store' : '',
          seat === you ? 'mine' : '',
          seat === view.turn && !view.over ? 'turn' : '',
          path.has(h) ? 'path' : '',
          view.last?.hole === h ? 'from' : '',
        ].join(' ');
        const lx = (R + (store ? sr : pr) + 14) * Math.cos(a);
        const ly = (R + (store ? sr : pr) + 14) * Math.sin(a);
        return (
          // biome-ignore lint/suspicious/noArrayIndexKey: holes are positional
          <g key={h} className={cls} style={{ '--cm-own': `var(--seat-${seat})` } as CSSProperties}>
            <circle className="cm-base" cx={x} cy={y} r={store ? sr : pr} />
            {store ? (
              <text x={x} y={y} className="cm-store-text">
                {stones.map((c, i) => (
                  <tspan key={COLOURS[i]} className={COLOURS[i]} x={x} dy={i === 0 ? -sr * 0.45 : sr * 0.45}>
                    {c}
                  </tspan>
                ))}
              </text>
            ) : (
              <Dots stones={stones} cx={x} cy={y} r={pr} />
            )}
            {!store && (
              <text x={lx} y={ly} className="cm-count">
                {sum(stones)}
              </text>
            )}
          </g>
        );
      })}
      <text className="cm-center" y={-8}>
        {view.over ? 'Game over' : view.yourTurn ? 'Your turn' : nameOf(view.players[view.turn]!)}
      </text>
      <text className="cm-center small" y={16}>
        {view.over ? '' : `${view.turnsLeft} turns left`}
      </text>
    </svg>
  );
}

export default function ColourMancala({ view: raw }: ScreenProps) {
  const view = raw as MancalaView;
  const [first, setFirst] = useState(0);
  const you = view.you ?? 0;
  const w = view.pits + 1;
  const myPits = Array.from({ length: view.pits }, (_, p) => view.holes[you * w + p]!);
  const sowPit = (pit: number) => view.yourTurn && sum(myPits[pit]!) > 0 && move({ type: 'sow', pit, first });
  useKeys((key) => {
    const k = key.toLowerCase();
    if (/^[1-9]$/.test(key) && Number(key) <= view.pits) sowPit(Number(key) - 1);
    else if (k === 'r' || k === 'g' || k === 'b') setFirst('rgb'.indexOf(k));
    else return false;
    return true;
  });
  const last = view.last;
  return (
    <Play
      top={
        <GameTop
          title="Balance Mancala"
          step={view.yourTurn ? 'Your turn' : `${nameOf(view.players[view.turn]!)} to move`}
          score={view.points[you] ?? 0}
        />
      }
      stage={<Ring view={view} />}
      controls={
        <>
          {last && (
            <p className="last">
              {last.seat === you ? 'You' : nameOf(view.players[last.seat]!)} sowed {last.path.length}
              {last.captured > 0 ? `, captured ${last.captured}` : ''}
              {last.extra ? ', extra turn' : ''}.
            </p>
          )}
          <div className="cm-first">
            <span className="muted small">Sow first:</span>
            {COLOURS.map((c, i) => (
              <button key={c} type="button" className={`btn small cm-colour ${c}${first === i ? ' on' : ''}`} onClick={() => setFirst(i)}>
                <span className={`cm-swatch ${c}`} /> {c}
              </button>
            ))}
          </div>
          <div className="cm-pits">
            {myPits.map((s, p) => (
              <button
                // biome-ignore lint/suspicious/noArrayIndexKey: pits are positional
                key={p}
                type="button"
                className="cm-pit"
                disabled={!view.yourTurn || sum(s) === 0}
                onClick={() => sowPit(p)}
              >
                <strong>{sum(s)}</strong>
                <span className="cm-mini">
                  {s.map((c, i) => (
                    <span key={COLOURS[i]} className={COLOURS[i]}>
                      {c}
                    </span>
                  ))}
                </span>
              </button>
            ))}
          </div>
          <ul className="list cm-scores">
            {view.players.map((p, k) => {
              const store = view.holes[k * w + view.pits]!;
              return (
                <li key={p} className={`${k === you ? 'me' : ''}${k === view.turn && !view.over ? ' cm-now' : ''}`}>
                  <span>{nameOf(p)}</span>
                  <span className="cm-mini">
                    {store.map((c, i) => (
                      <span key={COLOURS[i]} className={COLOURS[i]}>
                        {c}
                      </span>
                    ))}
                  </span>
                  <span className="score">{view.points[k]}</span>
                </li>
              );
            })}
          </ul>
          <p className="muted small center">Points = 50 × your weakest colour in the store + all its stones.</p>
          <KeyHint>Keys: 1–{view.pits} sow from a pit, R / G / B pick the first colour.</KeyHint>
        </>
      }
    />
  );
}
