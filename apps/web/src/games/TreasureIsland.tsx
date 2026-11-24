// Treasure Island: bid in secret for clue boxes, then dig once.
// Rules page: docs/games/treasure-island.md

import { useState } from 'react';
import './TreasureIsland.css';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';

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
  treasure: Square | null;
  digs: (Square | null)[] | null;
  scores: number[] | null;
}

const C = 40;

/**
 * Faceted (low-poly) pictograms lit from the top left, like the animals of Zoo:
 * drawn round 0,0 in the colour of their group (CSS fill), white and black overlays make the
 * facets, `eye` shapes are cut in the page's near-black, white glints.
 */
function Chest() {
  return (
    <>
      <path d="M-13,-10 -7,-13.5 7,-13.5 13,-10 14.5,-4 14,12 -14,12 -14.5,-4Z" />
      <path fill="#fff" fillOpacity=".5" d="M-13,-10 -7,-13.5 0,-13.5 0,-10Z" />
      <path fill="#fff" fillOpacity=".25" d="M0,-13.5 7,-13.5 13,-10 0,-10Z" />
      <path fill="#fff" fillOpacity=".15" d="M-13,-10 0,-10 0,-4 -14.5,-4Z M-14.4,-2.4 14.4,-2.4 -14,12Z" />
      <path fill="#000" fillOpacity=".15" d="M0,-10 13,-10 14.5,-4 0,-4Z" />
      <path fill="#000" fillOpacity=".22" d="M14.4,-2.4 14,12 -14,12Z" />
      <path
        fill="#000"
        fillOpacity=".3"
        d="M-10,-11.7 -6.8,-13.5 -6.8,12 -10,12Z M6.8,-13.5 10,-11.7 10,12 6.8,12Z M-14,9 14,9 14,12 -14,12Z"
      />
      <path className="eye" d="M-14.5,-4 14.5,-4 14.4,-2.4 -14.4,-2.4Z" />
      <path fill="#fff" fillOpacity=".5" d="M-3.2,-5 3.2,-5 3.2,2 0,4.2 -3.2,2Z" />
      <path className="eye" d="M0,-2.3 1,-1.3 0.5,-0.3 1,2 -1,2 -0.5,-0.3 -1,-1.3Z" />
      <path fill="#fff" d="M-4.5,-12.6 -3.8,-11.9 -4.5,-11.2 -5.2,-11.9Z M-11.6,-0.9 -11,-0.3 -11.6,0.3 -12.2,-0.3Z" />
    </>
  );
}

function XMark() {
  return (
    <>
      <path d="M-8.9,-13.1 0,-4.2 8.9,-13.1 13.1,-8.9 4.2,0 13.1,8.9 8.9,13.1 0,4.2 -8.9,13.1 -13.1,8.9 -4.2,0 -13.1,-8.9Z" />
      <path fill="#fff" fillOpacity=".45" d="M0,0 0,-4.2 8.9,-13.1 11,-11Z" />
      <path fill="#fff" fillOpacity=".35" d="M0,0 0,-4.2 -8.9,-13.1 -11,-11Z" />
      <path fill="#fff" fillOpacity=".25" d="M0,0 -4.2,0 -13.1,8.9 -11,11Z" />
      <path fill="#fff" fillOpacity=".12" d="M0,0 -11,-11 -13.1,-8.9 -4.2,0Z M0,0 4.2,0 13.1,8.9 11,11Z" />
      <path fill="#000" fillOpacity=".2" d="M0,0 11,-11 13.1,-8.9 4.2,0Z M0,0 -11,11 -8.9,13.1 0,4.2Z" />
      <path fill="#000" fillOpacity=".35" d="M0,0 11,11 8.9,13.1 0,4.2Z" />
      <path fill="#fff" d="M4.6,-8.8 5.3,-8.1 4.6,-7.4 3.9,-8.1Z" />
    </>
  );
}

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
      {view.treasure && (
        <g className="ti-treasure" transform={`translate(${view.treasure[0] * C + C / 2} ${view.treasure[1] * C + C / 2})`}>
          <Chest />
        </g>
      )}
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
          <XMark />
        </g>
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
        <GameTop
          title="Treasure Island"
          step={view.phase === 'auction' ? `Auction ${view.round} of ${view.auctions}` : 'Dig for the treasure'}
          score={view.coins[you] ?? 0}
        />
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
