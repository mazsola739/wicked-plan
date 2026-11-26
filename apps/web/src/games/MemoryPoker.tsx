// Memory Poker: study 16 cards, take two blind. Rules page: docs/games/memory-poker.md

import { t } from '../i18n';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './MemoryPoker.css';

interface Face {
  rank: number;
  suit: number;
}

interface MemoryPokerView {
  phase: 'study' | 'pick' | 'show';
  round: number;
  rounds: number;
  grid: { card: Face | null; takenBy: string | null }[];
  shared: Face[];
  seats: { id: string; ready: boolean; cards: (Face | null)[]; hand: string | null; points: number; roundPoints: number }[];
  you: number;
  turn: string | null;
  over: boolean;
}

const RANKS: Record<number, string> = { 11: 'J', 12: 'Q', 13: 'K', 14: 'A' };
const SUITS = ['spades', 'hearts', 'diamonds', 'clubs'];

/**
 * A faceted suit sign in the card's colour (red for hearts and diamonds), lit from the top left:
 * white and black overlays make the facets, with a white glint.
 */
function Suit({ suit }: { suit: number }) {
  return (
    <svg viewBox="0 0 40 40" className="mp-suit" role="img" aria-label={t(SUITS[suit] ?? '')}>
      {suit === 0 && (
        <>
          <path d="M20,3 28.5,10 35,16 36.5,22.5 33.5,28 27.5,29.5 22.5,26 24,32 28.5,37 11.5,37 16,32 17.5,26 12.5,29.5 6.5,28 3.5,22.5 5,16 11.5,10Z" />
          <path fill="#fff" fillOpacity=".2" d="M17.5,26 20,25 20,37 11.5,37 16,32Z" />
          <path fill="#fff" fillOpacity=".3" d="M5,16 3.5,22.5 11,22Z" />
          <path fill="#fff" fillOpacity=".4" d="M20,3 11.5,10 20,17Z M11.5,10 5,16 11,22 20,17Z" />
          <path fill="#000" fillOpacity=".2" d="M35,16 36.5,22.5 29,22Z M22.5,26 20,25 20,37 28.5,37 24,32Z" />
          <path fill="#000" fillOpacity=".28" d="M33.5,28 27.5,29.5 22.5,26 29,22Z" />
          <path
            fill="#000"
            fillOpacity=".35"
            d="M6.5,28 12.5,29.5 17.5,26 11,22Z M11,22 17.5,26 20,25 20,17Z M36.5,22.5 33.5,28 29,22Z M29,22 22.5,26 20,25 20,17Z"
          />
          <path fill="#fff" d="M15.4,9.6 16.6,10.8 15.4,12 14.2,10.8Z" />
        </>
      )}
      {suit === 1 && (
        <>
          <path d="M20,10 23.5,6 28,4 33.5,5.5 37,11 36,18 29,27 20,36 11,27 4,18 3,11 6.5,5.5 12,4 16.5,6Z" />
          <path fill="#fff" fillOpacity=".1" d="M3,11 12,12.5 4,18Z" />
          <path fill="#fff" fillOpacity=".3" d="M6.5,5.5 12,4 16.5,6 12,12.5Z" />
          <path fill="#fff" fillOpacity=".4" d="M3,11 6.5,5.5 12,12.5Z M33.5,5.5 28,4 23.5,6 28,12.5Z M23.5,6 20,10 20,21 28,12.5Z" />
          <path fill="#000" fillOpacity=".12" d="M4,18 12,12.5 20,21 11,27Z M11,27 20,21 20,36Z M37,11 33.5,5.5 28,12.5Z" />
          <path fill="#000" fillOpacity=".28" d="M37,11 28,12.5 36,18Z M29,27 20,21 20,36Z" />
          <path fill="#000" fillOpacity=".35" d="M36,18 28,12.5 20,21 29,27Z" />
          <path fill="#fff" d="M9.6,7.6 10.8,8.8 9.6,10 8.4,8.8Z" />
        </>
      )}
      {suit === 2 && (
        <>
          <path d="M20,2.5 32.5,20 20,37.5 7.5,20Z" />
          <path fill="#fff" fillOpacity=".3" d="M14,20 20,12 20,20Z" />
          <path fill="#fff" fillOpacity=".4" d="M7.5,20 20,2.5 20,12 14,20Z" />
          <path fill="#000" fillOpacity=".12" d="M20,2.5 32.5,20 26,20 20,12Z M20,37.5 7.5,20 14,20 20,28Z" />
          <path fill="#000" fillOpacity=".2" d="M26,20 20,28 20,20Z" />
          <path fill="#000" fillOpacity=".35" d="M32.5,20 20,37.5 20,28 26,20Z" />
          <path fill="#fff" d="M15.6,14 16.8,15.2 15.6,16.4 14.4,15.2Z" />
        </>
      )}
      {suit === 3 && (
        <>
          <path d="M16.9,18.6 23.1,18.6 23.1,24.7 22.3,27.5 23.5,32 28.5,37 11.5,37 16.5,32 17.7,27.5 16.9,24.7Z M16.9,3.8 23.1,3.8 27.4,8.1 27.4,14.3 23.1,18.6 16.9,18.6 12.6,14.3 12.6,8.1Z M6.5,14.3 12.6,14.3 16.9,18.6 16.9,24.7 12.6,29 6.5,29 2.2,24.7 2.2,18.6Z M27.4,14.3 33.5,14.3 37.8,18.6 37.8,24.7 33.5,29 27.4,29 23.1,24.7 23.1,18.6Z" />
          <path fill="#fff" fillOpacity=".1" d="M16.9,24.7 20,22.2 20,27.5 17.7,27.5Z" />
          <path
            fill="#fff"
            fillOpacity=".2"
            d="M12.6,14.3 12.6,8.1 20,11.2Z M2.2,24.7 2.2,18.6 9.5,21.7Z M23.1,24.7 23.1,18.6 30.5,21.7Z M16.9,24.7 16.9,18.6 20,22.2Z M17.7,27.5 20,27.5 20,37 11.5,37 16.5,32Z"
          />
          <path
            fill="#fff"
            fillOpacity=".3"
            d="M16.9,3.8 23.1,3.8 20,11.2Z M6.5,14.3 12.6,14.3 9.5,21.7Z M27.4,14.3 33.5,14.3 30.5,21.7Z"
          />
          <path
            fill="#fff"
            fillOpacity=".4"
            d="M12.6,8.1 16.9,3.8 20,11.2Z M2.2,18.6 6.5,14.3 9.5,21.7Z M23.1,18.6 27.4,14.3 30.5,21.7Z M16.9,18.6 23.1,18.6 20,22.2Z"
          />
          <path
            fill="#000"
            fillOpacity=".12"
            d="M16.9,18.6 12.6,14.3 20,11.2Z M6.5,29 2.2,24.7 9.5,21.7Z M27.4,29 23.1,24.7 30.5,21.7Z M22.3,27.5 20,27.5 20,37 28.5,37 23.5,32Z"
          />
          <path
            fill="#000"
            fillOpacity=".28"
            d="M27.4,8.1 27.4,14.3 20,11.2Z M16.9,18.6 16.9,24.7 9.5,21.7Z M37.8,18.6 37.8,24.7 30.5,21.7Z"
          />
          <path
            fill="#000"
            fillOpacity=".35"
            d="M27.4,14.3 23.1,18.6 20,11.2Z M23.1,18.6 16.9,18.6 20,11.2Z M16.9,24.7 12.6,29 9.5,21.7Z M12.6,29 6.5,29 9.5,21.7Z M37.8,24.7 33.5,29 30.5,21.7Z M33.5,29 27.4,29 30.5,21.7Z M23.1,18.6 23.1,24.7 20,22.2Z M23.1,24.7 22.3,27.5 20,27.5 20,22.2Z"
          />
          <path fill="#fff" d="M16.4,6.4 17.6,7.6 16.4,8.8 15.2,7.6Z M6,17.2 7,18.2 6,19.2 5,18.2Z" />
        </>
      )}
    </svg>
  );
}

function Card({ face, small }: { face: Face | null; small?: boolean }) {
  if (!face) return <span className={`mp-card back${small ? ' small' : ''}`} />;
  return (
    <span className={`mp-card${face.suit === 1 || face.suit === 2 ? ' red' : ''}${small ? ' small' : ''}`}>
      <b>{RANKS[face.rank] ?? face.rank}</b>
      <Suit suit={face.suit} />
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
        ? t('Waiting for the others…')
        : t('Tap Ready when you have them memorised.')
      : view.phase === 'pick'
        ? myTurn
          ? t('Your pick: tap a face-down card.')
          : t('{name} is picking…', { name: nameOf(view.turn ?? '') })
        : t('Two cards + the shared three make each hand.');
  return (
    <Play
      top={
        <GameTop
          title="Memory Poker"
          step={`${t('Round {n} of {total}', { n: view.round, total: view.rounds })} · ${t(step)}`}
          score={me?.points ?? 0}
        />
      }
      stage={
        <div className="mp-table">
          <div className="mp-shared">
            <span className="label">{t('Shared')}</span>
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
                  aria-label={t('Square {letter}', { letter: String.fromCharCode(65 + i) })}
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
              {view.phase === 'study' ? t('Ready') : view.round < view.rounds ? t('Next round') : t('Finish')}
            </button>
          )}
          <ul className="list mp-seats">
            {view.seats.map((s, i) => (
              <li key={s.id} className={i === view.you ? 'me' : ''}>
                <span className="mp-who">
                  {nameOf(s.id)}
                  {view.turn === s.id && <span className="muted small"> · {t('picking')}</span>}
                </span>
                <span className="mp-hand">
                  {s.cards.map((c, j) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: two cards
                    <Card key={j} face={c} small />
                  ))}
                  {s.hand && <span className="muted small">{t(s.hand)}</span>}
                </span>
                <span className="score">
                  {view.phase === 'show' && s.roundPoints > 0 && <small>+{s.roundPoints} </small>}
                  {s.points}
                </span>
              </li>
            ))}
          </ul>
          <p className="muted small center">
            {t(
              'Pair 50 · two pair 90 · three 130 · straight 180 · flush 200 · full house 250 · four 320 · straight flush 400 · best hand +60',
            )}
          </p>
          <KeyHint>{t('Keys: A–P pick a square, Enter is Ready / Next.')}</KeyHint>
        </>
      }
    />
  );
}
