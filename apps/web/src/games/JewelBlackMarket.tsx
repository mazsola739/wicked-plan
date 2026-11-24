// Gem Black Market. Rules page: docs/games/jewel-black-market.md

import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './JewelBlackMarket.css';

type Kind = 'diamond' | 'gold' | 'silver' | 'bronze' | 'bomb';
interface Card {
  kind: Kind;
  value: number;
}
interface Reveal {
  round: number;
  cards: Card[];
  buyer: Card[];
  discard: Card;
  points: number[];
}
interface JewelView {
  players: string[];
  me: number;
  round: number;
  rounds: number;
  swapRound: number;
  turn: number | null;
  myCard: Card;
  scores: number[];
  log: string[];
  last: Reveal | null;
  over: boolean;
}

const LETTERS = 'ABCDEFGH';
const said = (line: string) => line.replace(/\bbot-\d+\b|\byou\b/g, nameOf);

/**
 * One faceted (low-poly) picture per kind in the card's colour, lit from the top left: white and
 * black overlays make the facets, tiny white glints. Diamond: a brilliant; gold: a bar; silver: a
 * step-cut stone; bronze: a pear-cut stone; bomb: a faceted ball with a lit fuse.
 */
function GemArt({ kind }: { kind: Kind }) {
  switch (kind) {
    case 'diamond':
      return (
        <>
          <path d="M13,7 27,7 35,15 20,35 5,15Z" />
          <path fill="#fff" fillOpacity=".45" d="M5,15 13,7 10,15Z" />
          <path fill="#fff" fillOpacity=".3" d="M13,7 20,15 10,15Z" />
          <path fill="#fff" fillOpacity=".2" d="M13,7 27,7 20,15Z M5,15 10,15 20,35Z" />
          <path fill="#000" fillOpacity=".15" d="M27,7 35,15 30,15Z M20,15 30,15 20,35Z" />
          <path fill="#000" fillOpacity=".32" d="M30,15 35,15 20,35Z" />
          <path fill="#fff" d="M15,9.2 15.9,10.1 15,11 14.1,10.1Z M11.2,17.8 12,18.6 11.2,19.4 10.4,18.6Z" />
        </>
      );
    case 'gold':
      return (
        <>
          <path d="M12,11 32,11 37,21 29,31 3,31 6,19Z" />
          <path fill="#fff" fillOpacity=".45" d="M6,19 12,11 32,11 26,19Z" />
          <path fill="#fff" fillOpacity=".15" d="M6,19 26,19 16,25 4.4,25Z" />
          <path fill="#000" fillOpacity=".12" d="M4.4,25 16,25 26,19 29,31 3,31Z" />
          <path fill="#000" fillOpacity=".28" d="M26,19 32,11 37,21 29,31Z" />
          <path fill="#fff" d="M12.6,13.6 13.6,14.6 12.6,15.6 11.6,14.6Z M28.4,13.1 29.2,13.9 28.4,14.7 27.6,13.9Z" />
        </>
      );
    case 'silver':
      return (
        <>
          <path d="M13,4 27,4 32,9 32,31 27,36 13,36 8,31 8,9Z" />
          <path fill="#fff" fillOpacity=".5" d="M8,9 13,4 16,10 14,12Z" />
          <path fill="#fff" fillOpacity=".35" d="M13,4 27,4 24,10 16,10Z M8,9 14,12 14,28 8,31Z" />
          <path fill="#fff" fillOpacity=".15" d="M27,4 32,9 26,12 24,10Z M16,10 24,10 26,12 14,28 14,12Z" />
          <path fill="#000" fillOpacity=".15" d="M32,9 32,31 26,28 26,12Z" />
          <path fill="#000" fillOpacity=".22" d="M13,36 16,30 24,30 27,36Z" />
          <path fill="#000" fillOpacity=".35" d="M32,31 27,36 24,30 26,28Z" />
          <path fill="#fff" d="M12.4,7.4 13.3,8.3 12.4,9.2 11.5,8.3Z M18,12.6 18.8,13.4 18,14.2 17.2,13.4Z" />
        </>
      );
    case 'bronze':
      return (
        <>
          <path d="M20,4 26,12 30,19 31,26 27,33 20,36 13,33 9,26 10,19 14,12Z" />
          <path fill="#fff" fillOpacity=".45" d="M10,19 14,12 17.1,18.2 15.2,21.6Z" />
          <path fill="#fff" fillOpacity=".35" d="M14,12 20,4 20,14.4 17.1,18.2Z M9,26 10,19 15.2,21.6 14.7,25Z" />
          <path fill="#fff" fillOpacity=".25" d="M20,4 26,12 22.9,18.2 20,14.4Z" />
          <path
            fill="#fff"
            fillOpacity=".15"
            d="M13,33 9,26 14.7,25 16.6,28.3Z M20,14.4 22.9,18.2 24.8,21.6 25.3,25 23.4,28.3 20,29.8 16.6,28.3 14.7,25 15.2,21.6 17.1,18.2Z"
          />
          <path fill="#000" fillOpacity=".12" d="M20,36 13,33 16.6,28.3 20,29.8Z" />
          <path fill="#000" fillOpacity=".22" d="M30,19 31,26 25.3,25 24.8,21.6Z" />
          <path fill="#000" fillOpacity=".32" d="M27,33 20,36 20,29.8 23.4,28.3Z M31,26 27,33 23.4,28.3 25.3,25Z" />
          <path fill="#fff" d="M14,16.4 14.9,17.3 14,18.2 13.1,17.3Z M19,19 19.7,19.7 19,20.4 18.3,19.7Z" />
        </>
      );
    default:
      return (
        <>
          <path d="M19,12 26.3,14.4 30.9,20.6 30.9,28.4 26.3,34.6 19,37 11.7,34.6 7.1,28.4 7.1,20.6 11.7,14.4Z" />
          <path d="M25,14.5 28.1,12.2 33.1,18.2 30,20.5Z" />
          <path d="M31 15 Q 32 9 36 8" className="fuse" />
          <path fill="#fff" fillOpacity=".45" d="M7.1,20.6 11.7,14.4 14.6,18.7 12.3,21.8Z M11.7,14.4 19,12 18.3,17.5 14.6,18.7Z" />
          <path fill="#fff" fillOpacity=".3" d="M7.1,28.4 7.1,20.6 12.3,21.8 12.3,25.7Z M25,14.5 28.1,12.2 30.6,15.2 27.5,17.5Z" />
          <path
            fill="#fff"
            fillOpacity=".15"
            d="M19,12 26.3,14.4 21.9,18.7 18.3,17.5Z M18.3,17.5 21.9,18.7 24.2,21.8 24.2,25.7 21.9,28.8 18.3,30 14.6,28.8 12.3,25.7 12.3,21.8 14.6,18.7Z"
          />
          <path fill="#000" fillOpacity=".12" d="M26.3,14.4 30.9,20.6 24.2,21.8 21.9,18.7Z" />
          <path fill="#000" fillOpacity=".22" d="M19,37 11.7,34.6 14.6,28.8 18.3,30Z M30.9,20.6 30.9,28.4 24.2,25.7 24.2,21.8Z" />
          <path
            fill="#000"
            fillOpacity=".35"
            d="M26.3,34.6 19,37 18.3,30 21.9,28.8Z M30.9,28.4 26.3,34.6 21.9,28.8 24.2,25.7Z M27.5,17.5 30.6,15.2 33.1,18.2 30,20.5Z"
          />
          <path fill="#fff" d="M36,3.6 37,6.6 39.6,7.6 37,8.6 36,11.6 35,8.6 32.4,7.6 35,6.6Z M12,16.8 13.2,18 12,19.2 10.8,18Z" />
        </>
      );
  }
}

function Gem({ card, small }: { card: Card; small?: boolean }) {
  return (
    <div className={`jbm-card jbm-${card.kind}${small ? ' small' : ''}`}>
      <svg viewBox="0 0 40 40" aria-hidden="true">
        <GemArt kind={card.kind} />
      </svg>
      <strong>{card.value}</strong>
      {!small && <span>{card.kind}</span>}
    </div>
  );
}

function Back({ label, onClick }: { label: string; onClick?: () => void }) {
  return (
    <button type="button" className="jbm-card jbm-back" disabled={!onClick} onClick={onClick}>
      <span>{label}</span>
    </button>
  );
}

export default function JewelBlackMarket({ view: raw }: ScreenProps) {
  const view = raw as JewelView;
  const mine = view.turn === view.me && !view.over;
  const bomb = view.myCard.kind === 'bomb';
  const canTable = mine && !bomb;
  useKeys((key) => {
    if (!mine) return false;
    const k = key.toUpperCase();
    const seat = LETTERS.indexOf(k);
    if (k === 'K' && !bomb) move({ type: 'keep' });
    else if (k === 'D' && !bomb) move({ type: 'swapDiscard' });
    else if (/^[1-3]$/.test(key) && !bomb) move({ type: 'swapBuyer', slot: Number(key) - 1 });
    else if (seat >= 0 && seat < view.players.length && seat !== view.me) move({ type: 'swapPlayer', seat });
    else return false;
    return true;
  });
  const status = view.over
    ? 'Game over.'
    : mine
      ? bomb
        ? 'You hold a bomb: pass it to a player.'
        : 'Your turn: keep, or swap with a player, the buyer or the discard.'
      : `${nameOf(view.players[view.turn ?? 0] ?? '')} is choosing…`;
  return (
    <Play
      top={
        <GameTop
          title="Gem Black Market"
          step={`Round ${view.round} of ${view.rounds} · swap ${view.swapRound} of 2`}
          score={view.scores[view.me] ?? 0}
        />
      }
      stage={
        <div className="jbm-table">
          <div className="jbm-zone">
            <div className="label">Buyer's space</div>
            <div className="jbm-row">
              {[0, 1, 2].map((slot) => (
                <Back key={slot} label={`${slot + 1}`} onClick={canTable ? () => move({ type: 'swapBuyer', slot }) : undefined} />
              ))}
              <div className="jbm-gap" />
              <Back label="Discard" onClick={canTable ? () => move({ type: 'swapDiscard' }) : undefined} />
            </div>
          </div>
          <ul className="jbm-seats">
            {view.players.map((p, i) => (
              <li key={p}>
                <button
                  type="button"
                  className={`jbm-seat${view.turn === i ? ' turn' : ''}${i === view.me ? ' me' : ''}`}
                  disabled={!mine || i === view.me}
                  onClick={() => move({ type: 'swapPlayer', seat: i })}
                >
                  <em>{LETTERS[i]}</em>
                  <span>{nameOf(p)}</span>
                  <strong>{view.scores[i]}</strong>
                </button>
              </li>
            ))}
          </ul>
          <div className="jbm-mine">
            <div className="label">Your card</div>
            <Gem card={view.myCard} />
          </div>
        </div>
      }
      controls={
        <>
          <p className="ask">{status}</p>
          {mine && !bomb && (
            <button type="button" className="btn primary" onClick={() => move({ type: 'keep' })}>
              Keep my card
            </button>
          )}
          <p className="muted small center">
            A jewel scores its value × the same-type jewels in the buyer's space. A bomb costs its value.
          </p>
          <ul className="jbm-log small">
            {view.log.map((l, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows
              <li key={i}>{said(l)}</li>
            ))}
          </ul>
          {view.last && (
            <div className="card jbm-reveal">
              <div className="label">Round {view.last.round} revealed</div>
              <div className="jbm-row">
                <span className="muted small">Buyer</span>
                {view.last.buyer.map((c, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: fixed three slots
                  <Gem key={i} card={c} small />
                ))}
                <span className="muted small">Discard</span>
                <Gem card={view.last.discard} small />
              </div>
              <ul className="list">
                {view.players.map((p, i) => (
                  <li key={p}>
                    <span>{nameOf(p)}</span>
                    <Gem card={view.last?.cards[i] as Card} small />
                    <strong className="score">{(view.last?.points[i] ?? 0) > 0 ? `+${view.last?.points[i]}` : view.last?.points[i]}</strong>
                  </li>
                ))}
              </ul>
            </div>
          )}
          <KeyHint>Keys: K keeps, 1–3 buyer card, D discard, A–H a player.</KeyHint>
        </>
      }
    />
  );
}
