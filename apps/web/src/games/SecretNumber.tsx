// Secret Number. Rules page: docs/games/secret-number.md

import { useState } from 'react';
import { GameTop, KeyHint, move, NumberPad, nameOf, Play, type ScreenProps, useKeys } from './common';
import './SecretNumber.css';

interface Seat {
  id: string;
  score: number;
  exposed: number | null;
  done: boolean;
  clues: string[];
  possible: number;
  low: number;
  high: number;
}
interface SecretView {
  me: number;
  number: number;
  round: number;
  rounds: number;
  phase: 'trade' | 'guess' | 'over';
  seats: Seat[];
  acted: boolean;
  log: string[];
  notes: string[];
  numbers: number[] | null;
  over: boolean;
}

const LETTERS = 'ABCDEFGH';
const said = (line: string) => line.replace(/\bbot-\d+\b|\byou\b/g, nameOf);

export default function SecretNumber({ view: raw }: ScreenProps) {
  const view = raw as SecretView;
  const [target, setTarget] = useState<number | null>(null);
  const open = !view.over && !view.acted;
  const canPick = (i: number) => open && i !== view.me && (view.phase === 'trade' || view.seats[i]?.exposed === null);
  const choose = (i: number) => {
    if (!canPick(i)) return;
    if (view.phase === 'trade') move({ type: 'trade', seat: i });
    else setTarget(i);
  };
  const guessing = view.phase === 'guess' && open && target !== null && canPick(target);
  useKeys((key) => {
    const k = key.toUpperCase();
    const seat = LETTERS.indexOf(k);
    if (key.length === 1 && seat >= 0 && seat < view.seats.length) choose(seat);
    else if (k === 'S' && open && view.phase === 'trade') move({ type: 'skip' });
    else if (k === 'P' && open && view.phase === 'guess') move({ type: 'pass' });
    else return false;
    return true;
  });
  const ask = view.over
    ? 'Game over: all numbers are shown.'
    : !open
      ? `Waiting for ${view.seats
          .filter((s) => !s.done)
          .map((s) => nameOf(s.id))
          .join(', ')}…`
      : view.phase === 'trade'
        ? 'Trade clues: tap a player (they learn about you too), or skip.'
        : guessing
          ? `Guess ${nameOf(view.seats[target]?.id ?? '')}'s number.`
          : 'Guess: tap a player to guess their number, or pass.';
  return (
    <Play
      top={
        <GameTop
          title="Secret Number"
          step={view.over ? 'Game over' : `Round ${view.round} of ${view.rounds} · ${view.phase}`}
          score={view.seats[view.me]?.score ?? 0}
        />
      }
      stage={
        <>
          <div className="sn-mine">
            <span className="label">Your secret number</span>
            <strong>{view.number}</strong>
            {view.seats[view.me]?.exposed !== null && <span className="sn-bad small">Exposed</span>}
          </div>
          <ul className="sn-seats">
            {view.seats.map((s, i) =>
              i === view.me ? null : (
                <li key={s.id}>
                  <button
                    type="button"
                    className={`sn-seat${target === i && view.phase === 'guess' ? ' on' : ''}${s.exposed !== null ? ' exposed' : ''}`}
                    disabled={!canPick(i)}
                    onClick={() => choose(i)}
                  >
                    <div className="sn-head">
                      <em>{LETTERS[i]}</em>
                      <span>{nameOf(s.id)}</span>
                      {s.done && !view.over && <span className="sn-ok">✓</span>}
                      <strong className="score">{s.score}</strong>
                    </div>
                    <div className="sn-know">
                      {s.exposed !== null || view.numbers ? (
                        <b>{s.exposed ?? view.numbers?.[i]}</b>
                      ) : (
                        <span>
                          {s.possible} possible ({s.low}–{s.high})
                        </span>
                      )}
                    </div>
                    {s.clues.length > 0 && <div className="sn-clues small">{s.clues.join(' · ')}</div>}
                  </button>
                </li>
              ),
            )}
          </ul>
        </>
      }
      controls={
        <>
          <p className="ask">{ask}</p>
          {open && view.phase === 'trade' && (
            <button type="button" className="btn" onClick={() => move({ type: 'skip' })}>
              Skip the trade
            </button>
          )}
          {guessing && target !== null && (
            <NumberPad round={view.round * 10 + target} onSubmit={(number) => move({ type: 'guess', seat: target, number })} />
          )}
          {open && view.phase === 'guess' && (
            <button type="button" className="btn" onClick={() => move({ type: 'pass' })}>
              Pass
            </button>
          )}
          {view.notes.length > 0 && (
            <div className="card sn-notes">
              <div className="label">What you learned</div>
              <ul className="small">
                {view.notes.map((l, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: the notes only grow
                  <li key={i}>{said(l)}</li>
                ))}
              </ul>
            </div>
          )}
          <ul className="sn-log small">
            {view.log.map((l, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows
              <li key={i}>{said(l)}</li>
            ))}
          </ul>
          <KeyHint>Keys: A–H pick a player, S skips, P passes, digits + Enter guess.</KeyHint>
        </>
      }
    />
  );
}
