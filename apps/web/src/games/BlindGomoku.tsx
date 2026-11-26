// Blind Gomoku: five in a row, older tiles face down. Rules page: docs/games/blind-gomoku.md

import { useState } from 'react';
import { t } from '../i18n';
import { GameTop, KeyHint, move, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './BlindGomoku.css';

interface BlindGomokuView {
  size: number;
  cells: (null | 'down' | 'dark' | 'light')[];
  last: number | null;
  yourTurn: boolean;
  peeksLeft: number;
  peeking: boolean;
  peekLeftMs: number;
  tiles: number;
  outcome: null | 'won' | 'lost' | 'draw';
  winLine: number[];
  score: number;
  over: boolean;
}

export default function BlindGomoku({ view: raw }: ScreenProps) {
  const view = raw as BlindGomokuView;
  const n = view.size;
  const [cursor, setCursor] = useState<number | null>(null);
  const free = (c: number | null): c is number => c !== null && view.cells[c] === null;
  const place = (c: number | null) => {
    if (!view.yourTurn || !free(c)) return;
    move({ type: 'place', cell: c });
    setCursor(null);
  };
  const tap = (c: number) => (cursor === c ? place(c) : setCursor(c));
  useKeys((key) => {
    const d = ({ ArrowLeft: -1, ArrowRight: 1, ArrowUp: -n, ArrowDown: n } as Record<string, number>)[key];
    if (d !== undefined) setCursor((c) => Math.min(n * n - 1, Math.max(0, (c ?? Math.floor((n * n) / 2) - d) + d)));
    else if (key === 'Enter' || key === ' ') place(cursor);
    else if (key === 'p' || key === 'P') move({ type: 'peek' });
    else return false;
    return true;
  });
  const status = view.over
    ? t({ won: 'Five in a row: you won!', lost: 'The bot made five in a row.', draw: 'The board is full: a draw.' }[view.outcome ?? 'draw'])
    : view.yourTurn
      ? cursor !== null && free(cursor)
        ? t('Tap again (or Place) to put your tile here.')
        : t('Your turn: tap an empty square.')
      : t('The bot is thinking…');
  return (
    <Play
      top={
        <>
          <GameTop title="Blind Gomoku" step={t('{n} tiles on the board', { n: view.tiles })} score={view.score} />
          {view.peeking && <TimerBar leftMs={view.peekLeftMs} totalMs={3000} />}
        </>
      }
      stage={
        <div
          className={`bg-board${view.peeking ? ' peeking' : ''}`}
          style={{ gridTemplateColumns: `repeat(${n}, 1fr)`, gridTemplateRows: `repeat(${n}, 1fr)` }}
        >
          {view.cells.map((cell, i) => (
            <button
              // biome-ignore lint/suspicious/noArrayIndexKey: squares are positional
              key={i}
              type="button"
              aria-label={t('Square {n}: {state}', { n: i + 1, state: t(cell ?? 'empty') })}
              className={`bg-sq${cursor === i ? ' cursor' : ''}${view.last === i ? ' last' : ''}${view.winLine.includes(i) ? ' win' : ''}`}
              disabled={view.over}
              onClick={() => tap(i)}
            >
              {cell && <span className={`bg-tile ${cell}`} />}
              {!cell && cursor === i && view.yourTurn && <span className="bg-tile ghost" />}
            </button>
          ))}
        </div>
      }
      controls={
        <>
          <p className={`last${view.outcome === 'won' ? ' ok' : view.outcome === 'lost' ? ' bad' : ''}`}>{status}</p>
          <div className="bg-legend muted small">
            <span>
              <span className="bg-tile dark" /> {t('yours')}
            </span>
            <span>
              <span className="bg-tile light" /> {t('bot')}
            </span>
            <span>
              <span className="bg-tile down" /> {t('face down')}
            </span>
          </div>
          {!view.over && (
            <div className="row">
              <button type="button" className="btn" disabled={view.peeksLeft === 0 || view.peeking} onClick={() => move({ type: 'peek' })}>
                {t('Peek ({n})', { n: view.peeksLeft })}
              </button>
              <button type="button" className="btn primary" disabled={!view.yourTurn || !free(cursor)} onClick={() => place(cursor)}>
                {t('Place')}
              </button>
            </div>
          )}
          <p className="muted small center">
            {t('Only the newest tile shows its colour. A peek shows every tile for 3 seconds but costs 100 points of a win.')}
          </p>
          <KeyHint>{t('Keys: arrows move, Enter places, P peeks.')}</KeyHint>
        </>
      }
    />
  );
}
