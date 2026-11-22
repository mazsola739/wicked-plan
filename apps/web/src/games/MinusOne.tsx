// Minus One: show two cards, keep one; the lowest unique card wins. Rules: docs/games/minus-one.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './MinusOne.css';

type Phase = 'show' | 'keep' | 'result' | 'over';

interface MoView {
  phase: Phase;
  round: number;
  rounds: number;
  timeLeftMs: number;
  phaseMs: number;
  hand: number[];
  myPair: [number, number] | null;
  myKept: number | null;
  seats: { id: string; score: number; handSize: number; done: boolean; pair: [number, number] | null; kept: number | null }[];
  log: { round: number; kept: [string, number][]; winner: string | null; pot: number }[];
}

function Card({ n, dim, on, small }: { n: number; dim?: boolean; on?: boolean; small?: boolean }) {
  return <span className={`minus-one-card${small ? ' small' : ''}${dim ? ' dim' : ''}${on ? ' on' : ''}`}>{n}</span>;
}

export default function MinusOne({ view }: ScreenProps) {
  const v = view as MoView;
  const [picked, setPicked] = useState<number[]>([]);
  const [forKey, setForKey] = useState('');
  const key = `${v.round}-${v.phase}`;
  if (forKey !== key) {
    setForKey(key);
    setPicked([]);
  }
  const toggle = (c: number) => setPicked((cur) => (cur.includes(c) ? cur.filter((x) => x !== c) : [...cur, c].slice(-2)));
  const canShow = v.phase === 'show' && !v.myPair;
  const canKeep = v.phase === 'keep' && v.myKept === null && v.myPair !== null;
  const show = () => picked.length === 2 && move({ type: 'show', cards: picked });
  const keep = (card: number) => move({ type: 'keep', card });
  useKeys((k) => {
    const n = k === '0' ? 10 : Number(k);
    if (/^[0-9]$/.test(k)) {
      if (canShow && v.hand.includes(n)) toggle(n);
      else if (canKeep && v.myPair?.includes(n)) keep(n);
      else return false;
    } else if (k === 'Enter' && canShow) show();
    else return false;
    return true;
  });
  const me = v.seats.find((s) => s.id === 'you');
  const last = v.log[v.log.length - 1];
  const winnerKept = last?.kept.find(([id]) => id === last.winner)?.[1];
  let ask = '';
  if (v.phase === 'show') ask = v.myPair ? 'Waiting for the others to show…' : 'Pick two cards to show.';
  else if (v.phase === 'keep') ask = v.myKept !== null ? `You keep ${v.myKept}. Waiting…` : 'Keep one of your two cards.';
  else if (last)
    ask = last.winner
      ? `${nameOf(last.winner)} ${last.winner === 'you' ? 'win' : 'wins'} ${last.pot} with the ${winnerKept}.`
      : 'No unique card: nobody scores.';
  return (
    <Play
      top={
        <>
          <GameTop title="Minus One" step={`Round ${v.round} of ${v.rounds}`} score={me?.score ?? 0} />
          <TimerBar leftMs={v.timeLeftMs} totalMs={v.phaseMs} />
        </>
      }
      stage={
        <div className="card">
          <ul className="list">
            {v.seats.map((s) => {
              const won = (v.phase === 'result' || v.phase === 'over') && last?.winner === s.id;
              return (
                <li key={s.id} className={won ? 'minus-one-won' : ''}>
                  <span>
                    {nameOf(s.id)} <span className="score">{s.score}</span>
                  </span>
                  <span className="minus-one-seat">
                    {s.pair ? (
                      s.pair.map((c) => <Card key={c} n={c} small dim={s.kept !== null && s.kept !== c} on={s.kept === c} />)
                    ) : (
                      <span className="muted small">{s.done ? 'ready' : 'thinking…'}</span>
                    )}
                  </span>
                </li>
              );
            })}
          </ul>
          {v.log.length > 0 && (
            <ul className="minus-one-log muted small">
              {v.log
                .slice()
                .reverse()
                .map((l) => (
                  <li key={l.round}>
                    Round {l.round}: kept {l.kept.map(([, c]) => c).join(', ')} → {l.winner ? `${nameOf(l.winner)} +${l.pot}` : 'no winner'}
                  </li>
                ))}
            </ul>
          )}
        </div>
      }
      controls={
        <>
          <p className="ask">{ask}</p>
          {canKeep && v.myPair ? (
            <div className="row">
              {v.myPair.map((c) => (
                <button key={c} type="button" className="minus-one-big" onClick={() => keep(c)}>
                  {c}
                  <small>keep</small>
                </button>
              ))}
            </div>
          ) : (
            <div className="minus-one-hand">
              {Array.from({ length: 10 }, (_, i) => i + 1).map((c) => {
                const has = v.hand.includes(c);
                const inPair = v.myPair?.includes(c);
                return (
                  <button
                    key={c}
                    type="button"
                    className={`card-btn${has && (picked.includes(c) || inPair) ? ' picked' : ''}`}
                    disabled={!has || !canShow}
                    onClick={() => toggle(c)}
                  >
                    {has ? c : ''}
                  </button>
                );
              })}
            </div>
          )}
          {canShow && (
            <button type="button" className="btn primary" disabled={picked.length !== 2} onClick={show}>
              Show {picked.length === 2 ? `${picked[0]} and ${picked[1]}` : 'two cards'}
            </button>
          )}
          <KeyHint>Keys 1–9 and 0 (for 10) pick cards, Enter shows.</KeyHint>
        </>
      }
    />
  );
}
