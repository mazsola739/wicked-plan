// Russian Roulette: a paint drum goes round; spend credits to peek, spin or tamper. Rules: docs/games/russian-roulette.md

import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './RussianRoulette.css';

type Phase = 'turn' | 'shot' | 'over';
type Event = 'spin' | 'click' | 'hit' | 'reload';

interface RrView {
  phase: Phase;
  timeLeftMs: number;
  phaseMs: number;
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

function Drum({ v }: { v: RrView }) {
  const shotEvent = [...v.log].reverse().find((e) => e.event === 'hit' || e.event === 'click');
  const splat = v.phase === 'shot' && shotEvent?.event === 'hit';
  return (
    <svg className={`russian-roulette-drum${splat ? ' splat' : ''}`} viewBox="-60 -60 120 120" role="img" aria-label="The drum">
      <circle r="52" className="body" />
      {Array.from({ length: 6 }, (_, i) => {
        const a = (Math.PI / 3) * i - Math.PI / 2;
        // biome-ignore lint/suspicious/noArrayIndexKey: six fixed chambers
        return <circle key={i} cx={34 * Math.cos(a)} cy={34 * Math.sin(a)} r="11" className={i === 0 ? 'chamber next' : 'chamber'} />;
      })}
      <circle r="9" className="hub" />
      {splat && <path className="paint" d="M-30 -8 Q-10 -40 8 -22 Q40 -36 30 0 Q46 24 12 22 Q-4 46 -18 22 Q-48 22 -30 -8Z" />}
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
      top={
        <>
          <GameTop title="Russian Roulette" step={`Pull ${Math.min(v.pulls + 1, v.maxPulls)} of ${v.maxPulls}`} score={v.credits} />
          <TimerBar leftMs={v.timeLeftMs} totalMs={v.phaseMs} />
        </>
      }
      stage={
        <div className="card">
          <div className="russian-roulette-top">
            <Drum v={v} />
            <div className="russian-roulette-odds">
              <div className="label">Charges left</div>
              <div className="russian-roulette-charges">
                {Array.from({ length: 2 }, (_, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: two fixed charges
                  <i key={i} className={i < v.chargesLeft ? 'on' : ''} />
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
                    <i key={i} className={i < s.lives ? 'on' : ''} />
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
