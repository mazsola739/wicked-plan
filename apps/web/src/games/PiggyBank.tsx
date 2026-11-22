// Piggy Bank: drop coins, claim what is inside, doubt the others. Rules: docs/games/piggy-bank.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './PiggyBank.css';

type Phase = 'drop' | 'claim' | 'doubt' | 'result' | 'over';
type Colour = 'white' | 'black';

interface Opening {
  round: number;
  colour: Colour;
  claimer: string;
  claim: number;
  truth: number;
  white: number;
  black: number;
  doubters: string[];
  losers: string[];
  full: boolean;
}

interface PbView {
  phase: Phase;
  round: number;
  rounds: number;
  colour: Colour;
  timeLeftMs: number;
  phaseMs: number;
  myTurn: boolean;
  coins: number;
  drops: [string, number][];
  claims: { id: string; count: number; coins: number }[];
  accepted: number;
  mine: [number, number];
  seen: [number, number] | null;
  myChoice: boolean | null;
  seats: { id: string; score: number; holder: boolean; done: boolean }[];
  log: Opening[];
}

const Coin = ({ c }: { c: Colour }) => <span className={`piggy-bank-coin ${c === 'white' ? 'piggy-bank-white' : 'piggy-bank-black'}`} />;

function Pig({ coins, open }: { coins: number; open: boolean }) {
  return (
    <svg className={`piggy-bank-pig${open ? ' open' : ''}`} viewBox="0 0 160 110" role="img" aria-label={`Piggy bank, ${coins} coins`}>
      <ellipse cx="80" cy="60" rx="58" ry="40" className="body" />
      <path d="M44 30 L52 8 L64 26Z" className="ear" />
      <ellipse cx="136" cy="62" rx="14" ry="12" className="snout" />
      <circle cx="132" cy="60" r="2.5" className="dark" />
      <circle cx="140" cy="60" r="2.5" className="dark" />
      <circle cx="112" cy="44" r="4" className="dark" />
      <rect x="62" y="22" width="34" height="6" rx="3" className="dark" />
      {[44, 66, 94, 116].map((x) => (
        <rect key={x} x={x - 7} y="90" width="14" height="16" rx="4" className="body" />
      ))}
      <text x="78" y="72" className="count">
        {coins}
      </text>
    </svg>
  );
}

export default function PiggyBank({ view }: ScreenProps) {
  const v = view as PbView;
  const [pending, setPending] = useState<Colour[]>([]);
  const [claim, setClaim] = useState<number | null>(null);
  const [forKey, setForKey] = useState('');
  const key = `${v.round}-${v.phase}-${v.claims.length}-${v.drops.length}`;
  if (forKey !== key) {
    setForKey(key);
    setPending([]);
    setClaim(null);
  }
  const canDrop = v.phase === 'drop' && v.myTurn;
  const canClaim = v.phase === 'claim' && v.myTurn;
  const canDoubt = v.phase === 'doubt' && v.myTurn;
  const seenColour = v.seen ? (v.colour === 'white' ? v.seen[0] : v.seen[1]) : 0;
  const suggestion = Math.min(v.coins, v.accepted + seenColour);
  const value = claim ?? suggestion;
  const add = (c: Colour) => canDrop && setPending((p) => (p.length < 2 ? [...p, c] : p));
  const drop = () =>
    pending.length > 0 &&
    move({ type: 'drop', white: pending.filter((c) => c === 'white').length, black: pending.filter((c) => c === 'black').length });
  const sendClaim = () => canClaim && move({ type: 'claim', count: value });
  useKeys((k) => {
    const key = k.toLowerCase();
    if (canDrop && key === 'w') add('white');
    else if (canDrop && key === 'b') add('black');
    else if (canDrop && k === 'Backspace') setPending((p) => p.slice(0, -1));
    else if (canDrop && k === 'Enter') drop();
    else if (canClaim && /^[0-9]$/.test(k)) setClaim(Math.min(v.coins, Number(k)));
    else if (canClaim && k === 'Enter') sendClaim();
    else if (canDoubt && key === 'd') move({ type: 'doubt' });
    else if (canDoubt && key === 't') move({ type: 'trust' });
    else return false;
    return true;
  });
  const me = v.seats.find((s) => s.id === 'you');
  const holder = v.seats.find((s) => s.holder);
  const lastClaim = v.claims[v.claims.length - 1];
  const last = v.log[v.log.length - 1];
  const showResult = (v.phase === 'result' || v.phase === 'over') && last;
  let ask = '';
  if (v.phase === 'drop')
    ask = canDrop ? 'Drop 1 or 2 coins, then pass the bank.' : `${holder ? nameOf(holder.id) : ''} is dropping coins…`;
  else if (v.phase === 'claim')
    ask = canClaim ? `How many ${v.colour} coins are inside?` : `${holder ? nameOf(holder.id) : ''} is peeking…`;
  else if (v.phase === 'doubt' && lastClaim)
    ask = canDoubt
      ? `${nameOf(lastClaim.id)} says ${lastClaim.count} ${v.colour}. Doubt it?`
      : v.myChoice !== null
        ? `You ${v.myChoice ? 'doubt' : 'trust'} it. Waiting…`
        : `Waiting for the others to judge your claim…`;
  else if (showResult) {
    const right = last.claim === last.truth;
    const verdict = right ? 'right' : 'wrong';
    const lost = last.losers.length
      ? `${last.losers.map(nameOf).join(', ')} ${last.losers.length === 1 && last.losers[0] !== 'you' ? 'loses' : 'lose'} a point.`
      : 'Nobody loses.';
    ask = `${last.full ? 'Full bank! ' : ''}${nameOf(last.claimer)} said ${last.claim}, there were ${last.truth}: ${verdict}. ${lost}`;
  }
  return (
    <Play
      top={
        <>
          <GameTop title="Piggy Bank" step={`Round ${v.round} of ${v.rounds}`} score={me?.score ?? 0} />
          <TimerBar leftMs={v.timeLeftMs} totalMs={v.phaseMs} />
        </>
      }
      stage={
        <div className="card">
          <div className="piggy-bank-top">
            <Pig coins={showResult ? last.white + last.black : v.coins} open={!!showResult} />
            <div className="piggy-bank-info">
              <div className="label">This round counts</div>
              <div>
                <Coin c={v.colour} /> {v.colour}
              </div>
              {showResult ? (
                <div className="small">
                  Inside: {last.white} <Coin c="white" /> {last.black} <Coin c="black" />
                </div>
              ) : (
                <div className="muted small">Last accepted claim: {v.accepted}</div>
              )}
              <div className="muted small">
                Your coins: {v.mine[0]} {v.colour}, {v.mine[1]} other
              </div>
            </div>
          </div>
          <ul className="list">
            {v.seats.map((s) => (
              <li key={s.id} className={s.holder ? 'piggy-bank-holder' : ''}>
                <span>
                  {nameOf(s.id)} <span className="score">{s.score}</span>{' '}
                  {v.phase === 'doubt' && (
                    <span className="muted small">{s.id === lastClaim?.id ? 'claimer' : s.done ? 'decided' : 'thinking…'}</span>
                  )}
                  {showResult && last.doubters.includes(s.id) && <span className="muted small">doubted</span>}
                </span>
                <span className="muted small">
                  {v.drops
                    .filter(([id]) => id === s.id)
                    .map(([, n]) => n)
                    .join(' + ') || ''}
                  {v.drops.some(([id]) => id === s.id) ? ' dropped' : ''}
                </span>
              </li>
            ))}
          </ul>
          {v.claims.length > 0 && !showResult && (
            <p className="muted small">Claims: {v.claims.map((c) => `${nameOf(c.id)} ${c.count} of ${c.coins}`).join(' · ')}</p>
          )}
        </div>
      }
      controls={
        <>
          <p className="ask">{ask}</p>
          {canDrop && (
            <>
              <div className="piggy-bank-pending">
                {pending.length === 0 && <span className="muted small">No coins yet</span>}
                {pending.map((c, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: at most two coins
                  <Coin key={i} c={c} />
                ))}
              </div>
              <div className="row">
                <button type="button" className="btn" disabled={pending.length >= 2} onClick={() => add('white')}>
                  + White
                </button>
                <button type="button" className="btn" disabled={pending.length >= 2} onClick={() => add('black')}>
                  + Black
                </button>
                <button type="button" className="btn ghost" disabled={pending.length === 0} onClick={() => setPending([])}>
                  Clear
                </button>
              </div>
              <button type="button" className="btn primary" disabled={pending.length === 0} onClick={drop}>
                Drop and pass
              </button>
            </>
          )}
          {canClaim && (
            <>
              {v.seen && (
                <p className="center">
                  You see the last drop: {v.seen[0] > 0 && `${v.seen[0]} `}
                  {Array.from({ length: v.seen[0] }, (_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: identical coins
                    <Coin key={`w${i}`} c="white" />
                  ))}{' '}
                  {v.seen[1] > 0 && `${v.seen[1]} `}
                  {Array.from({ length: v.seen[1] }, (_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: identical coins
                    <Coin key={`b${i}`} c="black" />
                  ))}
                </p>
              )}
              <div className="piggy-bank-stepper">
                <button type="button" className="key" disabled={value <= 0} onClick={() => setClaim(Math.max(0, value - 1))}>
                  −
                </button>
                <strong>{value}</strong>
                <button type="button" className="key" disabled={value >= v.coins} onClick={() => setClaim(Math.min(v.coins, value + 1))}>
                  +
                </button>
              </div>
              <button type="button" className="btn primary" onClick={sendClaim}>
                Claim {value} {v.colour}
              </button>
            </>
          )}
          {canDoubt && (
            <div className="row">
              <button type="button" className="btn" onClick={() => move({ type: 'trust' })}>
                Trust
              </button>
              <button type="button" className="btn primary" onClick={() => move({ type: 'doubt' })}>
                Doubt!
              </button>
            </div>
          )}
          <KeyHint>W / B add a coin, Enter drops · digits and Enter claim · D doubt, T trust.</KeyHint>
        </>
      }
    />
  );
}
