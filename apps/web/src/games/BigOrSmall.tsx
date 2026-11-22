// Big or Small: play a card in secret, bet on the opponent's card. Rules page: docs/games/big-or-small.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './BigOrSmall.css';

type Guess = 'bigger' | 'smaller' | 'equal';

interface Side {
  card: number;
  guess: Guess;
  stake: number;
  right: boolean;
  change: number;
}

interface BigView {
  round: number;
  rounds: number;
  dealer: number;
  hand: number[];
  theirHand: number[];
  chips: number;
  theirChips: number;
  opponent: string;
  chosen: boolean;
  theyChose: boolean;
  maxStake: number;
  timeLeftMs: number;
  turnMs: number;
  history: { dealer: number; mine: Side; theirs: Side }[];
  over: boolean;
}

const GUESSES: { id: Guess; label: string; key: string }[] = [
  { id: 'smaller', label: 'Smaller', key: 's' },
  { id: 'equal', label: 'Equal ×2', key: 'e' },
  { id: 'bigger', label: 'Bigger', key: 'b' },
];

const SIGN: Record<Guess, string> = { bigger: '>', smaller: '<', equal: '=' };

export default function BigOrSmall({ view: raw }: ScreenProps) {
  const view = raw as BigView;
  const [card, setCard] = useState<number | null>(null);
  const [guess, setGuess] = useState<Guess | null>(null);
  const [stake, setStake] = useState(1);
  const [forRound, setForRound] = useState(view.round);
  if (forRound !== view.round) {
    setForRound(view.round);
    setCard(null);
    setGuess(null);
  }
  const most = Math.min(view.maxStake, view.chips);
  const least = view.chips > 0 ? 1 : 0;
  const bet = Math.max(least, Math.min(most, stake));
  const them = nameOf(view.opponent);
  const ready = card !== null && guess !== null && !view.chosen;
  const submit = () => ready && move({ type: 'play', card, guess, stake: bet });
  const last = view.history[view.history.length - 1];
  const played = new Set(view.history.map((h) => h.theirs.card));

  useKeys((key) => {
    const n = Number(key);
    const g = GUESSES.find((x) => x.key === key.toLowerCase());
    if (n >= 1 && n <= 9 && view.hand.includes(n)) setCard(n);
    else if (g) setGuess(g.id);
    else if (key === '+' || key === 'ArrowUp') setStake(Math.min(most, bet + 1));
    else if (key === '-' || key === 'ArrowDown') setStake(Math.max(least, bet - 1));
    else if (key === 'Enter') submit();
    else return false;
    return true;
  });

  return (
    <Play
      top={
        <>
          <GameTop title="Big or Small" step={`Round ${view.round} of ${view.rounds}`} score={view.chips} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.turnMs} />
        </>
      }
      stage={
        <div className="big-or-small-table">
          <div className="big-or-small-chips">
            <span>
              You <strong>{view.chips}</strong>
            </span>
            <span>
              {them} <strong>{view.theirChips}</strong>
            </span>
          </div>
          <div className="big-or-small-dealer">
            <div className="label">Dealer</div>
            <div className="big-or-small-card">{view.dealer}</div>
          </div>
          <div>
            <div className="label">{them}&apos;s cards left</div>
            <div className="big-or-small-mini">
              {[1, 2, 3, 4, 5, 6, 7, 8, 9].map((c) => (
                <span key={c} className={played.has(c) ? 'gone' : ''}>
                  {c}
                </span>
              ))}
            </div>
          </div>
          {last && (
            <p className="last">
              Last round (dealer {last.dealer}): {them} played <strong>{last.theirs.card}</strong>, your bet {SIGN[last.mine.guess]}{' '}
              <span className={last.mine.right ? 'big-or-small-ok' : 'big-or-small-bad'}>{fmt(last.mine.change)}</span>
              {' · '}you played <strong>{last.mine.card}</strong>, {them} bet {SIGN[last.theirs.guess]}{' '}
              <span className={last.theirs.right ? 'big-or-small-ok' : 'big-or-small-bad'}>{fmt(last.theirs.change)}</span>
            </p>
          )}
        </div>
      }
      controls={
        view.chosen ? (
          <p className="ask">Waiting for {them}…</p>
        ) : (
          <>
            <div className="label">Your card {view.theyChose && <span className="muted">· {them} has chosen</span>}</div>
            <div className="cards">
              {view.hand.map((c) => (
                <button key={c} type="button" className={`card-btn${card === c ? ' picked' : ''}`} onClick={() => setCard(c)}>
                  {c}
                </button>
              ))}
            </div>
            <div className="label">
              {them}&apos;s card compared with {view.dealer}
            </div>
            <div className="row">
              {GUESSES.map((g) => (
                <button key={g.id} type="button" className={`btn${guess === g.id ? ' primary' : ''}`} onClick={() => setGuess(g.id)}>
                  {g.label}
                </button>
              ))}
            </div>
            <div className="label">Stake</div>
            <div className="row">
              {Array.from({ length: view.maxStake + 1 }, (_, i) => i)
                .filter((n) => n >= least && n <= most)
                .map((n) => (
                  <button key={n} type="button" className={`btn${bet === n ? ' primary' : ''}`} onClick={() => setStake(n)}>
                    {n} chip{n === 1 ? '' : 's'}
                  </button>
                ))}
            </div>
            <button type="button" className="btn primary" disabled={!ready} onClick={submit}>
              Play
            </button>
            <KeyHint>Keys: 1–9 card, S / E / B bet, ↑ ↓ stake, Enter plays.</KeyHint>
          </>
        )
      }
    />
  );
}

function fmt(n: number): string {
  return n > 0 ? `+${n}` : String(n);
}
