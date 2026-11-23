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

/*
 * The pictures are cut like gems (docs/design.md, "Icons are faceted"): drawn around 0,0 in the
 * fill of their <g>, lit from the top left by white overlays, shaded down right by black ones,
 * with a tiny white glint.
 */

/** A five-pointed star, one ridge per point (outer radius 11.5). */
const STAR = (
  <>
    <polygon points="0,-11.5 3,-4.2 10.9,-3.6 4.9,1.6 6.8,9.3 0,5.2 -6.8,9.3 -4.9,1.6 -10.9,-3.6 -3,-4.2" />
    <path fill="#fff" fillOpacity=".45" d="M0,0 -3,-4.2 0,-11.5Z M0,0 -10.9,-3.6 -3,-4.2Z" />
    <path fill="#fff" fillOpacity=".35" d="M0,0 0,-11.5 3,-4.2Z M0,0 -4.9,1.6 -10.9,-3.6Z" />
    <path fill="#000" fillOpacity=".2" d="M0,0 10.9,-3.6 4.9,1.6Z M0,0 0,5.2 -6.8,9.3Z" />
    <path fill="#000" fillOpacity=".35" d="M0,0 4.9,1.6 6.8,9.3Z M0,0 6.8,9.3 0,5.2Z" />
    <path fill="#fff" d="M-1.6,-4.3 -0.7,-3.4 -1.6,-2.5 -2.5,-3.4Z" />
  </>
);

/** A circle, triangle, square or star of radius r. */
function Shape({ shape, x, y, r, fill }: { shape: number; x: number; y: number; r: number; fill: string }) {
  return (
    <g transform={`translate(${x} ${y}) scale(${r / 10})`} style={{ fill }}>
      {shape === 0 && (
        <>
          <circle r={10} />
          <path fill="#fff" fillOpacity=".5" d="M-5,-2.1 -9.2,-3.8A10,10 0 0 1 -3.8,-9.2L-2.1,-5Z" />
          <path fill="#fff" fillOpacity=".4" d="M-2.1,-5 -3.8,-9.2A10,10 0 0 1 3.8,-9.2L2.1,-5Z" />
          <path fill="#fff" fillOpacity=".3" d="M-5,2.1 -9.2,3.8A10,10 0 0 1 -9.2,-3.8L-5,-2.1Z" />
          <path fill="#000" fillOpacity=".25" d="M5,-2.1 9.2,-3.8A10,10 0 0 1 9.2,3.8L5,2.1Z" />
          <path fill="#000" fillOpacity=".3" d="M2.1,5 3.8,9.2A10,10 0 0 1 -3.8,9.2L-2.1,5Z" />
          <path fill="#000" fillOpacity=".35" d="M5,2.1 9.2,3.8A10,10 0 0 1 3.8,9.2L2.1,5Z" />
          <path fill="#fff" fillOpacity=".1" d="M5,2.1 2.1,5 -2.1,5 -5,2.1 -5,-2.1 -2.1,-5 2.1,-5 5,-2.1Z" />
          <path fill="#fff" d="M-2.6,-3.5 -1.7,-2.6 -2.6,-1.7 -3.5,-2.6Z" />
        </>
      )}
      {shape === 1 && (
        <>
          <polygon points="0,-10 10,8 -10,8" />
          <path fill="#fff" fillOpacity=".45" d="M-10,8 0,-10 0,-3 -4.2,4.5Z" />
          <path fill="#fff" fillOpacity=".12" d="M0,-3 4.2,4.5 -4.2,4.5Z" />
          <path fill="#000" fillOpacity=".3" d="M10,8 -10,8 -4.2,4.5 4.2,4.5Z" />
          <path fill="#fff" d="M-1.2,-1.3 -0.3,-0.4 -1.2,0.5 -2.1,-0.4Z" />
        </>
      )}
      {shape === 2 && (
        <>
          <rect x={-8.5} y={-8.5} width={17} height={17} />
          <path fill="#fff" fillOpacity=".4" d="M-8.5,-8.5 8.5,-8.5 4.3,-4.3 -4.3,-4.3Z" />
          <path fill="#fff" fillOpacity=".3" d="M-8.5,8.5 -8.5,-8.5 -4.3,-4.3 -4.3,4.3Z" />
          <path fill="#000" fillOpacity=".25" d="M8.5,-8.5 8.5,8.5 4.3,4.3 4.3,-4.3Z" />
          <path fill="#000" fillOpacity=".3" d="M8.5,8.5 -8.5,8.5 -4.3,4.3 4.3,4.3Z" />
          <path fill="#fff" d="M-2.4,-3.3 -1.5,-2.4 -2.4,-1.5 -3.3,-2.4Z" />
        </>
      )}
      {shape === 3 && STAR}
    </g>
  );
}

/** One pan of a balance, around x = 0. */
const PAN = (
  <>
    <path d="M-28,138 28,138 22,145 -22,145Z" />
    <path fill="#fff" fillOpacity=".45" d="M-28,138 28,138 26.6,140 -26.6,140Z" />
    <path fill="#fff" fillOpacity=".12" d="M-26.6,140 0,140 -4,145 -22,145Z" />
    <path fill="#000" fillOpacity=".3" d="M0,140 26.6,140 22,145 -4,145Z" />
  </>
);
/** The foot, the two pans and the pivot of a balance, around x = 0. */
const STAND = (
  <>
    <path d="M-26,195 -14,187 14,187 26,195Z" />
    <path fill="#fff" fillOpacity=".3" d="M-26,195 -14,187 2,187 -4,195Z" />
    <path fill="#000" fillOpacity=".25" d="M2,187 14,187 26,195 -4,195Z" />
    <g transform="translate(-45 0)">{PAN}</g>
    <g transform="translate(45 0)">{PAN}</g>
    <path d="M0,42 7,50 0,58 -7,50Z" />
    <path fill="#fff" fillOpacity=".45" d="M0,42 -7,50 0,50Z" />
    <path fill="#fff" fillOpacity=".12" d="M0,42 7,50 0,50Z" />
    <path fill="#000" fillOpacity=".12" d="M-7,50 0,58 0,50Z" />
    <path fill="#000" fillOpacity=".32" d="M7,50 0,58 0,50Z" />
    <path fill="#fff" d="M-2.5,45.2 -1.7,46 -2.5,46.8 -3.3,46Z" />
  </>
);

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
      <line x1={x - 45} y1={50} x2={x + 45} y2={50} className="rww-metal" />
      <path
        d={`M${x - 45} 50 L${x - 71} 140 M${x - 45} 50 L${x - 19} 140 M${x + 45} 50 L${x + 19} 140 M${x + 45} 50 L${x + 71} 140`}
        className="rww-strings"
      />
      <g transform={`translate(${x} 0)`} style={{ fill: WEIGHT }}>
        {STAND}
      </g>
      {pan(left, x - 45)}
      {pan(right, x + 45)}
    </g>
  );
}

/** A small faceted padlock, open (the shackle's right leg out of the body) or shut. */
function Lock({ open }: { open: boolean }) {
  const leg = open ? '19.5,9 16.5,9' : '19.5,13 16.5,13';
  return (
    <svg viewBox="0 0 24 28" className={`rww-lock${open ? ' open' : ''}`} aria-hidden="true">
      <path
        className="rww-shackle"
        d={`M4.5,13 4.5,7.5 6.5,3.5 9.5,1.5 14.5,1.5 17.5,3.5 19.5,7.5 ${leg} 16.5,8 15.3,5.4 13.6,4.4 10.4,4.4 8.7,5.4 7.5,8 7.5,13Z`}
      />
      <path fill="#fff" fillOpacity=".4" d="M4.5,13 4.5,7.5 6.5,3.5 9.5,1.5 10.4,4.4 8.7,5.4 7.5,8 7.5,13Z" />
      <path fill="#fff" fillOpacity=".15" d="M9.5,1.5 14.5,1.5 13.6,4.4 10.4,4.4Z" />
      <path fill="#000" fillOpacity=".25" d={`M14.5,1.5 17.5,3.5 19.5,7.5 ${leg} 16.5,8 15.3,5.4 13.6,4.4Z`} />
      <path className="rww-body" d="M4.5,12 19.5,12 21,13.5 21,25 19.5,26.5 4.5,26.5 3,25 3,13.5Z" />
      <path fill="#fff" fillOpacity=".35" d="M3,13.5 4.5,12 19.5,12 21,13.5 17.5,15 6.5,15Z" />
      <path fill="#fff" fillOpacity=".18" d="M3,13.5 6.5,15 6.5,23.5 3,25Z" />
      <path fill="#000" fillOpacity=".22" d="M21,13.5 21,25 17.5,23.5 17.5,15Z" />
      <path fill="#000" fillOpacity=".35" d="M3,25 6.5,23.5 17.5,23.5 21,25 19.5,26.5 4.5,26.5Z" />
      <path className="eye" d="M12,16.3 13.7,17.3 13.7,19.2 12.8,19.8 13.2,21.8 10.8,21.8 11.2,19.8 10.3,19.2 10.3,17.3Z" />
      <path fill="#fff" d="M5.6,12.7 6.3,13.4 5.6,14.1 4.9,13.4Z" />
    </svg>
  );
}

/** A star, crescent moon or water drop, cut like the stones. */
function SymbolGlyph({ id, x, y }: { id: number; x: number; y: number }) {
  return (
    <g transform={`translate(${x} ${y})`} className="rww-sym">
      {id === 0 && <g transform="scale(1.04)">{STAR}</g>}
      {id === 1 && (
        <>
          <path d="M5,-11 -2.8,-11.6 -9.3,-7.4 -11.8,0 -9.3,7.4 -2.8,11.6 5,11 2.1,7.9 0.3,4.2 -0.3,0 0.3,-4.2 2.1,-7.9Z" />
          <path fill="#fff" fillOpacity=".5" d="M-2.8,-11.6 -9.3,-7.4 -6.1,-6.3 -1.1,-10.4Z" />
          <path fill="#fff" fillOpacity=".4" d="M5,-11 -2.8,-11.6 -1.1,-10.4Z M-9.3,-7.4 -11.8,0 -8,0 -6.1,-6.3Z" />
          <path fill="#fff" fillOpacity=".15" d="M-11.8,0 -9.3,7.4 -6.1,6.3 -8,0Z" />
          <path fill="#000" fillOpacity=".15" d="M-9.3,7.4 -2.8,11.6 -1.1,10.4 -6.1,6.3Z M-6.1,6.3 -1.1,10.4 2.1,7.9 0.3,4.2Z" />
          <path fill="#000" fillOpacity=".2" d="M-8,0 -6.1,6.3 0.3,4.2 -0.3,0Z" />
          <path fill="#000" fillOpacity=".25" d="M-6.1,-6.3 -8,0 -0.3,0 0.3,-4.2Z" />
          <path fill="#000" fillOpacity=".3" d="M-1.1,-10.4 -6.1,-6.3 0.3,-4.2 2.1,-7.9Z M-2.8,11.6 5,11 -1.1,10.4Z" />
          <path fill="#000" fillOpacity=".35" d="M5,-11 -1.1,-10.4 2.1,-7.9Z" />
          <path fill="#fff" d="M-7.2,-6.5 -6.3,-5.6 -7.2,-4.7 -8.1,-5.6Z" />
        </>
      )}
      {id > 1 && (
        <>
          <path d="M0,-13 4.6,-5.5 7.4,0.5 7.4,6 4.3,9.9 0,11 -4.3,9.9 -7.4,6 -7.4,0.5 -4.6,-5.5Z" />
          <path fill="#fff" fillOpacity=".5" d="M-7.4,0.5 -4.6,-5.5 -1,3Z" />
          <path fill="#fff" fillOpacity=".45" d="M-4.6,-5.5 0,-13 -1,3Z" />
          <path fill="#fff" fillOpacity=".3" d="M0,-13 4.6,-5.5 -1,3Z M-7.4,6 -7.4,0.5 -1,3Z" />
          <path fill="#000" fillOpacity=".1" d="M-4.3,9.9 -7.4,6 -1,3Z" />
          <path fill="#000" fillOpacity=".25" d="M7.4,0.5 7.4,6 -1,3Z M0,11 -4.3,9.9 -1,3Z" />
          <path fill="#000" fillOpacity=".35" d="M7.4,6 4.3,9.9 -1,3Z M4.3,9.9 0,11 -1,3Z" />
          <path fill="#fff" d="M-3.4,-2.9 -2.5,-2 -3.4,-1.1 -4.3,-2Z" />
        </>
      )}
    </g>
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
