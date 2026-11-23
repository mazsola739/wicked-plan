// Find the Differences: tap what changed, faster than the rivals. Rules page: docs/games/find-the-differences.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './FindTheDifferences.css';

interface Shape {
  id: number;
  kind: number;
  colour: number;
  x: number;
  y: number;
  r: number;
}

interface Spot {
  x: number;
  y: number;
  r: number;
}

interface FindTheDifferencesView {
  round: number;
  rounds: number;
  timeLeftMs: number;
  roundMs: number;
  picture: { ground: number; left: Shape[]; right: Shape[] };
  diffs: number;
  found: Spot[];
  misses: number;
  all: Spot[] | null;
  seats: { id: string; found: number; points: number }[];
  you: number;
  revealing: boolean;
  revealLeftMs: number;
  over: boolean;
}

const COLOURS = [
  'var(--g-red)',
  'var(--g-orange)',
  'var(--g-yellow)',
  'var(--g-green)',
  'var(--g-blue)',
  'var(--g-purple)',
  'var(--g-pink)',
  'var(--g-white)',
];
/** Four grounds: the dark board, tinted a little. */
const GROUNDS = [
  'color-mix(in srgb, var(--violet) 16%, var(--board))',
  'color-mix(in srgb, var(--cyan) 10%, var(--board))',
  'color-mix(in srgb, var(--g-blue) 16%, var(--board))',
  'color-mix(in srgb, var(--g-pink) 12%, var(--board))',
];

function polygon(x: number, y: number, r: number, n: number, turn: number, inner?: number): string {
  const pts: string[] = [];
  const steps = inner ? n * 2 : n;
  for (let i = 0; i < steps; i++) {
    const a = turn + (i * 2 * Math.PI) / steps;
    const rr = inner && i % 2 ? r * inner : r;
    pts.push(`${(x + rr * Math.cos(a)).toFixed(1)},${(y + rr * Math.sin(a)).toFixed(1)}`);
  }
  return pts.join(' ');
}

function ShapeArt({ s }: { s: Shape }) {
  const fill = COLOURS[s.colour];
  const up = -Math.PI / 2;
  switch (s.kind) {
    case 0:
      return <circle cx={s.x} cy={s.y} r={s.r} style={{ fill }} />;
    case 1:
      return <rect x={s.x - s.r * 0.85} y={s.y - s.r * 0.85} width={s.r * 1.7} height={s.r * 1.7} rx={s.r * 0.12} style={{ fill }} />;
    case 2:
      return <polygon points={polygon(s.x, s.y + s.r * 0.15, s.r * 1.05, 3, up)} style={{ fill }} />;
    case 3:
      return <polygon points={polygon(s.x, s.y, s.r * 1.05, 5, up, 0.45)} style={{ fill }} />;
    case 4:
      return <polygon points={polygon(s.x, s.y, s.r, 4, up)} style={{ fill }} />;
    default:
      return <polygon points={polygon(s.x, s.y, s.r, 6, 0)} style={{ fill }} />;
  }
}

function Picture({
  shapes,
  ground,
  found,
  all,
  miss,
  label,
  onTap,
}: {
  shapes: Shape[];
  ground: number;
  found: Spot[];
  all: Spot[] | null;
  miss: Spot | null;
  label: string;
  onTap: (x: number, y: number) => void;
}) {
  const tap = (e: React.MouseEvent<SVGSVGElement>) => {
    const box = e.currentTarget.getBoundingClientRect();
    onTap(Math.round(((e.clientX - box.left) / box.width) * 1000), Math.round(((e.clientY - box.top) / box.height) * 1000));
  };
  return (
    // biome-ignore lint/a11y/useKeyWithClickEvents: tapping a spot in a picture has no keyboard equivalent
    <svg className="fd-pic" viewBox="0 0 1000 1000" role="img" aria-label={label} onClick={tap}>
      <rect width="1000" height="1000" style={{ fill: GROUNDS[ground] }} />
      {shapes.map((s) => (
        <ShapeArt key={s.id} s={s} />
      ))}
      {(all ?? []).map((sp) => (
        <circle key={`a${sp.x},${sp.y}`} className="fd-ring all" cx={sp.x} cy={sp.y} r={sp.r + 18} />
      ))}
      {found.map((sp) => (
        <circle key={`f${sp.x},${sp.y}`} className="fd-ring" cx={sp.x} cy={sp.y} r={sp.r + 18} />
      ))}
      {miss && (
        <g className="fd-miss">
          <line x1={miss.x - 30} y1={miss.y - 30} x2={miss.x + 30} y2={miss.y + 30} />
          <line x1={miss.x + 30} y1={miss.y - 30} x2={miss.x - 30} y2={miss.y + 30} />
        </g>
      )}
    </svg>
  );
}

export default function FindTheDifferences({ view: raw }: ScreenProps) {
  const view = raw as FindTheDifferencesView;
  // The last tap and the miss count before it: a cross shows where a tap missed.
  const [lastTap, setLastTap] = useState<{ x: number; y: number; misses: number } | null>(null);
  const tap = (x: number, y: number) => {
    if (view.revealing) return;
    setLastTap({ x, y, misses: view.misses });
    move({ type: 'tap', x, y });
    window.setTimeout(() => setLastTap((t) => (t && t.x === x && t.y === y ? null : t)), 700);
  };
  useKeys((key) => {
    if ((key === 'Enter' || key === 'n' || key === 'N') && view.revealing) move({ type: 'next' });
    else return false;
    return true;
  });
  const me = view.seats[view.you];
  const lastMiss = lastTap && view.misses > lastTap.misses ? { x: lastTap.x, y: lastTap.y, r: 0 } : null;
  return (
    <Play
      top={
        <>
          <GameTop title="Find the Differences" step={`Round ${view.round} of ${view.rounds}`} score={me?.points ?? 0} />
          <TimerBar leftMs={view.revealing ? view.revealLeftMs : view.timeLeftMs} totalMs={view.revealing ? 4000 : view.roundMs} />
        </>
      }
      stage={
        <div className="fd-pics">
          <Picture
            shapes={view.picture.left}
            ground={view.picture.ground}
            found={view.found}
            all={view.all}
            miss={lastMiss}
            label="Left picture"
            onTap={tap}
          />
          <Picture
            shapes={view.picture.right}
            ground={view.picture.ground}
            found={view.found}
            all={view.all}
            miss={lastMiss}
            label="Right picture"
            onTap={tap}
          />
        </div>
      }
      controls={
        <>
          <div className="fd-count">
            <strong>{view.found.length}</strong> / {view.diffs} found
            {view.misses > 0 && <span className="muted small"> · missed {view.misses}×</span>}
          </div>
          {view.revealing && (
            <button type="button" className="btn primary" onClick={() => move({ type: 'next' })}>
              {view.round < view.rounds ? 'Next pictures' : 'Finish'}
            </button>
          )}
          <ul className="list fd-seats">
            {view.seats.map((s, i) => (
              <li key={s.id} className={i === view.you ? 'me' : ''}>
                <span className="fd-who">{nameOf(s.id)}</span>
                <span className="fd-bar" title={`${s.found} of ${view.diffs}`}>
                  {Array.from({ length: view.diffs }, (_, k) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: fixed number of pips
                    <span key={k} className={k < s.found ? 'on' : ''} />
                  ))}
                </span>
                <span className="score">{s.points}</span>
              </li>
            ))}
          </ul>
          <p className="muted small center">
            Tap a difference in either picture: +30 each, −10 for a tap on nothing. Finding all first: +100.
          </p>
          <KeyHint>Keys: Enter or N goes on after the reveal.</KeyHint>
        </>
      }
    />
  );
}
