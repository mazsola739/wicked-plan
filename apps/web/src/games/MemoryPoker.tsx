// Memory Poker: study 16 cards, take two blind. Rules page: docs/games/memory-poker.md

import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './MemoryPoker.css';

interface Face {
  rank: number;
  suit: number;
}

interface MemoryPokerView {
  phase: 'study' | 'pick' | 'show';
  round: number;
  rounds: number;
  timeLeftMs: number;
  phaseMs: number;
  grid: { card: Face | null; takenBy: string | null }[];
  shared: Face[];
  seats: { id: string; ready: boolean; cards: (Face | null)[]; hand: string | null; points: number; roundPoints: number }[];
  you: number;
  turn: string | null;
  over: boolean;
}

const RANKS: Record<number, string> = { 11: 'J', 12: 'Q', 13: 'K', 14: 'A' };
const SUITS = ['♠', '♥', '♦', '♣'];

function Card({ face, small }: { face: Face | null; small?: boolean }) {
  if (!face) return <span className={`mp-card back${small ? ' small' : ''}`} />;
  return (
    <span className={`mp-card${face.suit === 1 || face.suit === 2 ? ' red' : ''}${small ? ' small' : ''}`}>
      <b>{RANKS[face.rank] ?? face.rank}</b>
      <i>{SUITS[face.suit]}</i>
    </span>
  );
}

export default function MemoryPoker({ view: raw }: ScreenProps) {
  const view = raw as MemoryPokerView;
  const me = view.seats[view.you];
  const myTurn = view.phase === 'pick' && view.turn === me?.id;
  const pick = (cell: number) => myTurn && !view.grid[cell]?.takenBy && move({ type: 'pick', cell });
  const primary = () => {
    if (view.phase === 'study' && !me?.ready) move({ type: 'ready' });
    else if (view.phase === 'show') move({ type: 'next' });
  };
  useKeys((key) => {
    const cell = key.toUpperCase().charCodeAt(0) - 65;
    if (key.length === 1 && cell >= 0 && cell < 16) pick(cell);
    else if (key === 'Enter') primary();
    else return false;
    return true;
  });
  const step = { study: 'Memorise the grid!', pick: 'Take two cards', show: 'Showdown' }[view.phase];
  const status =
    view.phase === 'study'
      ? me?.ready
        ? 'Waiting for the others…'
        : 'The cards turn face down when the time runs out.'
      : view.phase === 'pick'
        ? myTurn
          ? 'Your pick: tap a face-down card.'
          : `${nameOf(view.turn ?? '')} is picking…`
        : 'Two cards + the shared three make each hand.';
  return (
    <Play
      top={
        <>
          <GameTop title="Memory Poker" step={`Round ${view.round} of ${view.rounds} · ${step}`} score={me?.points ?? 0} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.phaseMs} />
        </>
      }
      stage={
        <div className="mp-table">
          <div className="mp-shared">
            <span className="label">Shared</span>
            {view.shared.map((f) => (
              <Card key={`${f.rank}-${f.suit}`} face={f} />
            ))}
          </div>
          <div className="mp-grid">
            {view.grid.map((g, i) => {
              const mine = g.takenBy === me?.id;
              const other = g.takenBy !== null && !mine;
              return (
                <button
                  // biome-ignore lint/suspicious/noArrayIndexKey: squares are positional
                  key={i}
                  type="button"
                  className={`mp-slot${mine ? ' mine' : ''}${other ? ' taken' : ''}${myTurn && !g.takenBy ? ' can' : ''}`}
                  disabled={!myTurn || g.takenBy !== null}
                  onClick={() => pick(i)}
                  aria-label={`Square ${String.fromCharCode(65 + i)}`}
                >
                  {other && view.phase === 'pick' ? <span className="mp-gone">{nameOf(g.takenBy ?? '')}</span> : <Card face={g.card} />}
                  {other && view.phase === 'show' && <span className="mp-tag">{nameOf(g.takenBy ?? '')}</span>}
                  <span className="mp-letter">{String.fromCharCode(65 + i)}</span>
                </button>
              );
            })}
          </div>
        </div>
      }
      controls={
        <>
          <p className="last">{status}</p>
          {(view.phase === 'study' || view.phase === 'show') && (
            <button type="button" className="btn primary" disabled={view.phase === 'study' && me?.ready} onClick={primary}>
              {view.phase === 'study' ? 'Ready' : view.round < view.rounds ? 'Next round' : 'Finish'}
            </button>
          )}
          <ul className="list mp-seats">
            {view.seats.map((s, i) => (
              <li key={s.id} className={i === view.you ? 'me' : ''}>
                <span className="mp-who">
                  {nameOf(s.id)}
                  {view.turn === s.id && <span className="muted small"> · picking</span>}
                </span>
                <span className="mp-hand">
                  {s.cards.map((c, j) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: two cards
                    <Card key={j} face={c} small />
                  ))}
                  {s.hand && <span className="muted small">{s.hand}</span>}
                </span>
                <span className="score">
                  {view.phase === 'show' && s.roundPoints > 0 && <small>+{s.roundPoints} </small>}
                  {s.points}
                </span>
              </li>
            ))}
          </ul>
          <p className="muted small center">
            Pair 50 · two pair 90 · three 130 · straight 180 · flush 200 · full house 250 · four 320 · straight flush 400 · best hand +60
          </p>
          <KeyHint>Keys: A–P pick a square, Enter is Ready / Next.</KeyHint>
        </>
      }
    />
  );
}
