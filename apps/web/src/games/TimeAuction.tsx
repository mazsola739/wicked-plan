// Time Auction: hold the button to bid time from your bank. Rules: docs/games/time-auction.md

import { useEffect, useRef } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar } from './common';
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

function status(v: TaView): string {
  const last = v.log[v.log.length - 1];
  switch (v.phase) {
    case 'ready':
      return v.bankMs === 0 ? 'Your bank is empty: you sit this one out.' : v.holding ? 'Keep holding…' : 'Press and hold the button.';
    case 'countdown':
      return v.holding ? `Starting in ${Math.ceil(v.timeLeftMs / 1000)}… let go now to pass.` : 'You pass this round.';
    case 'bidding':
      return v.holding
        ? `Bidding: ${secs(v.heldMs)}`
        : v.bidding
          ? `You let go at ${secs(v.heldMs)}. Waiting…`
          : 'You sit this one out. Waiting…';
    case 'result':
    case 'over':
      if (!last) return '';
      return last.winner ? `${nameOf(last.winner)} ${last.winner === 'you' ? 'win' : 'wins'} the token.` : 'Nobody wins the token.';
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
  useEffect(() => {
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
  const step = v.phase === 'over' ? 'Final round' : `Round ${v.round} of ${v.rounds}`;
  return (
    <Play
      top={
        <>
          <GameTop title="Time Auction" step={step} score={me?.tokens ?? 0} />
          {v.phaseMs > 0 ? (
            <TimerBar leftMs={v.timeLeftMs} totalMs={v.phaseMs} />
          ) : (
            <div className="timer time-auction-live">Clock running</div>
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
                  {s.outOfTime && <span className="muted small"> · out of time</span>}
                  {s.ready !== null && (
                    <span className={`time-auction-dot${s.ready ? ' on' : ''}`} title={s.ready ? 'holding' : 'not yet'} />
                  )}
                </span>
                <span className="time-auction-tokens" title="tokens">
                  {Array.from({ length: s.tokens }, (_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: identical tokens
                    <i key={i} />
                  ))}
                  {s.tokens === 0 && <span className="muted small">no tokens</span>}
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
                    Round {l.round}: {l.winner ? `${nameOf(l.winner)} won` : 'no winner'} ({l.bidders} bid)
                  </li>
                ))}
            </ol>
          )}
        </div>
      }
      controls={
        <>
          <div className="time-auction-bank">
            <div className="label">Your bank</div>
            <div className="time-auction-bar">
              <div style={{ width: `${(100 * v.bankMs) / v.startBankMs}%` }} />
              <span>{secs(v.bankMs)}</span>
            </div>
          </div>
          <p className="ask">{status(v)}</p>
          {(v.phase === 'result' || v.phase === 'over') && v.lastHeldMs > 0 && <p className="last">You held {secs(v.lastHeldMs)}.</p>}
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
            {v.holding ? 'HOLDING' : 'HOLD'}
          </button>
          <KeyHint>Or hold Space.</KeyHint>
        </>
      }
    />
  );
}
