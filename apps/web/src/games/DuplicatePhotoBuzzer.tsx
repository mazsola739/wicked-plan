// Duplicate Picture Buzzer: buzz when a picture repeats. Rules page: docs/games/duplicate-photo-buzzer.md

import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './DuplicatePhotoBuzzer.css';

interface Picture {
  ground: number;
  layout: number;
  marks: { kind: number; colour: number }[];
}

interface BuzzerView {
  round: number;
  rounds: number;
  picture: Picture | null;
  number: number;
  frameMs: number;
  frameLeftMs: number;
  locked: boolean;
  seats: { id: string; points: number; buzz: boolean | null }[];
  you: number;
  reveal: { picture: Picture; first: number; second: number; winner: string | null } | null;
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
/** Four grounds: the dark board, tinted a little (as in Find the Differences). */
const GROUNDS = [
  'color-mix(in srgb, var(--violet) 16%, var(--board))',
  'color-mix(in srgb, var(--cyan) 10%, var(--board))',
  'color-mix(in srgb, var(--g-blue) 16%, var(--board))',
  'color-mix(in srgb, var(--g-pink) 12%, var(--board))',
];
const LAYOUTS: [number, number][][] = [
  [
    [250, 300],
    [750, 300],
    [500, 720],
  ],
  [
    [250, 700],
    [750, 700],
    [500, 280],
  ],
  [
    [190, 500],
    [500, 500],
    [810, 500],
  ],
  [
    [280, 260],
    [720, 500],
    [300, 760],
  ],
];

function polygon(x: number, y: number, r: number, n: number, turn: number, inner?: number): string {
  const steps = inner ? n * 2 : n;
  return Array.from({ length: steps }, (_, i) => {
    const a = turn + (i * 2 * Math.PI) / steps;
    const rr = inner && i % 2 ? r * inner : r;
    return `${(x + rr * Math.cos(a)).toFixed(1)},${(y + rr * Math.sin(a)).toFixed(1)}`;
  }).join(' ');
}

function Mark({ kind, colour, x, y }: { kind: number; colour: number; x: number; y: number }) {
  const style = { fill: COLOURS[colour] };
  const r = 130;
  const up = -Math.PI / 2;
  switch (kind) {
    case 0:
      return <circle cx={x} cy={y} r={r} style={style} />;
    case 1:
      return <rect x={x - r * 0.85} y={y - r * 0.85} width={r * 1.7} height={r * 1.7} rx={r * 0.12} style={style} />;
    case 2:
      return <polygon points={polygon(x, y + r * 0.15, r * 1.05, 3, up)} style={style} />;
    case 3:
      return <polygon points={polygon(x, y, r * 1.05, 5, up, 0.45)} style={style} />;
    case 4:
      return <polygon points={polygon(x, y, r, 4, up)} style={style} />;
    default:
      return <polygon points={polygon(x, y, r, 6, 0)} style={style} />;
  }
}

function PictureArt({ p, className }: { p: Picture; className?: string }) {
  const spots = LAYOUTS[p.layout] ?? LAYOUTS[0] ?? [];
  return (
    <svg className={`db-pic ${className ?? ''}`} viewBox="0 0 1000 1000" role="img" aria-label="Picture">
      <rect width="1000" height="1000" style={{ fill: GROUNDS[p.ground] }} />
      {p.marks.map((m, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: three marks in fixed spots
        <Mark key={i} kind={m.kind} colour={m.colour} x={spots[i]?.[0] ?? 500} y={spots[i]?.[1] ?? 500} />
      ))}
    </svg>
  );
}

export default function DuplicatePhotoBuzzer({ view: raw }: ScreenProps) {
  const view = raw as BuzzerView;
  const canBuzz = !view.reveal && !view.locked;
  const buzz = () => canBuzz && move({ type: 'buzz' });
  useKeys((key) => {
    if (key === ' ' || key === 'b' || key === 'B') buzz();
    else if (key === 'Enter' && view.reveal) move({ type: 'next' });
    else return false;
    return true;
  });
  const me = view.seats[view.you];
  const r = view.reveal;
  return (
    <Play
      top={
        <>
          <GameTop title="Duplicate Picture Buzzer" step={`Round ${view.round} of ${view.rounds}`} score={me?.points ?? 0} />
          {r ? <TimerBar leftMs={view.revealLeftMs} totalMs={2500} /> : <TimerBar leftMs={view.frameLeftMs} totalMs={view.frameMs} />}
        </>
      }
      stage={
        <div className="db-stage">
          {r ? (
            <>
              <PictureArt p={r.picture} />
              <p className="db-caption">
                Picture {r.first} came back as picture {r.second}.
              </p>
            </>
          ) : (
            view.picture && (
              <>
                <PictureArt key={view.number} p={view.picture} className="flash" />
                <p className="db-caption muted">Picture {view.number}</p>
              </>
            )
          )}
        </div>
      }
      controls={
        <>
          {r ? (
            <>
              <p className={`last ${r.winner === me?.id ? 'ok' : 'bad'}`}>
                {r.winner ? (r.winner === me?.id ? 'You buzzed first!' : `${nameOf(r.winner)} buzzed first.`) : 'Nobody caught the repeat.'}
              </p>
              <button type="button" className="btn primary" onClick={() => move({ type: 'next' })}>
                {view.round < view.rounds ? 'Next round' : 'Finish'}
              </button>
            </>
          ) : (
            <button type="button" className="db-buzz" disabled={!canBuzz} onClick={buzz}>
              {view.locked ? 'Out this round' : 'BUZZ'}
            </button>
          )}
          <ul className="list db-seats">
            {view.seats.map((s, i) => (
              <li key={s.id} className={i === view.you ? 'me' : ''}>
                <span>{nameOf(s.id)}</span>
                <span className={`small ${s.buzz === true ? 'db-right' : s.buzz === false ? 'db-wrong' : 'muted'}`}>
                  {s.buzz === true ? 'buzzed first' : s.buzz === false ? 'wrong buzz' : ''}
                </span>
                <span className="score">{s.points}</span>
              </li>
            ))}
          </ul>
          <p className="muted small center">
            Buzz while the repeat or the picture after it is up: 100 + up to 60 for speed. A wrong buzz: −30.
          </p>
          <KeyHint>Keys: Space or B buzzes, Enter goes on.</KeyHint>
        </>
      }
    />
  );
}
