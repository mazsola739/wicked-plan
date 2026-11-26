// Sniper Hold'em: poker where a snipe knocks out every hand of one rank. Rules: docs/games/sniper-holdem.md

import { t } from '../i18n';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './SniperHoldem.css';

type Phase = 'betting' | 'snipe' | 'result' | 'over';
type Rank = 'highCard' | 'pair' | 'twoPair' | 'trips' | 'straight' | 'flush' | 'fullHouse' | 'quads' | 'straightFlush';
type Action = 'ante' | 'check' | 'call' | 'raise' | 'fold';

interface Shown {
  id: string;
  cards: number[];
  rank: Rank;
  sniped: boolean;
}

interface ShView {
  phase: Phase;
  hand: number;
  hands: number;
  cards: number[];
  community: number[];
  myRank: Rank | null;
  pot: number;
  carry: number;
  toCall: number;
  canRaise: boolean;
  myTurn: boolean;
  snipeChosen: boolean;
  mySnipe: Rank | null;
  seats: {
    id: string;
    chips: number;
    bet: number;
    inHand: boolean;
    last: Action | null;
    dealer: boolean;
    toAct: boolean;
    snipedYet: boolean;
  }[];
  log: {
    hand: number;
    community: number[];
    pot: number;
    winners: string[];
    carried: boolean;
    snipes: [string, Rank | null][];
    shown: Shown[];
  }[];
}

const RANKS: [Rank, string][] = [
  ['highCard', 'High card'],
  ['pair', 'Pair'],
  ['twoPair', 'Two pair'],
  ['trips', 'Three of a kind'],
  ['straight', 'Straight'],
  ['flush', 'Flush'],
  ['fullHouse', 'Full house'],
  ['quads', 'Four of a kind'],
  ['straightFlush', 'Straight flush'],
];
const rankName = (r: Rank) => t(RANKS.find(([k]) => k === r)?.[1] ?? r);
const COLOURS = ['red', 'blue', 'green', 'gold'];
const ACTIONS: Record<Action, string> = { ante: 'ante', check: 'check', call: 'call', raise: 'raise', fold: 'fold' };

/**
 * The faceted suit mark in the card's colour (red circle, blue square, green triangle, gold
 * diamond), cut like a gem: a flat table in a bevel lit from the top left, a white glint.
 */
function Suit({ suit }: { suit: number }) {
  return (
    <svg viewBox="0 0 20 20" role="img" aria-label={t(COLOURS[suit] ?? '')}>
      {suit === 0 && (
        <>
          <path d="M6.4,1.2 13.6,1.2 18.8,6.4 18.8,13.6 13.6,18.8 6.4,18.8 1.2,13.6 1.2,6.4Z" />
          <path fill="#fff" fillOpacity=".5" d="M1.2,6.4 6.4,1.2 7.6,4.2 4.2,7.6Z" />
          <path fill="#fff" fillOpacity=".32" d="M6.4,1.2 13.6,1.2 12.4,4.2 7.6,4.2Z" />
          <path fill="#fff" fillOpacity=".18" d="M1.2,6.4 4.2,7.6 4.2,12.4 1.2,13.6Z M13.6,1.2 18.8,6.4 15.8,7.6 12.4,4.2Z" />
          <path fill="#fff" fillOpacity=".12" d="M4.2,7.6 7.6,4.2 12.4,4.2 15.8,7.6 4.2,12.4Z" />
          <path fill="#000" fillOpacity=".15" d="M18.8,6.4 18.8,13.6 15.8,12.4 15.8,7.6Z M1.2,13.6 4.2,12.4 7.6,15.8 6.4,18.8Z" />
          <path fill="#000" fillOpacity=".25" d="M13.6,18.8 6.4,18.8 7.6,15.8 12.4,15.8Z" />
          <path fill="#000" fillOpacity=".35" d="M18.8,13.6 13.6,18.8 12.4,15.8 15.8,12.4Z" />
          <path fill="#fff" d="M6.4,5.4 7.4,6.4 6.4,7.4 5.4,6.4Z" />
        </>
      )}
      {suit === 1 && (
        <>
          <path d="M2,2 18,2 18,18 2,18Z" />
          <path fill="#fff" fillOpacity=".4" d="M2,2 18,2 14,6 6,6Z" />
          <path fill="#fff" fillOpacity=".2" d="M2,2 6,6 6,14 2,18Z" />
          <path fill="#fff" fillOpacity=".12" d="M6,6 14,6 6,14Z" />
          <path fill="#000" fillOpacity=".2" d="M18,2 18,18 14,14 14,6Z" />
          <path fill="#000" fillOpacity=".35" d="M2,18 6,14 14,14 18,18Z" />
          <path fill="#fff" d="M7.5,6.6 8.4,7.5 7.5,8.4 6.6,7.5Z" />
        </>
      )}
      {suit === 2 && (
        <>
          <path d="M10,1 19.5,18 0.5,18Z" />
          <path fill="#fff" fillOpacity=".42" d="M10,1 0.5,18 5.9,14.5 10,7.2Z" />
          <path fill="#fff" fillOpacity=".12" d="M10,7.2 10,14.5 5.9,14.5Z" />
          <path fill="#000" fillOpacity=".14" d="M10,1 19.5,18 14.1,14.5 10,7.2Z" />
          <path fill="#000" fillOpacity=".35" d="M19.5,18 0.5,18 5.9,14.5 14.1,14.5Z" />
          <path fill="#fff" d="M8.3,9.4 9.1,10.2 8.3,11 7.5,10.2Z" />
        </>
      )}
      {suit === 3 && (
        <>
          <path d="M10,0.5 19.5,10 10,19.5 0.5,10Z" />
          <path fill="#fff" fillOpacity=".45" d="M0.5,10 10,0.5 10,5.5 5.5,10Z" />
          <path fill="#fff" fillOpacity=".15" d="M10,0.5 19.5,10 14.5,10 10,5.5Z M5.5,10 10,5.5 10,14.5Z" />
          <path fill="#000" fillOpacity=".12" d="M10,19.5 0.5,10 5.5,10 10,14.5Z" />
          <path fill="#000" fillOpacity=".35" d="M19.5,10 10,19.5 10,14.5 14.5,10Z" />
          <path fill="#fff" d="M7.6,8.2 8.4,9 7.6,9.8 6.8,9Z" />
        </>
      )}
    </svg>
  );
}

function PlayingCard({ c, small }: { c: number; small?: boolean }) {
  return (
    <span className={`sniper-holdem-card ${COLOURS[Math.floor(c / 10)]}${small ? ' small' : ''}`}>
      {(c % 10) + 1}
      <Suit suit={Math.floor(c / 10)} />
    </span>
  );
}

export default function SniperHoldem({ view }: ScreenProps) {
  const v = view as ShView;
  const meIn = v.seats.find((s) => s.id === 'you')?.inHand ?? false;
  const canSnipe = v.phase === 'snipe' && meIn && !v.snipeChosen;
  const snipe = (rank: Rank | null) => move({ type: 'snipe', rank });
  useKeys((k) => {
    if (v.myTurn && (k === 'c' || k === 'C')) move({ type: 'call' });
    else if (v.myTurn && v.canRaise && (k === 'r' || k === 'R')) move({ type: 'raise' });
    else if (v.myTurn && (k === 'f' || k === 'F')) move({ type: 'fold' });
    else if (canSnipe && /^[0-9]$/.test(k)) snipe(k === '0' ? null : (RANKS[Number(k) - 1]?.[0] ?? null));
    else return false;
    return true;
  });
  const me = v.seats.find((s) => s.id === 'you');
  const last = v.log[v.log.length - 1];
  const showResult = (v.phase === 'result' || v.phase === 'over') && last;
  let ask = '';
  if (v.phase === 'betting') {
    const actor = v.seats.find((s) => s.toAct);
    ask = v.myTurn
      ? v.toCall > 0
        ? t('{n} to call.', { n: v.toCall })
        : t('Your turn.')
      : t('{name} is thinking…', { name: actor ? nameOf(actor.id) : '' });
  } else if (v.phase === 'snipe') {
    ask = !meIn
      ? t('You folded. Waiting for the snipes…')
      : canSnipe
        ? t('Snipe a hand rank, or none.')
        : t('Waiting for the other snipes…');
  } else if (last) {
    ask = last.carried
      ? t('Every hand was knocked out: {n} chips carry over.', { n: last.pot })
      : last.winners.length === 1 && last.winners[0] === 'you'
        ? t('You win {n} chips.', { n: last.pot })
        : last.winners.length === 1
          ? t('{name} wins {n} chips.', { name: nameOf(last.winners[0] ?? ''), n: last.pot })
          : t('{names} win {n} chips.', { names: last.winners.map(nameOf).join(t(' and ')), n: last.pot });
  }
  return (
    <Play
      top={<GameTop title="Sniper Hold'em" step={t('Hand {n} of {total}', { n: v.hand, total: v.hands })} score={me?.chips ?? 0} />}
      stage={
        <div className="card">
          <div className="sniper-holdem-table">
            <div className="sniper-holdem-row">
              {(showResult ? last.community : v.community).map((c) => (
                <PlayingCard key={c} c={c} />
              ))}
            </div>
            <div className="muted small center">
              {t('Pot')} <strong className="score">{showResult ? last.pot : v.pot}</strong>
              {v.carry > 0 && ` · ${t('{n} carried over', { n: v.carry })}`}
            </div>
          </div>
          {showResult ? (
            <ul className="list">
              {last.shown.length === 0 && <li className="muted small">{t('Everyone else folded.')}</li>}
              {last.shown.map((h) => (
                <li key={h.id} className={last.winners.includes(h.id) ? 'sniper-holdem-win' : ''}>
                  <span>
                    {nameOf(h.id)} <span className={h.sniped ? 'sniper-holdem-hit' : 'muted small'}>{rankName(h.rank)}</span>
                  </span>
                  <span className="sniper-holdem-row">
                    {h.cards.map((c) => (
                      <PlayingCard key={c} c={c} small />
                    ))}
                  </span>
                </li>
              ))}
              {last.snipes.some(([, r]) => r) && (
                <li className="muted small">
                  {t('Snipes:')}{' '}
                  {last.snipes
                    .filter(([, r]) => r)
                    .map(([id, r]) => `${nameOf(id)} → ${rankName(r as Rank)}`)
                    .join(', ')}
                </li>
              )}
            </ul>
          ) : (
            <ul className="list">
              {v.seats.map((s) => (
                <li key={s.id} className={s.toAct ? 'sniper-holdem-turn' : s.inHand ? '' : 'sniper-holdem-out'}>
                  <span>
                    {nameOf(s.id)}
                    {s.dealer && <span className="sniper-holdem-button">D</span>}{' '}
                    <span className="muted small">
                      {v.phase === 'snipe' && s.inHand ? (s.snipedYet ? t('chosen') : t('aiming…')) : s.last ? t(ACTIONS[s.last]) : ''}
                      {s.bet > 0 && ` · ${t('bet {n}', { n: s.bet })}`}
                    </span>
                  </span>
                  <span className="score">{s.chips}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      }
      controls={
        <>
          {v.cards.length > 0 && (
            <div className="sniper-holdem-mine">
              <div className="sniper-holdem-row">
                {v.cards.map((c) => (
                  <PlayingCard key={c} c={c} />
                ))}
              </div>
              {v.myRank && <span className="label">{t('You have: {rank}', { rank: rankName(v.myRank) })}</span>}
            </div>
          )}
          <p className="ask">{ask}</p>
          {v.phase === 'betting' && (
            <div className="row">
              <button type="button" className="btn" disabled={!v.myTurn} onClick={() => move({ type: 'fold' })}>
                {t('Fold')}
              </button>
              <button type="button" className="btn" disabled={!v.myTurn} onClick={() => move({ type: 'call' })}>
                {v.toCall > 0 ? t('Call {n}', { n: v.toCall }) : t('Check')}
              </button>
              <button type="button" className="btn primary" disabled={!v.myTurn || !v.canRaise} onClick={() => move({ type: 'raise' })}>
                {t('Raise +2')}
              </button>
            </div>
          )}
          {canSnipe && (
            <div className="sniper-holdem-snipes">
              {RANKS.map(([r, name]) => (
                <button key={r} type="button" className={`btn small${r === v.myRank ? ' sniper-holdem-own' : ''}`} onClick={() => snipe(r)}>
                  {t(name)}
                </button>
              ))}
              <button type="button" className="btn small ghost" onClick={() => snipe(null)}>
                {t('No snipe')}
              </button>
            </div>
          )}
          {v.phase === 'snipe' && v.snipeChosen && (
            <p className="last">{t('You snipe: {rank}.', { rank: v.mySnipe ? rankName(v.mySnipe) : t('nothing') })}</p>
          )}
          <KeyHint>{t('C check/call · R raise · F fold · snipe: 1–9 a rank, 0 none.')}</KeyHint>
        </>
      }
    />
  );
}
