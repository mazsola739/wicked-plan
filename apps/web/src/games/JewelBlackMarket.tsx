// Jewel Black Market. Rules page: docs/games/jewel-black-market.md

import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './JewelBlackMarket.css';

type Kind = 'diamond' | 'gold' | 'silver' | 'bronze' | 'bomb';
interface Card {
  kind: Kind;
  value: number;
}
interface Reveal {
  round: number;
  cards: Card[];
  buyer: Card[];
  discard: Card;
  points: number[];
}
interface JewelView {
  players: string[];
  me: number;
  round: number;
  rounds: number;
  swapRound: number;
  turn: number | null;
  myCard: Card;
  timeLeftMs: number;
  turnMs: number;
  scores: number[];
  log: string[];
  last: Reveal | null;
  over: boolean;
}

const LETTERS = 'ABCDEFGH';
const said = (line: string) => line.replace(/\bbot-\d+\b|\byou\b/g, nameOf);

function Gem({ card, small }: { card: Card; small?: boolean }) {
  return (
    <div className={`jbm-card jbm-${card.kind}${small ? ' small' : ''}`}>
      <svg viewBox="0 0 40 40" aria-hidden="true">
        {card.kind === 'bomb' ? (
          <>
            <circle cx="20" cy="23" r="12" />
            <path d="M27 13 l5 -6" className="fuse" />
          </>
        ) : (
          <polygon points="10,14 16,7 24,7 30,14 20,33" />
        )}
      </svg>
      <strong>{card.value}</strong>
      {!small && <span>{card.kind}</span>}
    </div>
  );
}

function Back({ label, onClick }: { label: string; onClick?: () => void }) {
  return (
    <button type="button" className="jbm-card jbm-back" disabled={!onClick} onClick={onClick}>
      <span>{label}</span>
    </button>
  );
}

export default function JewelBlackMarket({ view: raw }: ScreenProps) {
  const view = raw as JewelView;
  const mine = view.turn === view.me && !view.over;
  const bomb = view.myCard.kind === 'bomb';
  const canTable = mine && !bomb;
  useKeys((key) => {
    if (!mine) return false;
    const k = key.toUpperCase();
    const seat = LETTERS.indexOf(k);
    if (k === 'K' && !bomb) move({ type: 'keep' });
    else if (k === 'D' && !bomb) move({ type: 'swapDiscard' });
    else if (/^[1-3]$/.test(key) && !bomb) move({ type: 'swapBuyer', slot: Number(key) - 1 });
    else if (seat >= 0 && seat < view.players.length && seat !== view.me) move({ type: 'swapPlayer', seat });
    else return false;
    return true;
  });
  const status = view.over
    ? 'Game over.'
    : mine
      ? bomb
        ? 'You hold a bomb: pass it to a player.'
        : 'Your turn: keep, or swap with a player, the buyer or the discard.'
      : `${nameOf(view.players[view.turn ?? 0] ?? '')} is choosing…`;
  return (
    <Play
      top={
        <>
          <GameTop
            title="Jewel Black Market"
            step={`Round ${view.round} of ${view.rounds} · swap ${view.swapRound} of 2`}
            score={view.scores[view.me] ?? 0}
          />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.turnMs} />
        </>
      }
      stage={
        <div className="jbm-table">
          <div className="jbm-zone">
            <div className="label">Buyer's space</div>
            <div className="jbm-row">
              {[0, 1, 2].map((slot) => (
                <Back key={slot} label={`${slot + 1}`} onClick={canTable ? () => move({ type: 'swapBuyer', slot }) : undefined} />
              ))}
              <div className="jbm-gap" />
              <Back label="Discard" onClick={canTable ? () => move({ type: 'swapDiscard' }) : undefined} />
            </div>
          </div>
          <ul className="jbm-seats">
            {view.players.map((p, i) => (
              <li key={p}>
                <button
                  type="button"
                  className={`jbm-seat${view.turn === i ? ' turn' : ''}${i === view.me ? ' me' : ''}`}
                  disabled={!mine || i === view.me}
                  onClick={() => move({ type: 'swapPlayer', seat: i })}
                >
                  <em>{LETTERS[i]}</em>
                  <span>{nameOf(p)}</span>
                  <strong>{view.scores[i]}</strong>
                </button>
              </li>
            ))}
          </ul>
          <div className="jbm-mine">
            <div className="label">Your card</div>
            <Gem card={view.myCard} />
          </div>
        </div>
      }
      controls={
        <>
          <p className="ask">{status}</p>
          {mine && !bomb && (
            <button type="button" className="btn primary" onClick={() => move({ type: 'keep' })}>
              Keep my card
            </button>
          )}
          <p className="muted small center">
            A jewel scores its value × the same-type jewels in the buyer's space. A bomb costs its value.
          </p>
          <ul className="jbm-log small">
            {view.log.map((l, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows
              <li key={i}>{said(l)}</li>
            ))}
          </ul>
          {view.last && (
            <div className="card jbm-reveal">
              <div className="label">Round {view.last.round} revealed</div>
              <div className="jbm-row">
                <span className="muted small">Buyer</span>
                {view.last.buyer.map((c, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: fixed three slots
                  <Gem key={i} card={c} small />
                ))}
                <span className="muted small">Discard</span>
                <Gem card={view.last.discard} small />
              </div>
              <ul className="list">
                {view.players.map((p, i) => (
                  <li key={p}>
                    <span>{nameOf(p)}</span>
                    <Gem card={view.last?.cards[i] as Card} small />
                    <strong className="score">{(view.last?.points[i] ?? 0) > 0 ? `+${view.last?.points[i]}` : view.last?.points[i]}</strong>
                  </li>
                ))}
              </ul>
            </div>
          )}
          <KeyHint>Keys: K keeps, 1–3 buyer card, D discard, A–H a player.</KeyHint>
        </>
      }
    />
  );
}
