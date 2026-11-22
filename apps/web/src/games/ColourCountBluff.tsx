// Colour Count Bluff. Rules page: docs/games/colour-count-bluff.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
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
  timeLeftMs: number;
  turnMs: number;
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
      top={
        <>
          <GameTop title="Colour Count Bluff" step={`Round ${view.round} of ${view.maxRounds}`} score={view.lives[view.me] ?? 0} />
          {!view.over && <TimerBar leftMs={view.timeLeftMs} totalMs={view.turnMs} />}
        </>
      }
      stage={
        <>
          <ul className="ccb-seats">
            {view.players.map((p, i) => (
              <li key={p} className={`${view.turn === i ? 'turn' : ''}${view.lives[i] === 0 ? ' out' : ''}${i === view.me ? ' me' : ''}`}>
                <span>{nameOf(p)}</span>
                <span className="ccb-lives" role="img" aria-label={`${view.lives[i]} lives`}>
                  {'♥'.repeat(view.lives[i] ?? 0)}
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
                        {view.last.loser === i ? ' −♥' : ''}
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
