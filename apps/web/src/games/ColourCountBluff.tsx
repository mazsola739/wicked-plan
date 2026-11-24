// Colour Count Bluff. Rules page: docs/games/colour-count-bluff.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './ColourCountBluff.css';

interface Bet {
  seat: number;
  count: number;
  colour: number;
}
interface Showdown {
  round: number;
  hands: number[][];
  bet: Bet;
  caller: number;
  actual: number;
  loser: number;
}
interface BluffView {
  players: string[];
  me: number;
  round: number;
  maxRounds: number;
  hand: number[];
  lives: number[];
  cards: number[];
  totalCards: number;
  bet: Bet | null;
  turn: number | null;
  log: string[];
  last: Showdown | null;
  over: boolean;
}

const COLOURS = ['red', 'blue', 'green', 'yellow'];
const KEYS = 'RBGY';
const said = (line: string) => line.replace(/\bbot-\d+\b|\byou\b/g, nameOf);

/** The smallest bet of this colour that beats the current one. */
function minCount(bet: Bet | null, colour: number): number {
  if (!bet) return 1;
  return colour > bet.colour ? bet.count : bet.count + 1;
}

/** A life: a faceted heart in the text colour, lit from the top left, with a white glint. */
function Heart() {
  return (
    <svg viewBox="0 0 40 40" className="ccb-heart" role="img" aria-label="life">
      <path d="M20,10 23.5,6 28,4 33.5,5.5 37,11 36,18 29,27 20,36 11,27 4,18 3,11 6.5,5.5 12,4 16.5,6Z" />
      <path fill="#fff" fillOpacity=".1" d="M3,11 12,12.5 4,18Z" />
      <path fill="#fff" fillOpacity=".3" d="M6.5,5.5 12,4 16.5,6 12,12.5Z" />
      <path fill="#fff" fillOpacity=".4" d="M3,11 6.5,5.5 12,12.5Z M33.5,5.5 28,4 23.5,6 28,12.5Z M23.5,6 20,10 20,21 28,12.5Z" />
      <path fill="#000" fillOpacity=".12" d="M4,18 12,12.5 20,21 11,27Z M11,27 20,21 20,36Z M37,11 33.5,5.5 28,12.5Z" />
      <path fill="#000" fillOpacity=".28" d="M37,11 28,12.5 36,18Z M29,27 20,21 20,36Z" />
      <path fill="#000" fillOpacity=".35" d="M36,18 28,12.5 20,21 29,27Z" />
      <path fill="#fff" d="M9.6,7.6 10.8,8.8 9.6,10 8.4,8.8Z" />
    </svg>
  );
}

function Cards({ hand, small }: { hand: number[]; small?: boolean }) {
  return (
    <div className={`ccb-hand${small ? ' small' : ''}`}>
      {hand.map((c, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: a hand of plain colours
        <span key={i} className={`ccb-card ccb-c${c}`}>
          {small ? '' : COLOURS[c]?.[0]?.toUpperCase()}
        </span>
      ))}
    </div>
  );
}

export default function ColourCountBluff({ view: raw }: ScreenProps) {
  const view = raw as BluffView;
  const mine = view.turn === view.me && !view.over;
  const [colour, setColour] = useState(0);
  const [count, setCount] = useState(1);
  const low = minCount(view.bet, colour);
  const shown = Math.max(count, low);
  const pick = (c: number) => {
    setColour(c);
    setCount(minCount(view.bet, c));
  };
  const bet = () => mine && shown <= view.totalCards && move({ type: 'bet', count: shown, colour });
  const call = () => mine && view.bet && move({ type: 'call' });
  useKeys((key) => {
    const k = key.toUpperCase();
    const c = KEYS.indexOf(k);
    if (c >= 0 && key.length === 1) pick(c);
    else if (key === '+' || key === '=' || key === 'ArrowUp') setCount(Math.min(view.totalCards, shown + 1));
    else if (key === '-' || key === 'ArrowDown') setCount(Math.max(low, shown - 1));
    else if (key === 'Enter') bet();
    else if (k === 'C') call();
    else return false;
    return true;
  });
  const status = view.over
    ? 'Game over.'
    : mine
      ? view.bet
        ? 'Your turn: raise the bet or call it.'
        : 'You open: make the first bet.'
      : `${nameOf(view.players[view.turn ?? 0] ?? '')} is thinking…`;
  return (
    <Play
      top={<GameTop title="Colour Count Bluff" step={`Round ${view.round} of ${view.maxRounds}`} score={view.lives[view.me] ?? 0} />}
      stage={
        <>
          <ul className="ccb-seats">
            {view.players.map((p, i) => (
              <li key={p} className={`${view.turn === i ? 'turn' : ''}${view.lives[i] === 0 ? ' out' : ''}${i === view.me ? ' me' : ''}`}>
                <span>{nameOf(p)}</span>
                <span className="ccb-lives" role="img" aria-label={`${view.lives[i]} lives`}>
                  {Array.from({ length: view.lives[i] ?? 0 }, (_, k) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: identical hearts
                    <Heart key={k} />
                  ))}
                </span>
                <span className="muted small">{view.cards[i]} cards</span>
              </li>
            ))}
          </ul>
          <div className="ccb-bet">
            {view.bet ? (
              <>
                <span className="muted small">{nameOf(view.players[view.bet.seat] ?? '')} bets at least</span>
                <strong>
                  {view.bet.count} <span className={`ccb-word ccb-c${view.bet.colour}`}>{COLOURS[view.bet.colour]}</span>
                </strong>
              </>
            ) : (
              <span className="muted">No bet yet this round</span>
            )}
            <span className="muted small">{view.totalCards} cards on the table</span>
          </div>
          <div className="ccb-mine">
            <div className="label">Your cards</div>
            <Cards hand={view.hand} />
          </div>
        </>
      }
      controls={
        <>
          <p className="ask">{status}</p>
          {mine && (
            <>
              <div className="ccb-colours">
                {COLOURS.map((name, c) => (
                  <button
                    key={name}
                    type="button"
                    className={`btn ccb-pick ccb-c${c}${c === colour ? ' on' : ''}`}
                    disabled={minCount(view.bet, c) > view.totalCards}
                    onClick={() => pick(c)}
                  >
                    {name}
                  </button>
                ))}
              </div>
              <div className="row">
                <button type="button" className="btn" disabled={shown <= low} onClick={() => setCount(shown - 1)}>
                  −
                </button>
                <div className="pad-display ccb-count">{shown}</div>
                <button type="button" className="btn" disabled={shown >= view.totalCards} onClick={() => setCount(shown + 1)}>
                  +
                </button>
              </div>
              <div className="row">
                <button type="button" className="btn primary" disabled={shown > view.totalCards} onClick={bet}>
                  Bet {shown} {COLOURS[colour]}
                </button>
                <button type="button" className="btn" disabled={!view.bet} onClick={call}>
                  Call
                </button>
              </div>
            </>
          )}
          {view.last && (
            <div className="card">
              <div className="label">
                Round {view.last.round}: {view.last.bet.count} {COLOURS[view.last.bet.colour]} bet, {view.last.actual} there
              </div>
              <ul className="list">
                {view.players.map((p, i) =>
                  view.last?.hands[i]?.length ? (
                    <li key={p}>
                      <span>
                        {nameOf(p)}
                        {view.last.loser === i && (
                          <>
                            {' '}
                            −<Heart />
                          </>
                        )}
                      </span>
                      <Cards hand={view.last.hands[i] ?? []} small />
                    </li>
                  ) : null,
                )}
              </ul>
            </div>
          )}
          <ul className="ccb-log small">
            {view.log.map((l, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows
              <li key={i}>{said(l)}</li>
            ))}
          </ul>
          <KeyHint>Keys: R/B/G/Y colour, + / − count, Enter bets, C calls.</KeyHint>
        </>
      }
    />
  );
}
