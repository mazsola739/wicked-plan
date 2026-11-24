// Russian Roulette: a paint drum goes round; spend credits to peek, spin or tamper. Rules: docs/games/russian-roulette.md

import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './RussianRoulette.css';

type Phase = 'turn' | 'shot' | 'over';
type Event = 'spin' | 'click' | 'hit' | 'reload';

interface RrView {
  phase: Phase;
  myTurn: boolean;
  lives: number;
  credits: number;
  peek: boolean | null;
  chargesLeft: number;
  pullsSinceMix: number;
  pulls: number;
  maxPulls: number;
  seats: { id: string; lives: number; toAct: boolean }[];
  log: { id: string; event: Event }[];
}

const COSTS = { peek: 3, spin: 2, tamper: 4 } as const;

function line(e: { id: string; event: Event }): string {
  const who = nameOf(e.id);
  switch (e.event) {
    case 'spin':
      return `${who} spun the drum.`;
    case 'click':
      return `${who} pulled: click, safe.`;
    case 'hit':
      return `${who} pulled: SPLAT, hit!`;
    case 'reload':
      return 'Both charges spent: the drum is reloaded.';
  }
}

// Faceted icons (docs/design.md, "Icons are faceted"): lit from the top left.
const pt = (a: number, r: number, x = 0, y = 0) => `${(x + r * Math.cos(a)).toFixed(1)},${(y + r * Math.sin(a)).toFixed(1)}`;
const glint = (x: number, y: number, r: number) => `M${x},${y - r} ${x + r},${y} ${x},${y + r} ${x - r},${y}Z`;

/** The overlay for a facet that faces angle `a`: white toward the light, black away from it. */
function shade(a: number, k = 1) {
  const d = Math.cos(a + (3 * Math.PI) / 4);
  if (Math.abs(d) < 0.2) return null;
  return d > 0 ? { fill: '#fff', fillOpacity: k * (0.1 + 0.4 * d) } : { fill: '#000', fillOpacity: k * (0.12 + 0.23 * -d) };
}

/** `n` facets from radius `r0` in to `r1` (0: a point) around (x, y), shaded by the way each faces. */
function ring(n: number, r0: number, r1: number, k = 1, x = 0, y = 0) {
  return Array.from({ length: n }, (_, i) => {
    const a = (2 * Math.PI * i) / n;
    const b = a + (2 * Math.PI) / n;
    return { d: `M${pt(a, r0, x, y)} ${pt(b, r0, x, y)} ${pt(b, r1, x, y)} ${pt(a, r1, x, y)}Z`, s: shade((a + b) / 2, k) };
  });
}

// The steel drum: a bevelled rim, a shallow cone to the hub, a raised hub; chambers are cut in.
const DRUM = [...ring(12, 54, 45), ...ring(12, 45, 9, 0.35)];
const HUB_FACETS = ring(6, 9, 0);
const DRUM_EDGE = `M${Array.from({ length: 12 }, (_, i) => pt((Math.PI / 6) * i, 54)).join(' ')}Z`;
const HUB = `M${Array.from({ length: 6 }, (_, i) => pt((Math.PI / 3) * i, 9)).join(' ')}Z`;
const CHAMBERS = Array.from({ length: 6 }, (_, i) => {
  const a = (Math.PI / 3) * i - Math.PI / 2;
  const [x, y] = [34 * Math.cos(a), 34 * Math.sin(a)];
  const at = (k: number, r: number, dx = 0) => pt((Math.PI / 4) * k, r, x + dx, y + dx);
  // The far (lower-right) wall of the hole catches the light.
  return {
    hole: `M${[0, 1, 2, 3, 4, 5, 6, 7].map((k) => at(k, 11)).join(' ')}Z`,
    wall: `M${at(0, 11)} ${at(1, 11)} ${at(2, 11)} ${at(2, 9, -1.4)} ${at(1, 9, -1.4)} ${at(0, 9, -1.4)}Z`,
  };
});
// The paint splat: an uneven low-poly blot, each facet shaded like a dome, and a few drops.
const SPLAT_R = [30, 44, 28, 32, 26, 38, 29, 27, 47, 30, 26, 33, 28, 41, 27, 31, 25, 36, 29, 28];
const SPLAT = SPLAT_R.map((r, i) => pt((Math.PI / 10) * i, r, 0, 2));
const SPLAT_FACETS = SPLAT.map((p, i) => ({ d: `M-4,-2 ${p} ${SPLAT[(i + 1) % 20]}Z`, s: shade((Math.PI / 10) * (i + 0.5), 0.8) }));

function Drum({ v }: { v: RrView }) {
  const shotEvent = [...v.log].reverse().find((e) => e.event === 'hit' || e.event === 'click');
  const splat = v.phase === 'shot' && shotEvent?.event === 'hit';
  return (
    <svg className={`russian-roulette-drum${splat ? ' splat' : ''}`} viewBox="-60 -60 120 120" role="img" aria-label="The drum">
      <path className="body" d={DRUM_EDGE} />
      {DRUM.map((f) => f.s && <path key={f.d} d={f.d} {...f.s} />)}
      {CHAMBERS.map((c, i) => (
        <g key={c.hole}>
          <path className={i === 0 ? 'chamber next' : 'chamber'} d={c.hole} />
          <path fill="#fff" fillOpacity=".18" d={c.wall} />
        </g>
      ))}
      <path className="hub" d={HUB} />
      {HUB_FACETS.map((f) => f.s && <path key={f.d} d={f.d} {...f.s} />)}
      <path className="eye" d={glint(0, 0, 2.6)} />
      <path fill="#fff" d={`${glint(-35, -35, 2.4)} ${glint(-3.5, -3.5, 1.3)}`} />
      {splat && (
        <g className="paint">
          <path d={`M${SPLAT.join(' ')}Z ${glint(50, -30, 4)} ${glint(-48, 32, 3.5)} ${glint(30, 47, 3)}`} />
          {SPLAT_FACETS.map((f) => f.s && <path key={f.d} d={f.d} {...f.s} />)}
          <path fill="#fff" d={`${glint(-14, -12, 2.4)} ${glint(-21, -15, 1.2)}`} />
        </g>
      )}
    </svg>
  );
}

/** A life (a heart) or a paint charge (a drop), dim once it is spent. */
function Pip({ kind, on }: { kind: 'heart' | 'drop'; on: boolean }) {
  return (
    <svg viewBox="0 0 40 40" className={on ? 'on' : ''} aria-hidden="true">
      {kind === 'heart' ? (
        <>
          <path d="M20,11 14,5 7,5 2,11 2,19 20,36 38,19 38,11 33,5 26,5Z" />
          <path fill="#fff" fillOpacity=".5" d="M7,5 14,5 11,14Z" />
          <path fill="#fff" fillOpacity=".3" d="M2,11 7,5 11,14Z M14,5 20,11 11,14Z" />
          <path fill="#fff" fillOpacity=".15" d="M2,11 11,14 2,19Z M11,14 20,11 20,22Z M26,5 33,5 29,14Z" />
          <path fill="#000" fillOpacity=".12" d="M33,5 38,11 29,14Z M20,11 29,14 20,22Z" />
          <path fill="#000" fillOpacity=".25" d="M38,11 38,19 29,14Z" />
          <path fill="#000" fillOpacity=".32" d="M29,14 38,19 20,36 20,22Z" />
          <path fill="#fff" d={glint(8.5, 9, 1.8)} />
        </>
      ) : (
        <>
          <path d="M20,3 29,16 32,24 28,32 20,36 12,32 8,24 11,16Z" />
          <path fill="#fff" fillOpacity=".45" d="M20,3 11,16 17,22Z" />
          <path fill="#fff" fillOpacity=".22" d="M20,3 17,22 29,16Z M11,16 8,24 17,22Z" />
          <path fill="#000" fillOpacity=".12" d="M29,16 17,22 32,24Z M12,32 20,36 17,22Z" />
          <path fill="#000" fillOpacity=".3" d="M32,24 17,22 28,32Z M20,36 28,32 17,22Z" />
          <path fill="#fff" d={glint(14.5, 15, 1.8)} />
        </>
      )}
    </svg>
  );
}

export default function RussianRoulette({ view }: ScreenProps) {
  const v = view as RrView;
  const act = (type: 'peek' | 'spin' | 'tamper' | 'pull') => v.myTurn && move({ type });
  useKeys((k) => {
    const key = k.toLowerCase();
    if (key === 'p') act('peek');
    else if (key === 's') act('spin');
    else if (key === 't') act('tamper');
    else if (k === ' ' || k === 'Enter') act('pull');
    else return false;
    return true;
  });
  const unknown = Math.max(1, 6 - Math.min(5, v.pullsSinceMix));
  const actor = v.seats.find((s) => s.toAct);
  const last = v.log.filter((e) => e.event !== 'reload').at(-1);
  let ask = '';
  if (v.phase === 'shot' && last) ask = line(last);
  else if (v.phase === 'turn')
    ask = v.myTurn ? (v.lives > 0 ? 'Your turn: act, then pull.' : '') : `${actor ? nameOf(actor.id) : ''} has the drum…`;
  else if (v.phase === 'over') ask = 'The show is over.';
  return (
    <Play
      top={<GameTop title="Russian Roulette" step={`Pull ${Math.min(v.pulls + 1, v.maxPulls)} of ${v.maxPulls}`} score={v.credits} />}
      stage={
        <div className="card">
          <div className="russian-roulette-top">
            <Drum v={v} />
            <div className="russian-roulette-odds">
              <div className="label">Charges left</div>
              <div className="russian-roulette-charges">
                {Array.from({ length: 2 }, (_, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: two fixed charges
                  <Pip key={i} kind="drop" on={i < v.chargesLeft} />
                ))}
              </div>
              <div className="muted small">
                Table odds: {v.chargesLeft} in {unknown}
              </div>
            </div>
          </div>
          <ul className="list">
            {v.seats.map((s) => (
              <li key={s.id} className={s.toAct ? 'russian-roulette-turn' : s.lives === 0 ? 'russian-roulette-out' : ''}>
                <span>
                  {nameOf(s.id)} {s.lives === 0 && <span className="muted small">out</span>}
                </span>
                <span className="russian-roulette-lives">
                  {Array.from({ length: 2 }, (_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: two fixed lives
                    <Pip key={i} kind="heart" on={i < s.lives} />
                  ))}
                </span>
              </li>
            ))}
          </ul>
          {v.log.length > 0 && (
            <ul className="russian-roulette-log muted small">
              {v.log
                .slice(-6)
                .reverse()
                .map((e, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: log lines have no id
                  <li key={`${v.log.length}-${i}`}>{line(e)}</li>
                ))}
            </ul>
          )}
        </div>
      }
      controls={
        <>
          <p className={`ask${v.phase === 'shot' && last?.event === 'hit' ? ' russian-roulette-hit' : ''}`}>{ask}</p>
          <p className="muted small center">
            Your credits: <strong className="score">{v.credits}</strong> (secret)
            {v.peek !== null && (
              <>
                {' · '}
                <strong className={v.peek ? 'russian-roulette-hit' : 'russian-roulette-safe'}>Peek: {v.peek ? 'CHARGED' : 'empty'}</strong>
              </>
            )}
          </p>
          <div className="row">
            {(['peek', 'spin', 'tamper'] as const).map((a) => (
              <button key={a} type="button" className="btn" disabled={!v.myTurn || v.credits < COSTS[a]} onClick={() => act(a)}>
                {a[0]?.toUpperCase()}
                {a.slice(1)} ({COSTS[a]})
              </button>
            ))}
          </div>
          <button type="button" className="btn primary russian-roulette-pull" disabled={!v.myTurn} onClick={() => act('pull')}>
            Pull
          </button>
          <KeyHint>P peek · S spin · T tamper · Space pull.</KeyHint>
        </>
      }
    />
  );
}
