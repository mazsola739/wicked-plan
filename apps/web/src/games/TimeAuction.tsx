// Time Auction: hold the button to bid time from your bank. Rules: docs/games/time-auction.md

import { useContext, useEffect, useRef } from 'react';
import { t } from '../i18n';
import { GameTop, KeyHint, move, nameOf, Play, PreviewContext, type ScreenProps, TimerBar } from './common';
import './TimeAuction.css';

type Phase = 'ready' | 'countdown' | 'bidding' | 'result' | 'over';

interface TaView {
  phase: Phase;
  round: number;
  rounds: number;
  timeLeftMs: number;
  phaseMs: number;
  bankMs: number;
  startBankMs: number;
  holding: boolean;
  bidding: boolean;
  heldMs: number;
  lastHeldMs: number;
  seats: { id: string; tokens: number; ready: boolean | null; outOfTime: boolean }[];
  log: { round: number; winner: string | null; bidders: number }[];
}

const secs = (ms: number) => `${(ms / 1000).toFixed(1)} s`;

/**
 * A gold token cut like a gem (docs/design.md, "Icons are faceted"): an octagon with facets lit
 * from the top left, an hourglass cut in its face and a white glint.
 */
function Token() {
  return (
    <svg viewBox="0 0 20 20" aria-hidden="true">
      <path d="M18.8,13.6 13.6,18.8 6.4,18.8 1.2,13.6 1.2,6.4 6.4,1.2 13.6,1.2 18.8,6.4Z" />
      <path fill="#fff" fillOpacity=".5" d="M1.2,6.4 6.4,1.2 7.9,4.8 4.8,7.9Z" />
      <path fill="#fff" fillOpacity=".4" d="M6.4,1.2 13.6,1.2 12.1,4.8 7.9,4.8Z" />
      <path fill="#fff" fillOpacity=".3" d="M1.2,13.6 1.2,6.4 4.8,7.9 4.8,12.1Z" />
      <path fill="#000" fillOpacity=".25" d="M18.8,6.4 18.8,13.6 15.2,12.1 15.2,7.9Z" />
      <path fill="#000" fillOpacity=".3" d="M13.6,18.8 6.4,18.8 7.9,15.2 12.1,15.2Z" />
      <path fill="#000" fillOpacity=".35" d="M18.8,13.6 13.6,18.8 12.1,15.2 15.2,12.1Z" />
      <path className="eye" d="M7.7,6.9 12.3,6.9 10,10Z M7.7,13.1 12.3,13.1 10,10Z" />
      <path fill="#fff" d="M5.6,3.8 6.4,4.6 5.6,5.4 4.8,4.6Z" />
    </svg>
  );
}

function status(v: TaView): string {
  const last = v.log[v.log.length - 1];
  switch (v.phase) {
    case 'ready':
      return v.bankMs === 0
        ? t('Your bank is empty: you sit this one out.')
        : v.holding
          ? t('Keep holding…')
          : t('Press and hold the button.');
    case 'countdown':
      return v.holding ? t('Starting in {n}… let go now to pass.', { n: Math.ceil(v.timeLeftMs / 1000) }) : t('You pass this round.');
    case 'bidding':
      return v.holding
        ? t('Bidding: {time}', { time: secs(v.heldMs) })
        : v.bidding
          ? t('You let go at {time}. Waiting…', { time: secs(v.heldMs) })
          : t('You sit this one out. Waiting…');
    case 'result':
    case 'over':
      if (!last) return '';
      return last.winner
        ? last.winner === 'you'
          ? t('You win the token.')
          : t('{name} wins the token.', { name: nameOf(last.winner) })
        : t('Nobody wins the token.');
  }
}

export default function TimeAuction({ view }: ScreenProps) {
  const v = view as TaView;
  const ref = useRef(v);
  ref.current = v;
  const press = () => {
    const cur = ref.current;
    if (cur.phase === 'ready' && !cur.holding && cur.bankMs > 0) move({ type: 'press' });
  };
  const release = () => {
    if (ref.current.holding) move({ type: 'release' });
  };
  // Desktop: hold Space.
  const preview = useContext(PreviewContext);
  useEffect(() => {
    if (preview) return;
    const down = (e: KeyboardEvent) => {
      if (e.code !== 'Space') return;
      e.preventDefault();
      if (!e.repeat) press();
    };
    const up = (e: KeyboardEvent) => {
      if (e.code !== 'Space') return;
      e.preventDefault();
      release();
    };
    window.addEventListener('keydown', down);
    window.addEventListener('keyup', up);
    return () => {
      window.removeEventListener('keydown', down);
      window.removeEventListener('keyup', up);
    };
  });
  const me = v.seats.find((s) => s.id === 'you');
  const step = v.phase === 'over' ? t('Final round') : t('Round {n} of {total}', { n: v.round, total: v.rounds });
  return (
    <Play
      top={
        <>
          <GameTop title="Time Auction" step={step} score={me?.tokens ?? 0} />
          {v.phase === 'bidding' ? (
            <div className="timer time-auction-live">{t('Clock running')}</div>
          ) : (
            v.phaseMs > 0 && <TimerBar leftMs={v.timeLeftMs} totalMs={v.phaseMs} />
          )}
        </>
      }
      stage={
        <div className="card">
          <ul className="list">
            {v.seats.map((s) => (
              <li key={s.id}>
                <span>
                  {nameOf(s.id)}
                  {s.outOfTime && <span className="muted small"> · {t('out of time')}</span>}
                  {s.ready !== null && (
                    <span className={`time-auction-dot${s.ready ? ' on' : ''}`} title={s.ready ? t('holding') : t('not yet')} />
                  )}
                </span>
                <span className="time-auction-tokens" title={t('tokens')}>
                  {Array.from({ length: s.tokens }, (_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: identical tokens
                    <Token key={i} />
                  ))}
                  {s.tokens === 0 && <span className="muted small">{t('no tokens')}</span>}
                </span>
              </li>
            ))}
          </ul>
          {v.log.length > 0 && (
            <ol className="time-auction-log muted small">
              {v.log
                .slice()
                .reverse()
                .map((l) => (
                  <li key={l.round}>
                    {l.winner === 'you'
                      ? t('Round {n}: You won ({b} bid)', { n: l.round, b: l.bidders })
                      : l.winner
                        ? t('Round {n}: {name} won ({b} bid)', { n: l.round, name: nameOf(l.winner), b: l.bidders })
                        : t('Round {n}: no winner ({b} bid)', { n: l.round, b: l.bidders })}
                  </li>
                ))}
            </ol>
          )}
        </div>
      }
      controls={
        <>
          <div className="time-auction-bank">
            <div className="label">{t('Your bank')}</div>
            <div className="time-auction-bar">
              <div style={{ width: `${(100 * v.bankMs) / v.startBankMs}%` }} />
              <span>{secs(v.bankMs)}</span>
            </div>
          </div>
          <p className="ask">{status(v)}</p>
          {(v.phase === 'result' || v.phase === 'over') && v.lastHeldMs > 0 && (
            <p className="last">{t('You held {time}.', { time: secs(v.lastHeldMs) })}</p>
          )}
          <button
            type="button"
            className={`time-auction-btn${v.holding ? ' held' : ''}`}
            disabled={!v.holding && (v.phase !== 'ready' || v.bankMs === 0)}
            onPointerDown={(e) => {
              e.currentTarget.setPointerCapture(e.pointerId);
              press();
            }}
            onPointerUp={release}
            onPointerCancel={release}
            onContextMenu={(e) => e.preventDefault()}
          >
            {v.holding ? t('HOLDING') : t('HOLD')}
          </button>
          <KeyHint>{t('Or hold Space.')}</KeyHint>
        </>
      }
    />
  );
}
