// Rising Water Well: four picture riddles before the well fills. Rules page: docs/games/rising-water-well.md

import type { ReactNode } from 'react';
import { GameTop, move, NumberPad, Play, type ScreenProps, TimerBar } from './common';
import './RisingWaterWell.css';

interface Stone {
  shape: number;
  colour: number;
  x: number;
  y: number;
}

type Riddle =
  | { kind: 'stones'; stones: Stone[]; shape: number; colour: number | null }
  | { kind: 'scratches'; groups: number[] }
  | { kind: 'scales'; circles: number; extra: number }
  | { kind: 'symbols'; sums: { symbols: number[]; total: number | null }[] }
  | { kind: 'reflection'; hourAngle: number; minuteAngle: number };

interface WellView {
  riddle: Riddle | null;
  index: number;
  riddles: number;
  solved: number;
  wrongTries: number;
  lastWrong: number | null;
  score: number;
  timeLeftMs: number;
  totalMs: number;
  escaped: boolean;
  over: boolean;
}

const W = 300;
const H = 210;
const COLOURS = ['var(--g-red)', 'var(--g-blue)', 'var(--g-yellow)'];
const WEIGHT = 'var(--silver)';
const COLOUR_NAMES = ['red', 'blue', 'gold'];
const SHAPE_NAMES = ['circle', 'triangle', 'square', 'star'];

function star(cx: number, cy: number, r: number): string {
  return Array.from({ length: 10 }, (_, i) => {
    const a = (Math.PI / 5) * i - Math.PI / 2;
    const rr = i % 2 ? r * 0.45 : r;
    return `${cx + rr * Math.cos(a)},${cy + rr * Math.sin(a)}`;
  }).join(' ');
}

/** A circle, triangle, square or star of radius r. */
function Shape({ shape, x, y, r, fill }: { shape: number; x: number; y: number; r: number; fill: string }) {
  if (shape === 0) return <circle cx={x} cy={y} r={r} style={{ fill }} />;
  if (shape === 1) return <polygon points={`${x},${y - r} ${x + r},${y + r * 0.8} ${x - r},${y + r * 0.8}`} style={{ fill }} />;
  if (shape === 2) return <rect x={x - r * 0.85} y={y - r * 0.85} width={r * 1.7} height={r * 1.7} style={{ fill }} />;
  return <polygon points={star(x, y, r * 1.15)} style={{ fill }} />;
}

function Tally({ count, x, y }: { count: number; x: number; y: number }) {
  const lines: ReactNode[] = [];
  for (let g = 0; g * 5 < count; g++) {
    const gx = x + (g % 2) * 24;
    const gy = y + Math.floor(g / 2) * 26;
    const inGroup = Math.min(5, count - g * 5);
    for (let i = 0; i < Math.min(4, inGroup); i++) {
      lines.push(<line key={`${g}-${i}`} x1={gx + i * 5} y1={gy} x2={gx + i * 5} y2={gy + 20} />);
    }
    if (inGroup === 5) lines.push(<line key={`${g}-x`} x1={gx - 3} y1={gy + 16} x2={gx + 18} y2={gy + 4} />);
  }
  return <g className="rww-tally">{lines}</g>;
}

/** A balance with `left` and `right` items (shape numbers), centred at x. */
function Balance({ x, left, right }: { x: number; left: number[]; right: number[] }) {
  const pan = (items: number[], px: number) =>
    items.map((s, i) => (
      // biome-ignore lint/suspicious/noArrayIndexKey: items are positional
      <Shape key={i} shape={s} x={px - (items.length - 1) * 7.5 + i * 15} y={s === 0 ? 132 : 129} r={s === 0 ? 7 : 9.5} fill={WEIGHT} />
    ));
  return (
    <g>
      <line x1={x} y1={50} x2={x} y2={190} className="rww-metal" />
      <path d={`M${x - 25} 192 H${x + 25}`} className="rww-metal" />
      <line x1={x - 45} y1={50} x2={x + 45} y2={50} className="rww-metal" />
      <path
        d={`M${x - 45} 50 L${x - 71} 140 M${x - 45} 50 L${x - 19} 140 M${x + 45} 50 L${x + 19} 140 M${x + 45} 50 L${x + 71} 140`}
        className="rww-strings"
      />
      <path d={`M${x - 73} 140 H${x - 17} M${x + 17} 140 H${x + 73}`} className="rww-metal" />
      {pan(left, x - 45)}
      {pan(right, x + 45)}
    </g>
  );
}

/** A small padlock, open or shut. */
function Lock({ open }: { open: boolean }) {
  return (
    <svg viewBox="0 0 24 28" className={`rww-lock${open ? ' open' : ''}`} aria-hidden="true">
      <path d={open ? 'M7 12 V8 a5 5 0 0 1 10 0 V5' : 'M7 12 V8 a5 5 0 0 1 10 0 V12'} className="rww-shackle" />
      <rect x={3} y={12} width={18} height={14} rx={3} className="rww-body" />
    </svg>
  );
}

function SymbolGlyph({ id, x, y }: { id: number; x: number; y: number }) {
  if (id === 0) return <polygon points={star(x, y, 12)} className="rww-sym" />;
  if (id === 1)
    return <path d={`M${x + 5} ${y - 11} A12 12 0 1 0 ${x + 5} ${y + 11} A14 14 0 0 1 ${x + 5} ${y - 11} Z`} className="rww-sym" />;
  return (
    <path
      d={`M${x} ${y - 13} C${x + 10} ${y - 1} ${x + 10} ${y + 11} ${x} ${y + 11} C${x - 10} ${y + 11} ${x - 10} ${y - 1} ${x} ${y - 13} Z`}
      className="rww-sym"
    />
  );
}

function hand(angle: number, length: number): [number, number] {
  const a = ((angle - 90) * Math.PI) / 180;
  return [150 + length * Math.cos(a), 115 + length * Math.sin(a)];
}

function Drawing({ riddle }: { riddle: Riddle }) {
  switch (riddle.kind) {
    case 'stones':
      return (
        <g>
          {riddle.stones.map((s) => (
            <Shape
              key={`${s.x},${s.y}`}
              shape={s.shape}
              x={12 + s.x * 2.76}
              y={8 + s.y * 2}
              r={11}
              fill={COLOURS[s.colour] ?? 'var(--muted)'}
            />
          ))}
        </g>
      );
    case 'scratches':
      return (
        <g>
          {[...riddle.groups, -1].map((count, i) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: panels are positional
            <g key={i}>
              <rect x={6 + i * 58} y={14} width={52} height={182} rx={6} className="rww-panel" />
              {count < 0 ? (
                <text x={32 + i * 58} y={112} className="rww-ask">
                  ?
                </text>
              ) : (
                <Tally count={count} x={13 + i * 58} y={24} />
              )}
            </g>
          ))}
        </g>
      );
    case 'scales':
      return (
        <g>
          <Balance x={75} left={[2]} right={Array(riddle.circles).fill(0)} />
          <Balance x={225} left={[1]} right={[2, ...Array(riddle.extra).fill(0)]} />
        </g>
      );
    case 'symbols':
      return (
        <g>
          {riddle.sums.map((sum, row) => {
            const y = 30 + row * 50;
            return (
              // biome-ignore lint/suspicious/noArrayIndexKey: rows are positional
              <g key={row}>
                {sum.symbols.map((id, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: symbols are positional
                  <g key={i}>
                    {i > 0 && (
                      <text x={32 + i * 50 - 25} y={y} className="rww-op">
                        +
                      </text>
                    )}
                    <SymbolGlyph id={id} x={32 + i * 50} y={y} />
                  </g>
                ))}
                <text x={190} y={y} className="rww-op">
                  =
                </text>
                <text x={240} y={y} className={sum.total === null ? 'rww-ask' : 'rww-op'}>
                  {sum.total ?? '?'}
                </text>
              </g>
            );
          })}
        </g>
      );
    case 'reflection': {
      const [hx, hy] = hand(riddle.hourAngle, 45);
      const [mx, my] = hand(riddle.minuteAngle, 70);
      const [kx, ky] = hand(180, 82);
      return (
        <g>
          <path d="M0 16 Q 37 10 75 16 T 150 16 T 225 16 T 300 16" className="rww-ripple" />
          <circle cx={150} cy={115} r={90} className="rww-dial" />
          {Array.from({ length: 12 }, (_, i) => {
            const [x1, y1] = hand(i * 30, 80);
            const [x2, y2] = hand(i * 30, 88);
            // biome-ignore lint/suspicious/noArrayIndexKey: ticks are positional
            return <line key={i} x1={x1} y1={y1} x2={x2} y2={y2} className="rww-tick" />;
          })}
          {/* The 12 mark, seen upside down at the bottom. */}
          <polygon points={`${kx},${ky - 8} ${kx - 7},${ky + 4} ${kx + 7},${ky + 4}`} className="rww-twelve" />
          <line x1={150} y1={115} x2={mx} y2={my} className="rww-hand minute" />
          <line x1={150} y1={115} x2={hx} y2={hy} className="rww-hand" />
          <circle cx={150} cy={115} r={5} className="rww-twelve" />
        </g>
      );
    }
  }
}

function question(riddle: Riddle): string {
  switch (riddle.kind) {
    case 'stones':
      return `How many ${riddle.colour === null ? '' : `${COLOUR_NAMES[riddle.colour]} `}${SHAPE_NAMES[riddle.shape]}s${riddle.colour === null ? ' of any colour' : ''}?`;
    case 'scratches':
      return 'The scratches follow a rule. How many marks come next?';
    case 'scales':
      return 'Both scales balance. How many circles weigh as much as one triangle?';
    case 'symbols':
      return 'Each symbol stands for a number. What is the last sum?';
    case 'reflection':
      return 'This clock is seen in the water below it. What hour does the real clock show?';
  }
}

export default function RisingWaterWell({ view: raw }: ScreenProps) {
  const view = raw as WellView;
  const water = 1 - view.timeLeftMs / view.totalMs;
  const waterTop = 18 + (H + 4) * (1 - water);
  return (
    <Play
      top={
        <>
          <GameTop title="Rising Water Well" step={`Riddle ${view.index} of ${view.riddles}`} score={view.score} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.totalMs} />
        </>
      }
      stage={
        <svg className="drawing rww" viewBox={`-10 -10 ${W + 20} ${H + 30}`} role="img" aria-label="The riddle on the well wall">
          <defs>
            <pattern id="rww-bricks" width="40" height="24" patternUnits="userSpaceOnUse">
              <rect width="40" height="24" className="rww-wall" />
              <path d="M0 0.5 H40 M0 12.5 H40 M20 0 V12 M0.5 12 V24" className="rww-mortar" />
            </pattern>
          </defs>
          <rect x={-10} y={-10} width={W + 20} height={H + 30} fill="url(#rww-bricks)" />
          <rect x={0} y={0} width={W} height={H} rx={10} className="rww-tablet" />
          {view.riddle && <Drawing riddle={view.riddle} />}
          <rect x={-10} y={waterTop} width={W + 20} height={H + 30} className="rww-water" />
          <path d={`M-10 ${waterTop} q 20 -6 40 0 t 40 0 t 40 0 t 40 0 t 40 0 t 40 0 t 40 0 t 40 0 t 40 0`} className="rww-wave" />
        </svg>
      }
      controls={
        <>
          <div className="rww-locks" role="img" aria-label={`${view.solved} of ${view.riddles} locks open`}>
            {Array.from({ length: view.riddles }, (_, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: locks are positional
              <Lock key={i} open={i < view.solved} />
            ))}
          </div>
          {view.riddle ? (
            <>
              <p className="ask">{question(view.riddle)}</p>
              {view.lastWrong !== null && <p className="last bad">{view.lastWrong} is wrong: the water rises!</p>}
              <NumberPad round={view.index * 1000 + view.wrongTries} onSubmit={(value) => move({ type: 'answer', value })} />
              <button type="button" className="btn ghost" onClick={() => move({ type: 'giveUp' })}>
                Give up
              </button>
            </>
          ) : (
            <p className="ask">{view.escaped ? 'The grate opens: you climb out!' : 'The well is full.'}</p>
          )}
        </>
      }
    />
  );
}
