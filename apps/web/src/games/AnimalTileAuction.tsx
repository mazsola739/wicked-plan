// Animal Tile Auction. Rules page: docs/games/animal-tile-auction.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './AnimalTileAuction.css';

interface Part {
  x: number;
  y: number;
  animal: number;
}
interface TileView {
  players: string[];
  me: number;
  auction: number;
  auctions: number;
  phase: 'bid' | 'place' | 'over';
  grid: (number | null)[];
  tile: number | null;
  pattern: Part[];
  chips: number[];
  myBid: number | null;
  bidDone: boolean[];
  placer: number | null;
  timeLeftMs: number;
  phaseMs: number;
  patternScore: number;
  log: string[];
  result: { patterns: Part[][]; scores: number[] } | null;
}

const ANIMALS = ['fox', 'owl', 'frog', 'bear'];
const said = (line: string) => line.replace(/\bbot-\d+\b|\byou\b/g, nameOf);

/** A small face per animal, drawn in SVG. */
function Animal({ animal }: { animal: number }) {
  return (
    <svg viewBox="0 0 40 40" className={`ata-animal ata-a${animal}`} role="img" aria-label={ANIMALS[animal]}>
      {animal === 0 && (
        <>
          <polygon points="7,6 16,14 9,20" />
          <polygon points="33,6 24,14 31,20" />
          <polygon points="8,16 32,16 20,34" />
        </>
      )}
      {animal === 1 && (
        <>
          <ellipse cx="20" cy="22" rx="13" ry="14" />
          <circle cx="14" cy="19" r="5" className="eye" />
          <circle cx="26" cy="19" r="5" className="eye" />
          <polygon points="18,24 22,24 20,29" className="eye" />
        </>
      )}
      {animal === 2 && (
        <>
          <ellipse cx="20" cy="25" rx="15" ry="10" />
          <circle cx="12" cy="14" r="5" />
          <circle cx="28" cy="14" r="5" />
          <circle cx="12" cy="14" r="2" className="eye" />
          <circle cx="28" cy="14" r="2" className="eye" />
        </>
      )}
      {animal === 3 && (
        <>
          <circle cx="10" cy="10" r="6" />
          <circle cx="30" cy="10" r="6" />
          <circle cx="20" cy="22" r="14" />
          <ellipse cx="20" cy="27" rx="5" ry="4" className="eye" />
        </>
      )}
    </svg>
  );
}

function PatternCard({ parts }: { parts: Part[] }) {
  const w = Math.max(...parts.map((p) => p.x)) + 1;
  const h = Math.max(...parts.map((p) => p.y)) + 1;
  return (
    <div className="ata-pattern" style={{ gridTemplateColumns: `repeat(${w}, 32px)` }}>
      {Array.from({ length: w * h }, (_, i) => {
        const p = parts.find((q) => q.y * w + q.x === i);
        // biome-ignore lint/suspicious/noArrayIndexKey: a fixed tiny grid
        return <span key={i}>{p && <Animal animal={p.animal} />}</span>;
      })}
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
    <div className="ata-bid">
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
        {[0, 1, 3, 5].map((n) => (
          <button key={n} type="button" className="btn small" disabled={n > max} onClick={() => set(n)}>
            {n}
          </button>
        ))}
      </div>
      <button type="button" className="btn primary" onClick={() => onBid(value)}>
        Bid {value} chip{value === 1 ? '' : 's'}
      </button>
    </div>
  );
}

export default function AnimalTileAuction({ view: raw }: ScreenProps) {
  const view = raw as TileView;
  const placing = view.phase === 'place' && view.placer === view.me;
  const bidding = view.phase === 'bid' && view.myBid === null;
  const [cursor, setCursor] = useState(12);
  const place = (cell: number) => placing && view.grid[cell] === null && move({ type: 'place', cell });
  useKeys((key) => {
    if (!placing) return false;
    const x = cursor % 5;
    const y = Math.floor(cursor / 5);
    if (key === 'ArrowLeft') setCursor(y * 5 + Math.max(0, x - 1));
    else if (key === 'ArrowRight') setCursor(y * 5 + Math.min(4, x + 1));
    else if (key === 'ArrowUp') setCursor(Math.max(0, y - 1) * 5 + x);
    else if (key === 'ArrowDown') setCursor(Math.min(4, y + 1) * 5 + x);
    else if (key === 'Enter') place(cursor);
    else return false;
    return true;
  });
  const waiting = view.players.filter((_, i) => !view.bidDone[i]).map(nameOf);
  const status =
    view.phase === 'over'
      ? 'Game over: every pattern is scored.'
      : placing
        ? 'You won the tile: tap an empty cell to place it.'
        : view.phase === 'place'
          ? `${nameOf(view.players[view.placer ?? 0] ?? '')} is placing the tile…`
          : bidding
            ? 'Bid for this tile.'
            : `Waiting for ${waiting.join(', ') || 'the others'}…`;
  return (
    <Play
      top={
        <>
          <GameTop
            title="Animal Tile Auction"
            step={view.phase === 'over' ? 'Game over' : `Auction ${view.auction} of ${view.auctions}`}
            score={view.patternScore + (view.chips[view.me] ?? 0)}
          />
          {view.phase !== 'over' && <TimerBar leftMs={view.timeLeftMs} totalMs={view.phaseMs} />}
        </>
      }
      stage={
        <div className={`ata-grid${placing ? ' placing' : ''}`}>
          {view.grid.map((a, c) => (
            <button
              // biome-ignore lint/suspicious/noArrayIndexKey: the fixed 5×5 grid
              key={c}
              type="button"
              className={`ata-cell${placing && cursor === c ? ' cursor' : ''}`}
              disabled={!placing || a !== null}
              onClick={() => place(c)}
              aria-label={`${String.fromCharCode(65 + (c % 5))}${Math.floor(c / 5) + 1}${a === null ? '' : ` ${ANIMALS[a]}`}`}
            >
              {a !== null && <Animal animal={a} />}
            </button>
          ))}
        </div>
      }
      controls={
        <>
          <div className="ata-top">
            {view.tile !== null && (
              <div className="ata-offer">
                <span className="label">On offer</span>
                <span className="ata-cell big">
                  <Animal animal={view.tile} />
                </span>
                <span className="small">{ANIMALS[view.tile]}</span>
              </div>
            )}
            <div className="ata-offer">
              <span className="label">Your pattern</span>
              <PatternCard parts={view.pattern} />
              <span className="small muted">any rotation</span>
            </div>
          </div>
          <p className="ask">{status}</p>
          {bidding && <BidBox max={view.chips[view.me] ?? 0} round={view.auction} onBid={(chips) => move({ type: 'bid', chips })} />}
          <ul className="ata-chips small">
            {view.players.map((p, i) => (
              <li key={p} className={i === view.me ? 'me' : ''}>
                {nameOf(p)} <strong className="score">{view.chips[i]}</strong>
                {view.phase === 'bid' && view.bidDone[i] ? ' ✓' : ''}
              </li>
            ))}
          </ul>
          {view.result && (
            <div className="card">
              <div className="label">Patterns</div>
              <ul className="list">
                {view.players.map((p, i) => (
                  <li key={p}>
                    <span>{nameOf(p)}</span>
                    <PatternCard parts={view.result?.patterns[i] ?? []} />
                    <strong className="score">{view.result?.scores[i]}</strong>
                  </li>
                ))}
              </ul>
            </div>
          )}
          <ul className="ata-log small">
            {view.log.map((l, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows
              <li key={i}>{said(l)}</li>
            ))}
          </ul>
          <KeyHint>Keys: digits + Enter bid; arrows + Enter place.</KeyHint>
        </>
      }
    />
  );
}
