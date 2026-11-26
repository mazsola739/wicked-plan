// Piggy Bank: drop coins, claim what is inside, doubt the others. Rules: docs/games/piggy-bank.md

import { useState } from 'react';
import { t } from '../i18n';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
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

/** A faceted coin (an octagon cut like a gem, a minted diamond in the middle), lit from the top left. */
const Coin = ({ c }: { c: Colour }) => (
  <svg viewBox="0 0 20 20" className={`piggy-bank-coin piggy-bank-${c}`} role="img" aria-label={t('{colour} coin', { colour: t(c) })}>
    <path className="edge" d="M6.4,1.2 13.6,1.2 18.8,6.4 18.8,13.6 13.6,18.8 6.4,18.8 1.2,13.6 1.2,6.4Z" />
    <path fill="#fff" fillOpacity=".5" d="M1.2,6.4 6.4,1.2 7.6,4.2 4.2,7.6Z" />
    <path fill="#fff" fillOpacity=".3" d="M6.4,1.2 13.6,1.2 12.4,4.2 7.6,4.2Z M7,10 10,7 10,10Z" />
    <path fill="#fff" fillOpacity=".18" d="M1.2,6.4 4.2,7.6 4.2,12.4 1.2,13.6Z M13.6,1.2 18.8,6.4 15.8,7.6 12.4,4.2Z" />
    <path fill="#000" fillOpacity=".12" d="M10,7 13,10 10,10Z" />
    <path fill="#000" fillOpacity=".15" d="M18.8,6.4 18.8,13.6 15.8,12.4 15.8,7.6Z M1.2,13.6 4.2,12.4 7.6,15.8 6.4,18.8Z" />
    <path fill="#000" fillOpacity=".25" d="M13.6,18.8 6.4,18.8 7.6,15.8 12.4,15.8Z M10,13 7,10 10,10Z" />
    <path fill="#000" fillOpacity=".35" d="M18.8,13.6 13.6,18.8 12.4,15.8 15.8,12.4Z M13,10 10,13 10,10Z" />
    <path fill="#fff" d="M5.6,5.8 6.4,6.6 5.6,7.4 4.8,6.6Z" />
  </svg>
);

/** The legs' x positions; each leg is lit on its left half, shaded on its right, with a dark hoof. */
const LEGS = [44, 66, 94, 116];

/** A faceted pink pig facing right, lit from the top left; the count sits on its side. */
function Pig({ coins, open }: { coins: number; open: boolean }) {
  return (
    <svg
      className={`piggy-bank-pig${open ? ' open' : ''}`}
      viewBox="0 0 160 110"
      role="img"
      aria-label={t('Piggy bank, {n} coins', { n: coins })}
    >
      <path d="M88,22 96,6 105,22Z" />
      <path fill="#000" fillOpacity=".3" d="M88,22 96,6 105,22Z" />
      <path d="M24,50 12,41 8,49 15,52 10,58 22,61Z" />
      <path fill="#000" fillOpacity=".2" d="M8,49 15,52 10,58 22,61Z" />
      {LEGS.map((x) => (
        <g key={x}>
          <path d={`M${x - 7},84 ${x + 7},84 ${x + 6},106 ${x - 6},106Z`} />
          <path fill="#fff" fillOpacity=".2" d={`M${x - 7},84 ${x},84 ${x},106 ${x - 6},106Z`} />
          <path fill="#000" fillOpacity=".25" d={`M${x},84 ${x + 7},84 ${x + 6},106 ${x},106Z`} />
          <path fill="#000" fillOpacity=".3" d={`M${x - 6.2},101 ${x + 6.2},101 ${x + 6},106 ${x - 6},106Z`} />
        </g>
      ))}
      <path d="M24,46 40,28 70,19 102,19 124,29 136,44 137,76 124,90 96,98 60,99 34,91 22,72Z" />
      <path fill="#fff" fillOpacity=".42" d="M40,28 70,19 58,40Z" />
      <path fill="#fff" fillOpacity=".3" d="M24,46 40,28 58,40Z" />
      <path fill="#fff" fillOpacity=".22" d="M70,19 102,19 96,38 58,40Z" />
      <path fill="#fff" fillOpacity=".12" d="M102,19 124,29 96,38Z M24,46 58,40 22,72Z M58,40 96,38 80,74Z" />
      <path fill="#000" fillOpacity=".12" d="M136,44 137,76 120,58Z M22,72 80,74 34,91Z" />
      <path fill="#000" fillOpacity=".2" d="M34,91 80,74 60,99Z M96,38 120,58 80,74Z" />
      <path fill="#000" fillOpacity=".26" d="M60,99 80,74 96,98Z" />
      <path fill="#000" fillOpacity=".32" d="M96,98 80,74 120,58 124,90Z" />
      <path fill="#000" fillOpacity=".35" d="M124,90 120,58 137,76Z" />
      <path d="M102,23 113,4 125,31Z" />
      <path fill="#fff" fillOpacity=".38" d="M102,23 113,4 113,27Z" />
      <path fill="#000" fillOpacity=".22" d="M113,4 125,31 113,27Z" />
      <path d="M130,49 144,47 152,55 153,68 146,76 131,75Z" />
      <path fill="#fff" fillOpacity=".36" d="M130,49 144,47 142,61Z" />
      <path fill="#fff" fillOpacity=".14" d="M144,47 152,55 142,61Z M131,75 130,49 142,61Z" />
      <path fill="#000" fillOpacity=".14" d="M152,55 153,68 142,61Z" />
      <path fill="#000" fillOpacity=".32" d="M153,68 146,76 142,61Z" />
      <path fill="#000" fillOpacity=".22" d="M146,76 131,75 142,61Z" />
      <path
        className="eye"
        d="M114,39.5 118,42 118,46 114,48.5 110,46 110,42Z M62,25 96,24 95,29 63,30Z M138,57 140,59.5 139,64 136.5,61.5Z M147,57 149,59.5 148,64 145.5,61.5Z"
      />
      <path fill="#fff" d="M112.6,41.6 113.6,42.6 112.6,43.6 111.6,42.6Z M47,31 48.4,32.4 47,33.8 45.6,32.4Z" />
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
    ask = canDrop ? t('Drop 1 or 2 coins, then pass the bank.') : t('{name} is dropping coins…', { name: holder ? nameOf(holder.id) : '' });
  else if (v.phase === 'claim')
    ask = canClaim
      ? t('How many {colour} coins are inside?', { colour: t(v.colour) })
      : t('{name} is peeking…', { name: holder ? nameOf(holder.id) : '' });
  else if (v.phase === 'doubt' && lastClaim)
    ask = canDoubt
      ? t('{name} says {n} {colour}. Doubt it?', { name: nameOf(lastClaim.id), n: lastClaim.count, colour: t(v.colour) })
      : v.myChoice !== null
        ? v.myChoice
          ? t('You doubt it. Waiting…')
          : t('You trust it. Waiting…')
        : t('Waiting for the others to judge your claim…');
  else if (showResult) {
    const right = last.claim === last.truth;
    const verdict = right ? t('right') : t('wrong');
    const lost = last.losers.length
      ? last.losers.length === 1 && last.losers[0] !== 'you'
        ? t('{names} loses a point.', { names: last.losers.map(nameOf).join(', ') })
        : t('{names} lose a point.', { names: last.losers.map(nameOf).join(', ') })
      : t('Nobody loses.');
    ask = `${last.full ? `${t('Full bank!')} ` : ''}${t('{name} said {claim}, there were {truth}: {verdict}.', { name: nameOf(last.claimer), claim: last.claim, truth: last.truth, verdict })} ${lost}`;
  }
  return (
    <Play
      top={<GameTop title="Piggy Bank" step={t('Round {n} of {total}', { n: v.round, total: v.rounds })} score={me?.score ?? 0} />}
      stage={
        <div className="card">
          <div className="piggy-bank-top">
            <Pig coins={showResult ? last.white + last.black : v.coins} open={!!showResult} />
            <div className="piggy-bank-info">
              <div className="label">{t('This round counts')}</div>
              <div>
                <Coin c={v.colour} /> {t(v.colour)}
              </div>
              {showResult ? (
                <div className="small">
                  {t('Inside:')} {last.white} <Coin c="white" /> {last.black} <Coin c="black" />
                </div>
              ) : (
                <div className="muted small">{t('Last accepted claim: {n}', { n: v.accepted })}</div>
              )}
              <div className="muted small">
                {t('Your coins: {n} {colour}, {other} other', { n: v.mine[0], colour: t(v.colour), other: v.mine[1] })}
              </div>
            </div>
          </div>
          <ul className="list">
            {v.seats.map((s) => (
              <li key={s.id} className={s.holder ? 'piggy-bank-holder' : ''}>
                <span>
                  {nameOf(s.id)} <span className="score">{s.score}</span>{' '}
                  {v.phase === 'doubt' && (
                    <span className="muted small">{s.id === lastClaim?.id ? t('claimer') : s.done ? t('decided') : t('thinking…')}</span>
                  )}
                  {showResult && last.doubters.includes(s.id) && <span className="muted small">{t('doubted')}</span>}
                </span>
                <span className="muted small">
                  {v.drops
                    .filter(([id]) => id === s.id)
                    .map(([, n]) => n)
                    .join(' + ') || ''}
                  {v.drops.some(([id]) => id === s.id) ? ` ${t('dropped')}` : ''}
                </span>
              </li>
            ))}
          </ul>
          {v.claims.length > 0 && !showResult && (
            <p className="muted small">
              {t('Claims:')}{' '}
              {v.claims.map((c) => t('{name} {n} of {total}', { name: nameOf(c.id), n: c.count, total: c.coins })).join(' · ')}
            </p>
          )}
        </div>
      }
      controls={
        <>
          <p className="ask">{ask}</p>
          {canDrop && (
            <>
              <div className="piggy-bank-pending">
                {pending.length === 0 && <span className="muted small">{t('No coins yet')}</span>}
                {pending.map((c, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: at most two coins
                  <Coin key={i} c={c} />
                ))}
              </div>
              <div className="row">
                <button type="button" className="btn" disabled={pending.length >= 2} onClick={() => add('white')}>
                  {t('+ White')}
                </button>
                <button type="button" className="btn" disabled={pending.length >= 2} onClick={() => add('black')}>
                  {t('+ Black')}
                </button>
                <button type="button" className="btn ghost" disabled={pending.length === 0} onClick={() => setPending([])}>
                  {t('Clear')}
                </button>
              </div>
              <button type="button" className="btn primary" disabled={pending.length === 0} onClick={drop}>
                {t('Drop and pass')}
              </button>
            </>
          )}
          {canClaim && (
            <>
              {v.seen && (
                <p className="center">
                  {t('You see the last drop:')} {v.seen[0] > 0 && `${v.seen[0]} `}
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
                {t('Claim {n} {colour}', { n: value, colour: t(v.colour) })}
              </button>
            </>
          )}
          {canDoubt && (
            <div className="row">
              <button type="button" className="btn" onClick={() => move({ type: 'trust' })}>
                {t('Trust')}
              </button>
              <button type="button" className="btn primary" onClick={() => move({ type: 'doubt' })}>
                {t('Doubt!')}
              </button>
            </div>
          )}
          <KeyHint>{t('W / B add a coin, Enter drops · digits and Enter claim · D doubt, T trust.')}</KeyHint>
        </>
      }
    />
  );
}
