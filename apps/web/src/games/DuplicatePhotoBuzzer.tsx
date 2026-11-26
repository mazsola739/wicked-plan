// Montage: buzz when a picture repeats. Rules page: docs/games/duplicate-photo-buzzer.md

import { t } from '../i18n';
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
/** Four grounds: the dark board, tinted a little (as in Spot the Difference). */
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

type Pt = [number, number];

function polygon(x: number, y: number, r: number, n: number, turn: number, inner?: number): Pt[] {
  const steps = inner ? n * 2 : n;
  return Array.from({ length: steps }, (_, i) => {
    const a = turn + (i * 2 * Math.PI) / steps;
    const rr = inner && i % 2 ? r * inner : r;
    return [x + rr * Math.cos(a), y + rr * Math.sin(a)];
  });
}

const points = (ps: Pt[]) => ps.map(([a, b]) => `${a.toFixed(1)},${b.toFixed(1)}`).join(' ');

/** A mark's outline, its centre height and its flat table (0: the facets meet in the centre). */
function outline(kind: number, x: number, y: number, r: number): [Pt[], number, number] {
  const up = -Math.PI / 2;
  switch (kind) {
    case 0:
      return [polygon(x, y, r, 12, up), y, 0.55];
    case 1:
      return [polygon(x, y, r * 1.2, 4, Math.PI / 4), y, 0.6];
    case 2:
      return [polygon(x, y + r * 0.15, r * 1.05, 3, up), y + r * 0.15, 0.45];
    case 3:
      return [polygon(x, y, r * 1.05, 5, up, 0.45), y, 0];
    case 4:
      return [polygon(x, y, r, 4, up), y, 0];
    default:
      return [polygon(x, y, r, 6, 0), y, 0.55];
  }
}

/**
 * A faceted mark, lit from the top left: one facet per edge (a triangle to the centre, or a band
 * around a flat table), white on the lit side, black on the far side, and a tiny glint.
 */
function Mark({ kind, colour, x, y }: { kind: number; colour: number; x: number; y: number }) {
  const r = 130;
  const [outer, cy, table] = outline(kind, x, y, r);
  const inner = outer.map(([px, py]): Pt => [x + (px - x) * table, cy + (py - cy) * table]);
  const facets = outer.map((p, i) => {
    const q = outer[(i + 1) % outer.length] as Pt;
    const [mx, my] = [(p[0] + q[0]) / 2 - x, (p[1] + q[1]) / 2 - cy];
    const lit = -(mx + my) / Math.hypot(mx, my) / Math.SQRT2;
    const shape = table ? [p, q, inner[(i + 1) % outer.length] as Pt, inner[i] as Pt] : [p, q, [x, cy] as Pt];
    return { lit, shape };
  });
  const best = facets.reduce((a, b) => (b.lit > a.lit ? b : a));
  const [gx, gy] = best.shape.reduce(([sx, sy], [px, py]) => [sx + px / best.shape.length, sy + py / best.shape.length], [0, 0]);
  const g = r * 0.08;
  return (
    <g>
      <polygon points={points(outer)} style={{ fill: COLOURS[colour] }} />
      {facets.map(({ lit, shape }, i) =>
        Math.abs(lit) < 0.15 ? null : (
          <polygon
            // biome-ignore lint/suspicious/noArrayIndexKey: a fixed ring of facets
            key={i}
            points={points(shape)}
            fill={lit > 0 ? '#fff' : '#000'}
            fillOpacity={lit > 0 ? 0.08 + 0.37 * lit : 0.1 - 0.25 * lit}
          />
        ),
      )}
      {table > 0 && <polygon points={points(inner)} fill="#fff" fillOpacity={0.15} />}
      <polygon
        points={points([
          [gx, gy - g],
          [gx + g, gy],
          [gx, gy + g],
          [gx - g, gy],
        ])}
        fill="#fff"
      />
    </g>
  );
}

function PictureArt({ p, className }: { p: Picture; className?: string }) {
  const spots = LAYOUTS[p.layout] ?? LAYOUTS[0] ?? [];
  return (
    <svg className={`db-pic ${className ?? ''}`} viewBox="0 0 1000 1000" role="img" aria-label={t('Picture')}>
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
          <GameTop title="Montage" step={t('Round {n} of {total}', { n: view.round, total: view.rounds })} score={me?.points ?? 0} />
          {r ? <TimerBar leftMs={view.revealLeftMs} totalMs={2500} /> : <TimerBar leftMs={view.frameLeftMs} totalMs={view.frameMs} />}
        </>
      }
      stage={
        <div className="db-stage">
          {r ? (
            <>
              <PictureArt p={r.picture} />
              <p className="db-caption">{t('Picture {first} came back as picture {second}.', { first: r.first, second: r.second })}</p>
            </>
          ) : (
            view.picture && (
              <>
                <PictureArt key={view.number} p={view.picture} className="flash" />
                <p className="db-caption muted">{t('Picture {n}', { n: view.number })}</p>
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
                {r.winner
                  ? r.winner === me?.id
                    ? t('You buzzed first!')
                    : t('{name} buzzed first.', { name: nameOf(r.winner) })
                  : t('Nobody caught the repeat.')}
              </p>
              <button type="button" className="btn primary" onClick={() => move({ type: 'next' })}>
                {view.round < view.rounds ? t('Next round') : t('Finish')}
              </button>
            </>
          ) : (
            <button type="button" className="db-buzz" disabled={!canBuzz} onClick={buzz}>
              {view.locked ? t('Out this round') : t('BUZZ')}
            </button>
          )}
          <ul className="list db-seats">
            {view.seats.map((s, i) => (
              <li key={s.id} className={i === view.you ? 'me' : ''}>
                <span>{nameOf(s.id)}</span>
                <span className={`small ${s.buzz === true ? 'db-right' : s.buzz === false ? 'db-wrong' : 'muted'}`}>
                  {s.buzz === true ? t('buzzed first') : s.buzz === false ? t('wrong buzz') : ''}
                </span>
                <span className="score">{s.points}</span>
              </li>
            ))}
          </ul>
          <p className="muted small center">
            {t('Buzz while the repeat or the picture after it is up: 100 + up to 60 for speed. A wrong buzz: −30.')}
          </p>
          <KeyHint>{t('Keys: Space or B buzzes, Enter goes on.')}</KeyHint>
        </>
      }
    />
  );
}
