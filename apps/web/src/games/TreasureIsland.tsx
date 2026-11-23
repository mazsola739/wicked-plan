// Treasure Island: bid in secret for clue boxes, then dig once.
// Rules page: docs/games/treasure-island.md

import { useState } from 'react';
import './TreasureIsland.css';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';

type Square = [number, number];

interface BoxView {
  x: number;
  y: number;
  auction: number | null;
  arrow: [number, number] | null;
  openedBy: number[];
}

interface Auction {
  bids: number[];
  winners: number[];
  price: number;
}

interface IslandView {
  players: string[];
  you: number | null;
  size: number;
  phase: 'auction' | 'dig' | 'over';
  round: number;
  auctions: number;
  boxes: BoxView[];
  possible: Square[];
  coins: number[];
  bidIn: boolean[];
  yourBid: number | null;
  history: Auction[];
  yourDig: Square | null;
  dug: boolean[];
  timeLeftMs: number;
  phaseMs: number;
  treasure: Square | null;
  digs: (Square | null)[] | null;
  scores: number[] | null;
}

const C = 40;

function Island({ view, pick, onPick }: { view: IslandView; pick: Square | null; onPick: (s: Square) => void }) {
  const n = view.size;
  const possible = new Set(view.possible.map(([x, y]) => `${x},${y}`));
  const cells = Array.from({ length: n * n }, (_, i) => ({ x: i % n, y: Math.floor(i / n), id: `c${i}` }));
  const digging = view.phase === 'dig' && view.yourDig === null;
  const current = view.phase === 'auction' ? view.round - 1 : -1;
  const dig = view.yourDig ?? pick;
  return (
    <svg className="drawing ti-map" viewBox={`-12 -12 ${n * C + 24} ${n * C + 24}`} role="img" aria-label="Island map">
      <defs>
        <marker id="ti-head" viewBox="0 0 10 10" refX="6" refY="5" markerWidth="4" markerHeight="4" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" className="ti-head" />
        </marker>
      </defs>
      <rect className="ti-sea" x={-12} y={-12} width={n * C + 24} height={n * C + 24} rx={12} />
      {cells.map((c) => (
        // biome-ignore lint/a11y/useSemanticElements: an SVG cell cannot be a <button>
        <rect
          key={c.id}
          className={`ti-cell${possible.has(`${c.x},${c.y}`) ? ' maybe' : ''}${digging ? ' can' : ''}`}
          x={c.x * C + 1}
          y={c.y * C + 1}
          width={C - 2}
          height={C - 2}
          rx={5}
          role="button"
          tabIndex={-1}
          onClick={() => digging && onPick([c.x, c.y])}
          onKeyDown={() => {}}
        />
      ))}
      {view.boxes.map((b) => {
        const cx = b.x * C + C / 2;
        const cy = b.y * C + C / 2;
        const len = 13 / Math.hypot(b.arrow?.[0] || 1, b.arrow?.[1] || 1);
        return (
          <g key={`b${b.auction ?? 'own'}`} className={`ti-box${b.auction === null ? ' own' : ''}${b.auction === current ? ' up' : ''}`}>
            <rect x={b.x * C + 5} y={b.y * C + 5} width={C - 10} height={C - 10} rx={4} />
            {b.arrow ? (
              <line
                className="ti-arrow"
                x1={cx - b.arrow[0] * len * 0.6}
                y1={cy - b.arrow[1] * len * 0.6}
                x2={cx + b.arrow[0] * len}
                y2={cy + b.arrow[1] * len}
                markerEnd="url(#ti-head)"
              />
            ) : (
              <text className="ti-label" x={cx} y={cy}>
                {b.auction === null ? '' : b.auction + 1}
              </text>
            )}
          </g>
        );
      })}
      {view.digs?.map((d, k) =>
        d ? (
          <text
            key={view.players[k]}
            className={`ti-dig${k === view.you ? ' me' : ''}`}
            style={{ fill: `var(--seat-${k})` }}
            x={d[0] * C + 8 + (k % 3) * 10}
            y={d[1] * C + 12 + Math.floor(k / 3) * 14}
          >
            {nameOf(view.players[k]!).slice(0, 1)}
          </text>
        ) : null,
      )}
      {dig && !view.digs && (
        <g className="ti-x" transform={`translate(${dig[0] * C + C / 2} ${dig[1] * C + C / 2})`}>
          <line x1={-11} y1={-11} x2={11} y2={11} />
          <line x1={11} y1={-11} x2={-11} y2={11} />
        </g>
      )}
      {view.treasure && (
        <circle className="ti-treasure" cx={view.treasure[0] * C + C / 2} cy={view.treasure[1] * C + C / 2} r={C / 2 - 2} />
      )}
    </svg>
  );
}

function BidControls({ view }: { view: IslandView }) {
  const you = view.you ?? 0;
  const coins = view.coins[you] ?? 0;
  const [bid, setBid] = useState(0);
  const clamp = (v: number) => Math.max(0, Math.min(coins, v));
  const sent = view.yourBid !== null;
  const send = () => !sent && move({ type: 'bid', amount: clamp(bid) });
  useKeys((key) => {
    if (sent) return false;
    if (key === 'ArrowUp') setBid((b) => clamp(b + 1));
    else if (key === 'ArrowDown') setBid((b) => clamp(b - 1));
    else if (key === 'PageUp') setBid((b) => clamp(b + 10));
    else if (key === 'PageDown') setBid((b) => clamp(b - 10));
    else if (key === 'Enter') send();
    else return false;
    return true;
  });
  return (
    <>
      <p className="ask">Box {view.round} is up. Your secret bid:</p>
      <div className="ti-bid">
        <button type="button" className="key" disabled={sent} onClick={() => setBid((b) => clamp(b - 10))}>
          −10
        </button>
        <button type="button" className="key" disabled={sent} onClick={() => setBid((b) => clamp(b - 1))}>
          −1
        </button>
        <strong>{sent ? view.yourBid : clamp(bid)}</strong>
        <button type="button" className="key" disabled={sent} onClick={() => setBid((b) => clamp(b + 1))}>
          +1
        </button>
        <button type="button" className="key" disabled={sent} onClick={() => setBid((b) => clamp(b + 10))}>
          +10
        </button>
      </div>
      <button type="button" className="btn primary" disabled={sent} onClick={send}>
        {sent ? 'Bid sent, waiting…' : `Bid ${clamp(bid)} coins`}
      </button>
      <KeyHint>Keys: ↑/↓ change the bid by 1, PgUp/PgDn by 10, Enter bids.</KeyHint>
    </>
  );
}

export default function TreasureIsland({ view: raw }: ScreenProps) {
  const view = raw as IslandView;
  const you = view.you ?? 0;
  const [pick, setPick] = useState<Square | null>(null);
  const who = (k: number) => (k === you ? 'You' : nameOf(view.players[k]!));
  const last = view.history[view.history.length - 1];
  const digNow = () => pick && view.yourDig === null && move({ type: 'dig', x: pick[0], y: pick[1] });
  useKeys((key) => {
    if (view.phase !== 'dig' || view.yourDig !== null) return false;
    const d = ({ ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] } as Record<string, Square>)[key];
    if (d) {
      const c = pick ?? [4, 4];
      setPick([Math.max(0, Math.min(view.size - 1, c[0] + d[0])), Math.max(0, Math.min(view.size - 1, c[1] + d[1]))]);
    } else if (key === 'Enter') digNow();
    else return false;
    return true;
  });
  return (
    <Play
      top={
        <>
          <GameTop
            title="Treasure Island"
            step={view.phase === 'auction' ? `Auction ${view.round} of ${view.auctions}` : 'Dig for the treasure'}
            score={view.coins[you] ?? 0}
          />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.phaseMs} />
        </>
      }
      stage={
        <>
          <Island view={view} pick={pick} onPick={setPick} />
          <p className="muted small center">Lit squares still fit your arrows ({view.possible.length}). Your private box has a cyan rim.</p>
        </>
      }
      controls={
        <>
          {view.phase === 'auction' ? (
            // Remount per auction so the bid starts at 0 again.
            <BidControls key={view.round} view={view} />
          ) : (
            <>
              <p className="ask">{view.yourDig ? 'Dug. Waiting for the others…' : 'Tap a square, then dig. Only once!'}</p>
              <button type="button" className="btn primary" disabled={!pick || view.yourDig !== null} onClick={digNow}>
                {pick ? `Dig at ${String.fromCharCode(65 + pick[0])}${pick[1] + 1}` : 'Pick a square'}
              </button>
              <KeyHint>Keys: arrows move the spade, Enter digs.</KeyHint>
            </>
          )}
          {last && (
            <p className="last">
              Box {view.history.length}: {last.winners.length ? `${last.winners.map(who).join(' & ')} paid ${last.price}` : 'nobody bid'} ·{' '}
              <span className="muted">{last.bids.map((b, k) => `${who(k)} ${b}`).join(', ')}</span>
            </p>
          )}
          <ul className="list">
            {view.players.map((p, k) => (
              <li key={p} className={k === you ? 'me' : ''}>
                <span>
                  {who(k)} {(view.phase === 'auction' ? view.bidIn[k] : view.dug[k]) && <span className="ti-check">✓</span>}
                </span>
                <span className="score">{view.coins[k]} coins</span>
              </li>
            ))}
          </ul>
          <p className="muted small center">Dig points: exact 500, one step off 300, two 150, three 50; plus the coins you keep.</p>
        </>
      }
    />
  );
}
