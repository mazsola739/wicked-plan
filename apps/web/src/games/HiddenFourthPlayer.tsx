// Hidden Fourth Player: three in a row for three, against the Shadow. Rules page: docs/games/hidden-fourth-player.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './HiddenFourthPlayer.css';

interface HiddenView {
  size: number;
  board: number[];
  me: number;
  players: string[];
  lines: number[];
  shadowLines: number;
  turn: number;
  yourTurn: boolean;
  danger: number[];
  last: [number, number][];
  round: number;
  timeLeftMs: number;
  turnMs: number;
  over: boolean;
  shadowWon: boolean;
}

/** Stone colours: index 1–3 the seats, 4 the Shadow. */
const COLOURS = ['', '#e8dcc4', '#3aa86b', '#4f8fd8', '#0b0910'];

export default function HiddenFourthPlayer({ view: raw }: ScreenProps) {
  const view = raw as HiddenView;
  const n = view.size;
  const [cursor, setCursor] = useState(Math.floor((n * n) / 2));
  const danger = new Set(view.danger);
  const lastAt = new Map(view.last.map(([c, sq]) => [sq, c]));
  const place = (c: number) => view.yourTurn && view.board[c] === 0 && move({ type: 'place', x: c % n, y: Math.floor(c / n) });

  // Desktop: arrows move a cursor, Enter places.
  useKeys((key) => {
    const x = cursor % n;
    const y = Math.floor(cursor / n);
    if (key === 'ArrowLeft') setCursor(y * n + Math.max(0, x - 1));
    else if (key === 'ArrowRight') setCursor(y * n + Math.min(n - 1, x + 1));
    else if (key === 'ArrowUp') setCursor(Math.max(0, y - 1) * n + x);
    else if (key === 'ArrowDown') setCursor(Math.min(n - 1, y + 1) * n + x);
    else if (key === 'Enter' || key === ' ') place(cursor);
    else return false;
    return true;
  });

  const status = view.yourTurn
    ? danger.size > 0
      ? `Your stone. The Shadow threatens ${danger.size} square${danger.size > 1 ? 's' : ''}!`
      : 'Your stone: build lines, keep the Shadow down.'
    : `${nameOf(view.players[view.turn] ?? '')} is placing…`;

  return (
    <Play
      top={
        <>
          <GameTop title="Hidden Fourth Player" step={`Round ${view.round}`} score={view.lines[view.me] ?? 0} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.turnMs} />
        </>
      }
      stage={
        <div className="hidden-fourth-player-board" style={{ gridTemplateColumns: `repeat(${n}, 1fr)` }}>
          {view.board
            .map((_, i) => i)
            .map((c) => {
              const v = view.board[c] ?? 0;
              const cls = [
                'hidden-fourth-player-sq',
                danger.has(c) ? 'danger' : '',
                lastAt.has(c) ? 'last' : '',
                cursor === c ? 'cursor' : '',
              ].join(' ');
              return (
                <button key={c} type="button" className={cls} disabled={!view.yourTurn || v !== 0} onClick={() => place(c)}>
                  {v !== 0 && (
                    <span className={`hidden-fourth-player-stone${v === 4 ? ' shadow' : ''}`} style={{ background: COLOURS[v] }} />
                  )}
                </button>
              );
            })}
        </div>
      }
      controls={
        <>
          <ul className="list">
            {view.players.map((p, i) => (
              <li key={p} className={i === view.turn ? 'hidden-fourth-player-turn' : ''}>
                <span>
                  <i className="hidden-fourth-player-dot" style={{ background: COLOURS[i + 1] }} /> {nameOf(p)}
                </span>
                <span className="score">
                  {view.lines[i]} line{view.lines[i] === 1 ? '' : 's'}
                </span>
              </li>
            ))}
            <li>
              <span>
                <i className="hidden-fourth-player-dot shadow" /> The Shadow
              </span>
              <span className="muted small">plays after the three of you</span>
            </li>
          </ul>
          <p className={`ask${danger.size > 0 ? ' hidden-fourth-player-alert' : ''}`}>{status}</p>
          <p className="muted small center">
            Lines score 100 each, and everyone gets +300 if the board fills up. If the Shadow makes three in a row, the game ends and lines
            are worth half.
          </p>
          <KeyHint>Arrow keys move, Enter places.</KeyHint>
        </>
      }
    />
  );
}
