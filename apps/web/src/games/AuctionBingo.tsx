// Auction Bingo. Rules page: docs/games/auction-bingo.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './AuctionBingo.css';

interface Board {
  shapes: number[];
  marked: boolean[];
}
interface BingoView {
  players: string[];
  me: number;
  round: number;
  rounds: number;
  phase: 'pick' | 'bid' | 'over';
  board: Board;
  coins: number;
  myPick: number | null;
  myBid: number | null;
  counts: [number, number, number] | null;
  done: boolean[];
  timeLeftMs: number;
  phaseMs: number;
  score: number;
  log: string[];
  result: { boards: Board[]; coins: number[]; scores: number[] } | null;
}

const NAMES = ['circle', 'triangle', 'star'];

function Shape({ shape }: { shape: number }) {
  return (
    <svg viewBox="0 0 40 40" className={`ab-shape ab-s${shape}`} aria-label={NAMES[shape]}>
      {shape === 0 && <circle cx="20" cy="20" r="13" />}
      {shape === 1 && <polygon points="20,6 34,32 6,32" />}
      {shape === 2 && <polygon points="20,4 24.5,15 36,15.5 27,23 30,34.5 20,28 10,34.5 13,23 4,15.5 15.5,15" />}
    </svg>
  );
}

function Grid({ board, pick, onCell }: { board: Board; pick?: number | null; onCell?: (c: number) => void }) {
  return (
    <div className={`ab-grid${onCell ? '' : ' small'}`}>
      {board.shapes.map((shape, c) => (
        <button
          // biome-ignore lint/suspicious/noArrayIndexKey: fixed 3×3 board
          key={c}
          type="button"
          className={`ab-cell${board.marked[c] ? ' marked' : ''}${pick === c ? ' picked' : ''}`}
          disabled={!onCell || board.marked[c]}
          onClick={() => onCell?.(c)}
        >
          <Shape shape={shape} />
          {onCell && <em>{c + 1}</em>}
        </button>
      ))}
    </div>
  );
}

function BidBox({ max, round, onBid }: { max: number; round: number; onBid: (n: number) => void }) {
  const [value, setValue] = useState(0);
  const [forRound, setForRound] = useState(round);
  if (forRound !== round) {
    setForRound(round);
    setValue(0);
  }
  const set = (n: number) => setValue(Math.max(0, Math.min(max, n)));
  useKeys((key) => {
    if (/^[0-9]$/.test(key)) set(Number(`${value === 0 ? '' : value}${key}`));
    else if (key === 'Backspace') set(Math.floor(value / 10));
    else if (key === 'Enter') onBid(value);
    else return false;
    return true;
  });
  return (
    <div className="ab-bid">
      <div className="row">
        <button type="button" className="btn" onClick={() => set(value - 1)}>
          −
        </button>
        <div className="pad-display">{value}</div>
        <button type="button" className="btn" onClick={() => set(value + 1)}>
          +
        </button>
      </div>
      <div className="row">
        {[0, 2, 5, max].map((n, i) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: fixed quick bids
          <button key={i} type="button" className="btn small" onClick={() => set(n)}>
            {i === 3 ? 'All' : n}
          </button>
        ))}
      </div>
      <button type="button" className="btn primary" onClick={() => onBid(value)}>
        Bid {value} coin{value === 1 ? '' : 's'}
      </button>
    </div>
  );
}

export default function AuctionBingo({ view: raw }: ScreenProps) {
  const view = raw as BingoView;
  const picking = view.phase === 'pick' && view.myPick === null;
  const bidding = view.phase === 'bid' && view.myBid === null;
  const pickCell = (c: number) => move({ type: 'pick', cell: c });
  useKeys((key) => {
    if (!picking || !/^[1-9]$/.test(key)) return false;
    pickCell(Number(key) - 1);
    return true;
  });
  const myShape = view.myPick !== null ? (view.board.shapes[view.myPick] ?? 0) : null;
  const waiting = view.players.filter((_, i) => !view.done[i]).map(nameOf);
  return (
    <Play
      top={
        <>
          <GameTop
            title="Auction Bingo"
            step={view.phase === 'over' ? 'Game over' : `Round ${view.round} of ${view.rounds} · ${view.phase}`}
            score={view.score}
          />
          {view.phase !== 'over' && <TimerBar leftMs={view.timeLeftMs} totalMs={view.phaseMs} />}
        </>
      }
      stage={
        <>
          <Grid board={view.board} pick={view.myPick} onCell={picking ? pickCell : undefined} />
          <p className="muted small center">
            Coins: <strong className="score">{view.coins}</strong> · 5 per line, 1 per cell, 1 per 5 coins left
          </p>
        </>
      }
      controls={
        <>
          {picking && <p className="ask">Pick an empty cell: you go for its shape.</p>}
          {view.counts && (
            <div className="ab-counts">
              {view.counts.map((n, shape) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: three shapes
                <div key={shape} className={shape === myShape ? 'mine' : ''}>
                  <Shape shape={shape} />
                  <span>
                    {n} picked{shape === myShape ? ' (you)' : ''}
                  </span>
                </div>
              ))}
            </div>
          )}
          {bidding && <BidBox max={view.coins} round={view.round} onBid={(coins) => move({ type: 'bid', coins })} />}
          {view.phase !== 'over' && !picking && !bidding && (
            <p className="muted center">Waiting for {waiting.join(', ') || 'the others'}…</p>
          )}
          {view.result && (
            <div className="card">
              <div className="label">Final boards</div>
              <div className="ab-finals">
                {view.players.map((p, i) => (
                  <div key={p} className="ab-final">
                    <Grid board={view.result?.boards[i] as Board} />
                    <span>
                      {nameOf(p)}: <strong className="score">{view.result?.scores[i]}</strong>
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
          <ul className="ab-log small">
            {view.log.map((l, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows
              <li key={i} className={l.startsWith('You') ? 'own' : ''}>
                {l}
              </li>
            ))}
          </ul>
          <KeyHint>Keys: 1–9 pick a cell; type a bid and press Enter.</KeyHint>
        </>
      }
    />
  );
}
