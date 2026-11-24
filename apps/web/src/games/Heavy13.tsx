// Heavy 13: add cards to coloured stacks without going over 13. Rules: docs/games/heavy-13.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './Heavy13.css';

type Phase = 'play' | 'result' | 'over';
type Outcome = 'stays' | 'full' | 'overloaded';

interface PlayT {
  card: number | null;
  stack: number;
  rainbow: boolean;
}

interface HvView {
  phase: Phase;
  round: number;
  rounds: number;
  stacks: [number, number, number, number];
  hand: number[];
  rainbow: boolean;
  passes: number;
  myPlay: PlayT | null;
  seats: { id: string; score: number; cards: number; rainbow: boolean; passes: number; done: boolean }[];
  log: { round: number; plays: [string, PlayT, number][]; stacks: { before: number; added: number; outcome: Outcome }[] }[];
}

const COLOURS = ['red', 'blue', 'green', 'gold'];
const KEYS = ['q', 'w', 'e', 'r'];
const val = (c: number) => (c % 6) + 1;
const col = (c: number) => Math.floor(c / 6);

function Rock({ c, on, onClick, disabled }: { c: number; on?: boolean; onClick?: () => void; disabled?: boolean }) {
  return (
    <button type="button" className={`heavy-13-card heavy-13-${COLOURS[col(c)]}${on ? ' on' : ''}`} onClick={onClick} disabled={disabled}>
      {val(c)}
    </button>
  );
}

function describe(p: PlayT): string {
  if (p.card === null) return 'passed';
  return `${COLOURS[col(p.card)]} ${val(p.card)} → ${COLOURS[p.stack]}${p.rainbow ? ' (rainbow)' : ''}`;
}

export default function Heavy13({ view }: ScreenProps) {
  const v = view as HvView;
  const [picked, setPicked] = useState<number | null>(null);
  const [forRound, setForRound] = useState(v.round);
  if (forRound !== v.round) {
    setForRound(v.round);
    setPicked(null);
  }
  const canPlay = v.phase === 'play' && !v.myPlay;
  const card = picked !== null && v.hand.includes(picked) ? picked : null;
  const playOn = (stack: number) => {
    if (card === null || !canPlay) return;
    if (stack !== col(card) && !v.rainbow) return;
    move({ type: 'play', card, stack: stack === col(card) ? null : stack });
  };
  const pass = () => canPlay && v.passes > 0 && move({ type: 'pass' });
  useKeys((k) => {
    const n = Number(k);
    if (canPlay && n >= 1 && n <= v.hand.length) setPicked(v.hand[n - 1] ?? null);
    else if (KEYS.includes(k.toLowerCase())) playOn(KEYS.indexOf(k.toLowerCase()));
    else if (k === 'Enter' && card !== null) playOn(col(card));
    else if (k === 'p' || k === 'P') pass();
    else return false;
    return true;
  });
  const me = v.seats.find((s) => s.id === 'you');
  const last = v.log[v.log.length - 1];
  const showResult = (v.phase === 'result' || v.phase === 'over') && last;
  let ask = '';
  if (v.phase === 'play')
    ask = v.myPlay ? `You ${describe(v.myPlay)}. Waiting…` : card === null ? 'Pick a card, or pass.' : 'Tap a stack to play it there.';
  else if (last) {
    const mine = last.plays.find(([id]) => id === 'you');
    ask = mine ? `You ${describe(mine[1])}: ${mine[2] >= 0 ? '+' : ''}${mine[2]}.` : '';
  }
  return (
    <Play
      top={<GameTop title="Heavy 13" step={`Round ${v.round} of ${v.rounds}`} score={me?.score ?? 0} />}
      stage={
        <div className="card">
          <div className="heavy-13-stacks">
            {COLOURS.map((name, k) => {
              const log = showResult ? last.stacks[k] : undefined;
              const total = log ? log.before + log.added : (v.stacks[k] ?? 0);
              const target = card !== null && canPlay && (k === col(card) || v.rainbow);
              return (
                <button
                  key={name}
                  type="button"
                  className={`heavy-13-stack heavy-13-${name}${target ? ' target' : ''}${log ? ` ${log.outcome}` : ''}`}
                  disabled={!target}
                  onClick={() => playOn(k)}
                >
                  <span className="heavy-13-fill" style={{ height: `${(Math.min(total, 13) / 13) * 100}%` }} />
                  <strong>{total}</strong>
                  <small>{log ? (log.outcome === 'stays' ? `+${log.added}` : log.outcome) : `${13 - total} left`}</small>
                </button>
              );
            })}
          </div>
          <ul className="list">
            {v.seats.map((s) => {
              const play = showResult ? last.plays.find(([id]) => id === s.id) : undefined;
              return (
                <li key={s.id}>
                  <span>
                    {nameOf(s.id)} <span className="score">{s.score}</span>{' '}
                    <span className="muted small">
                      {play
                        ? `${describe(play[1])} ${play[2] >= 0 ? '+' : ''}${play[2]}`
                        : `${s.cards} cards · ${s.passes} passes${s.rainbow ? ' · chip' : ''}`}
                    </span>
                  </span>
                  {v.phase === 'play' && <span className="muted small">{s.done ? 'ready' : 'thinking…'}</span>}
                </li>
              );
            })}
          </ul>
        </div>
      }
      controls={
        <>
          <p className="ask">{ask}</p>
          <div className="heavy-13-hand">
            {v.hand.map((c, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: a hand may hold two equal cards
              <Rock key={`${c}-${i}`} c={c} on={card === c} disabled={!canPlay} onClick={() => setPicked(c)} />
            ))}
          </div>
          <div className="row">
            <button type="button" className="btn" disabled={!canPlay || v.passes === 0} onClick={pass}>
              Pass ({v.passes} left)
            </button>
            <button
              type="button"
              className="btn primary"
              disabled={!canPlay || card === null}
              onClick={() => card !== null && playOn(col(card))}
            >
              Play on its colour
            </button>
          </div>
          <p className="muted small center">{v.rainbow ? 'Rainbow chip ready: tap another stack to use it.' : 'Rainbow chip used.'}</p>
          <KeyHint>1–8 pick a card · Q W E R stack · Enter its own stack · P pass.</KeyHint>
        </>
      }
    />
  );
}
