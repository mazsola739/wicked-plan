// Zoo. Rules page: docs/games/animal-tile-auction.md

import { useState } from 'react';
import { t } from '../i18n';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
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
  patternScore: number;
  log: string[];
  result: { patterns: Part[][]; scores: number[] } | null;
}

const ANIMALS = ['fox', 'owl', 'frog', 'bear'];
const said = (line: string) => line.replace(/\bbot-\d+\b|\byou\b/g, nameOf);

/**
 * A faceted (low-poly) head per animal in its colour, lit from the top left: white and black
 * overlays make the facets, `eye` shapes are cut in the page's near-black, white glints.
 */
function Animal({ animal }: { animal: number }) {
  return (
    <svg viewBox="0 0 40 40" className={`ata-animal ata-a${animal}`} role="img" aria-label={t(ANIMALS[animal] ?? '')}>
      {animal === 0 && (
        <>
          <path d="M7,4 16,12 20,11 24,12 33,4 34,18 36,23 25,30 20,36 15,30 4,23 6,18Z" />
          <path fill="#fff" fillOpacity=".38" d="M16,12 20,11 20,21 12,17Z" />
          <path fill="#fff" fillOpacity=".2" d="M7,4 16,12 12,17 6,18Z M20,11 24,12 28,17 20,21Z" />
          <path fill="#fff" fillOpacity=".1" d="M12,17 20,21 20,30 14,24Z" />
          <path fill="#000" fillOpacity=".12" d="M28,17 20,21 20,30 26,24Z" />
          <path fill="#000" fillOpacity=".25" d="M34,18 28,17 26,24 36,23Z" />
          <path fill="#fff" fillOpacity=".5" d="M4,23 14,24 20,30 20,36 15,30Z" />
          <path fill="#fff" fillOpacity=".36" d="M36,23 26,24 20,30 20,36 25,30Z" />
          <path fill="#000" fillOpacity=".35" d="M8.5,7.5 14.5,12.3 9,15Z M31.5,7.5 25.5,12.3 31,15Z" />
          <path className="eye" d="M10,18.6 14.5,19.4 16.8,21 12.4,22Z M30,18.6 25.5,19.4 23.2,21 27.6,22Z M17,31 23,31 20,34.6Z" />
          <path
            fill="#fff"
            d="M12.6,19.3 13.4,20.1 12.6,20.9 11.8,20.1Z M25.8,19.5 26.6,20.3 25.8,21.1 25,20.3Z M19,31.3 19.6,31.9 19,32.5 18.4,31.9Z"
          />
        </>
      )}
      {animal === 1 && (
        <>
          <path d="M3.5,3.5 12,9 20,8 28,9 36.5,3.5 34,15 35,24 32,31 26,35.5 20,36.5 14,35.5 8,31 5,24 6,15Z" />
          <path fill="#fff" fillOpacity=".3" d="M3.5,3.5 12,9 20,8 20,22 5.7,18 6,15Z" />
          <path fill="#fff" fillOpacity=".2" d="M3.5,3.5 12,9 7.5,12Z" />
          <path
            fill="#000"
            fillOpacity=".15"
            d="M34.3,18 20,22 20,30 33.3,28 35,24Z M6.7,28 20,30 20,36.5 14,35.5 8,31Z M36.5,3.5 28,9 32.5,12Z"
          />
          <path fill="#000" fillOpacity=".3" d="M33.3,28 20,30 20,36.5 26,35.5 32,31Z" />
          <path
            fill="#fff"
            fillOpacity=".42"
            d="M20,21.7 16.2,25.5 10.8,25.5 7,21.7 7,16.3 10.8,12.5 16.2,12.5 20,16.3Z M20,21.7 23.8,25.5 29.2,25.5 33,21.7 33,16.3 29.2,12.5 23.8,12.5 20,16.3Z"
          />
          <path
            className="eye"
            d="M13.5,15.6 16.4,17.3 16.4,20.7 13.5,22.4 10.6,20.7 10.6,17.3Z M26.5,15.6 29.4,17.3 29.4,20.7 26.5,22.4 23.6,20.7 23.6,17.3Z M17.6,22 22.4,22 20,28.5Z"
          />
          <path
            fill="#fff"
            fillOpacity=".3"
            d="M17.6,22 20,22 20,28.5Z M13.3,29.8 15.5,31.4 17.7,29.8 15.5,33.4Z M22.3,29.8 24.5,31.4 26.7,29.8 24.5,33.4Z M17.8,31.6 20,33.2 22.2,31.6 20,35.2Z"
          />
          <path fill="#fff" d="M12.3,17 13.2,17.9 12.3,18.8 11.4,17.9Z M25.3,17 26.2,17.9 25.3,18.8 24.4,17.9Z" />
        </>
      )}
      {animal === 2 && (
        <>
          <path d="M4,21 11,15 20,14 29,15 36,21 35,28 28,34 20,35.5 12,34 5,28Z" />
          <path fill="#fff" fillOpacity=".2" d="M4,21 11,15 20,14 20,21Z M35.3,25.4 20,30 20,35.5 28,34 35,28Z" />
          <path d="M11.5,4.5 17.6,8 17.6,15 11.5,18.5 5.4,15 5.4,8Z M28.5,4.5 34.6,8 34.6,15 28.5,18.5 22.4,15 22.4,8Z" />
          <path fill="#fff" fillOpacity=".33" d="M11.5,4.5 5.4,8 5.4,15 11.5,18.5Z M4.7,25.4 20,30 20,35.5 12,34 5,28Z" />
          <path fill="#000" fillOpacity=".18" d="M28.5,4.5 34.6,8 34.6,15 28.5,18.5Z M36,21 20,21 20,28 35.5,23.4Z" />
          <path
            fill="#fff"
            fillOpacity=".45"
            d="M11.5,6.7 15.7,9.1 15.7,13.9 11.5,16.3 7.3,13.9 7.3,9.1Z M28.5,6.7 32.7,9.1 32.7,13.9 28.5,16.3 24.3,13.9 24.3,9.1Z"
          />
          <path
            className="eye"
            d="M8.3,11.5 10,9.8 13,9.8 14.7,11.5 13,13.2 10,13.2Z M25.3,11.5 27,9.8 30,9.8 31.7,11.5 30,13.2 27,13.2Z M7.5,23.2 20,27.4 32.5,23.2 32.3,24.8 20,29 7.7,24.8Z M17.6,19.6 18.4,20.4 17.6,21.2 16.8,20.4Z M22.4,19.6 23.2,20.4 22.4,21.2 21.6,20.4Z"
          />
          <path fill="#fff" d="M10.4,10 11.1,10.7 10.4,11.4 9.7,10.7Z M27.4,10 28.1,10.7 27.4,11.4 26.7,10.7Z" />
        </>
      )}
      {animal === 3 && (
        <>
          <path d="M9.5,3.5 14,6 14,12 9.5,14.5 5,12 5,6Z M30.5,3.5 35,6 35,12 30.5,14.5 26,12 26,6Z" />
          <path
            fill="#000"
            fillOpacity=".35"
            d="M9.3,6 11.6,7.3 11.6,9.9 9.3,11.2 7,9.9 7,7.3Z M30.7,6 33,7.3 33,9.9 30.7,11.2 28.4,9.9 28.4,7.3Z"
          />
          <path d="M13,8 20,7 27,8 33,13.5 36,21 33.5,29.5 27,34.5 20,36 13,34.5 6.5,29.5 4,21 7,13.5Z" />
          <path fill="#fff" fillOpacity=".12" d="M7,13.5 20,16 14,24 4,21Z M27,8 20,7 20,16 33,13.5Z" />
          <path fill="#000" fillOpacity=".12" d="M33,13.5 20,16 26,24 36,21Z M4,21 14,24 20,34 20,36 13,34.5 6.5,29.5Z" />
          <path fill="#000" fillOpacity=".25" d="M36,21 26,24 20,34 20,36 27,34.5 33.5,29.5Z" />
          <path fill="#fff" fillOpacity=".4" d="M13,8 20,7 20,16 7,13.5Z M14,24 20,21.5 26,24 27,29.5 20,34 13,29.5Z" />
          <path
            className="eye"
            d="M13.5,16.7 15.5,17.9 15.5,20.1 13.5,21.3 11.5,20.1 11.5,17.9Z M26.5,16.7 28.5,17.9 28.5,20.1 26.5,21.3 24.5,20.1 24.5,17.9Z M16.8,24.6 20,23.6 23.2,24.6 22.2,27 20,28.2 17.8,27Z M19.2,27.8 20.8,27.8 20.8,30.2 22.3,29.3 23.1,30.6 20,32.6 16.9,30.6 17.7,29.3 19.2,30.2Z"
          />
          <path
            fill="#fff"
            d="M12.8,17.6 13.5,18.3 12.8,19 12.1,18.3Z M25.8,17.6 26.5,18.3 25.8,19 25.1,18.3Z M18.6,24.4 19.3,25 18.6,25.6 17.9,25Z"
          />
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
        {value === 1 ? t('Bid {n} chip', { n: value }) : t('Bid {n} chips', { n: value })}
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
      ? t('Game over: every pattern is scored.')
      : placing
        ? t('You won the tile: tap an empty cell to place it.')
        : view.phase === 'place'
          ? t('{name} is placing the tile…', { name: nameOf(view.players[view.placer ?? 0] ?? '') })
          : bidding
            ? t('Bid for this tile.')
            : t('Waiting for {names}…', { names: waiting.join(', ') || t('the others') });
  return (
    <Play
      top={
        <GameTop
          title="Zoo"
          step={view.phase === 'over' ? t('Game over') : t('Auction {n} of {total}', { n: view.auction, total: view.auctions })}
          score={view.patternScore + (view.chips[view.me] ?? 0)}
        />
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
              aria-label={`${String.fromCharCode(65 + (c % 5))}${Math.floor(c / 5) + 1}${a === null ? '' : ` ${t(ANIMALS[a] ?? '')}`}`}
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
                <span className="label">{t('On offer')}</span>
                <span className="ata-cell big">
                  <Animal animal={view.tile} />
                </span>
                <span className="small">{t(ANIMALS[view.tile] ?? '')}</span>
              </div>
            )}
            <div className="ata-offer">
              <span className="label">{t('Your pattern')}</span>
              <PatternCard parts={view.pattern} />
              <span className="small muted">{t('any rotation')}</span>
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
              <div className="label">{t('Patterns')}</div>
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
              <li key={i}>{said(t(l))}</li>
            ))}
          </ul>
          <KeyHint>{t('Keys: digits + Enter bid; arrows + Enter place.')}</KeyHint>
        </>
      }
    />
  );
}
