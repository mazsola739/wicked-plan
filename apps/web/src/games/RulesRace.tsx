// Rules Race: race to square 18, secret personal rules, group rules by vote. Rules: docs/games/rules-race.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './RulesRace.css';

interface RuleView {
  condition: string;
  effect: string;
  text: string;
}

interface RacerView {
  id: string;
  pos: number;
  inPrison: boolean;
  escape: boolean;
  rule: RuleView | null;
  written: boolean;
  voted: boolean;
}

interface Choice {
  id: string;
  text: string;
}

interface RaceView {
  phase: 'write' | 'turn' | 'vote' | 'over';
  track: number;
  round: number;
  maxRounds: number;
  timeLeftMs: number;
  phaseTimeMs: number;
  racers: RacerView[];
  current: string | null;
  lastRoll: string | null;
  group: { text: string; on: boolean }[];
  conditions: Choice[];
  effects: Choice[];
  log: string[];
  score: number;
}

const SEAT_COLOURS = ['#c4183a', '#d4a637', '#3aa86b', '#4a8fd9', '#a45bd6', '#e07b39', '#38b6b0', '#d65b9c'];
const COLS = 5;
const CELL = 96;

/** Serpentine track: row 0 left to right, row 1 right to left, … */
function cellXY(n: number): [number, number] {
  const row = Math.floor(n / COLS);
  const col = row % 2 === 0 ? n % COLS : COLS - 1 - (n % COLS);
  return [col * CELL, row * CELL];
}

/** Replaces seat ids in log lines with display names. */
function names(line: string): string {
  return line
    .replace(/\b(you|bot-\d+)\b/g, (id) => nameOf(id))
    .replace(/\bYou's\b/g, 'Your')
    .replace(/\bYou (has|goes|\w+s)\b/g, (_, w: string) => `You ${w === 'has' ? 'have' : w === 'goes' ? 'go' : w.slice(0, -1)}`);
}

export default function RulesRace({ view }: ScreenProps) {
  const v = view as RaceView;
  const me = v.racers.find((r) => r.id === 'you');
  const myTurn = v.phase === 'turn' && v.current === 'you';
  useKeys((key) => {
    if ((key === ' ' || key === 'Enter') && myTurn) move({ type: 'roll' });
    else return false;
    return true;
  });
  const rows = Math.ceil((v.track + 1) / COLS);
  const step = v.phase === 'write' ? 'Write your rule' : v.phase === 'vote' ? 'Group vote' : `Round ${v.round} of ${v.maxRounds}`;
  return (
    <Play
      top={
        <>
          <GameTop title="Rules Race" step={step} score={v.score} />
          <TimerBar leftMs={v.timeLeftMs} totalMs={v.phaseTimeMs} />
        </>
      }
      stage={
        <>
          <svg
            className="drawing rules-race-track"
            viewBox={`-4 -4 ${COLS * CELL + 8} ${rows * CELL + 8}`}
            role="img"
            aria-label="Race track"
          >
            {Array.from({ length: v.track + 1 }, (_, n) => {
              const [x, y] = cellXY(n);
              const cls = n === v.track ? ' goal' : n > 0 && n % 4 === 0 ? ' four' : '';
              return (
                // biome-ignore lint/suspicious/noArrayIndexKey: squares are positional
                <g key={n}>
                  <rect x={x + 3} y={y + 3} width={CELL - 6} height={CELL - 6} rx={12} className={`rules-race-cell${cls}`} />
                  <text x={x + 12} y={y + 24} className="rules-race-num">
                    {n === v.track ? 'GOAL' : n}
                  </text>
                </g>
              );
            })}
            {v.racers.map((r, i) => {
              if (r.inPrison) return null;
              const [x, y] = cellXY(r.pos);
              const here = v.racers.filter((o) => !o.inPrison && o.pos === r.pos);
              const k = here.indexOf(r);
              const px = x + 22 + (k % 3) * 26;
              const py = y + 52 + Math.floor(k / 3) * 26;
              return (
                <g key={r.id}>
                  <circle cx={px} cy={py} r={13} fill={SEAT_COLOURS[i]} className={r.id === v.current ? 'rules-race-now' : ''} />
                  <text x={px} y={py + 5} className="rules-race-piece">
                    {nameOf(r.id)[0]}
                  </text>
                </g>
              );
            })}
          </svg>
          <div className="rules-race-prison">
            <span className="label">Prison</span>
            {v.racers
              .filter((r) => r.inPrison)
              .map((r) => (
                <span key={r.id} className="rules-race-chip" style={{ background: SEAT_COLOURS[v.racers.indexOf(r)] }}>
                  {nameOf(r.id)}
                </span>
              ))}
          </div>
          <ul className="list rules-race-players">
            {v.racers.map((r, i) => (
              <li key={r.id} className={r.id === 'you' ? 'me' : ''}>
                <span>
                  <span className="rules-race-dot" style={{ background: SEAT_COLOURS[i] }} />
                  {nameOf(r.id)} · {r.pos}
                  {r.escape ? ' · escape card' : ''}
                </span>
                <small className="muted">{r.rule ? r.rule.text : r.written ? 'secret rule' : 'writing…'}</small>
              </li>
            ))}
          </ul>
        </>
      }
      controls={
        <>
          {v.phase === 'write' && me && !me.written && <WriteRule view={v} />}
          {v.phase === 'write' && me?.written && <p className="muted center">Waiting for the others to write their rules…</p>}
          {v.phase === 'turn' && (
            <>
              {v.lastRoll && <p className="last">{names(v.lastRoll)}</p>}
              <button type="button" className="btn primary" disabled={!myTurn} onClick={() => move({ type: 'roll' })}>
                {myTurn ? 'Roll the die' : `${nameOf(v.current ?? '')} is rolling…`}
              </button>
              <KeyHint>Space or Enter rolls.</KeyHint>
            </>
          )}
          {v.phase === 'vote' && <Vote view={v} voted={!!me?.voted} />}
          <div>
            <div className="label">Group rules</div>
            <ol className="rules-race-group">
              {v.group.map((g) => (
                <li key={g.text} className={g.on ? 'on' : 'off'}>
                  <strong>{g.on ? 'ON' : 'off'}</strong> {g.text}
                </li>
              ))}
            </ol>
          </div>
          <div className="rules-race-log">
            {v.log
              .slice(-6)
              .reverse()
              .map((line, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows at the end
                <p key={i} className="small">
                  {names(line)}
                </p>
              ))}
          </div>
        </>
      }
    />
  );
}

function WriteRule({ view }: { view: RaceView }) {
  const [cond, setCond] = useState<string | null>(null);
  const [effect, setEffect] = useState<string | null>(null);
  return (
    <div className="rules-race-write">
      <div className="label">When…</div>
      {view.conditions.map((c) => (
        <button key={c.id} type="button" className={`btn small${cond === c.id ? ' primary' : ''}`} onClick={() => setCond(c.id)}>
          {c.text}
        </button>
      ))}
      <div className="label">then…</div>
      {view.effects.map((e) => (
        <button key={e.id} type="button" className={`btn small${effect === e.id ? ' primary' : ''}`} onClick={() => setEffect(e.id)}>
          {e.text}
        </button>
      ))}
      <button
        type="button"
        className="btn primary"
        disabled={!cond || !effect}
        onClick={() => move({ type: 'write', condition: cond, effect })}
      >
        Write my secret rule
      </button>
    </div>
  );
}

function Vote({ view, voted }: { view: RaceView; voted: boolean }) {
  if (voted) return <p className="muted center">Vote sent. Waiting for the others…</p>;
  return (
    <div className="rules-race-write">
      <div className="label">Switch one group rule?</div>
      {view.group.map((g, k) => (
        <button key={g.text} type="button" className="btn small" onClick={() => move({ type: 'vote', rule: k })}>
          Switch {k + 1} {g.on ? 'off' : 'on'}
        </button>
      ))}
      <button type="button" className="btn primary" onClick={() => move({ type: 'vote', rule: null })}>
        Keep them all
      </button>
    </div>
  );
}
