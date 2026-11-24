// Unknown: race to the centre and work out each colour's secret rule. Rules: docs/games/hidden-rules-race.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './HiddenRulesRace.css';

type Dir = 'up' | 'right' | 'down' | 'left';
type Pos = [number, number];

interface Step {
  player: string;
  colour: number;
  dir: Dir | null;
  from: Pos;
  to: Pos;
}

interface HrrView {
  phase: 'choose' | 'reveal' | 'over';
  size: number;
  goal: Pos;
  colours: string[];
  ruleNames: string[];
  board: number[];
  pieces: { id: string; pos: Pos; chosen: boolean; distance: number }[];
  round: number;
  maxRounds: number;
  known: { colour: number; rule: string }[];
  history: Step[][];
  myChoice: Dir | null;
  score: number;
}

const ARROW: Record<Dir, string> = { up: '↑', right: '→', down: '↓', left: '←' };
const KEYS: Record<string, Dir> = { ArrowUp: 'up', ArrowRight: 'right', ArrowDown: 'down', ArrowLeft: 'left' };

/** The goal: a faceted star, every arm cut along its ridge and lit from the top left. */
function Star() {
  return (
    <svg viewBox="0 0 40 40" className="hrr-star" role="img" aria-label="goal">
      <path d="M20,2 24.7,14.5 38.1,15.1 27.6,23.5 31.2,36.4 20,29 8.8,36.4 12.4,23.5 1.9,15.1 15.3,14.5Z" />
      <path fill="#fff" fillOpacity=".42" d="M1.9,15.1 15.3,14.5 20,21Z" />
      <path fill="#fff" fillOpacity=".34" d="M20,2 15.3,14.5 20,21Z" />
      <path fill="#fff" fillOpacity=".26" d="M1.9,15.1 12.4,23.5 20,21Z" />
      <path fill="#fff" fillOpacity=".18" d="M20,2 24.7,14.5 20,21Z" />
      <path fill="#fff" fillOpacity=".1" d="M8.8,36.4 12.4,23.5 20,21Z" />
      <path fill="#000" fillOpacity=".12" d="M38.1,15.1 24.7,14.5 20,21Z M8.8,36.4 20,29 20,21Z" />
      <path fill="#000" fillOpacity=".25" d="M38.1,15.1 27.6,23.5 20,21Z" />
      <path fill="#000" fillOpacity=".3" d="M31.2,36.4 20,29 20,21Z" />
      <path fill="#000" fillOpacity=".35" d="M31.2,36.4 27.6,23.5 20,21Z" />
      <path fill="#fff" d="M11.5,15.4 12.7,16.6 11.5,17.8 10.3,16.6Z M18.6,8.6 19.4,9.4 18.6,10.2 17.8,9.4Z" />
    </svg>
  );
}

function describe(s: Step): string {
  if (!s.dir) return 'stayed (no choice)';
  const dx = s.to[0] - s.from[0];
  const dy = s.to[1] - s.from[1];
  if (dx === 0 && dy === 0) return `${ARROW[s.dir]} · did not move`;
  const n = Math.abs(dx) + Math.abs(dy);
  const way = dx > 0 ? '→' : dx < 0 ? '←' : dy > 0 ? '↓' : '↑';
  return `${ARROW[s.dir]} · moved ${n} ${way}`;
}

export default function HiddenRulesRace({ view }: ScreenProps) {
  const v = view as HrrView;
  const [notes, setNotes] = useState<number[]>([-1, -1, -1, -1, -1]);
  const choosing = v.phase === 'choose' && v.myChoice === null;
  const go = (dir: Dir) => choosing && move({ type: 'go', dir });
  useKeys((key) => {
    if (KEYS[key]) go(KEYS[key]);
    else if (key === 'Enter' && v.phase === 'reveal') move({ type: 'close' });
    else return false;
    return true;
  });
  const knownOf = (c: number) => v.known.find((k) => k.colour === c)?.rule;
  const cycle = (c: number) => setNotes((n) => n.map((x, i) => (i === c ? ((x + 2) % (v.ruleNames.length + 1)) - 1 : x)));
  const last = v.history[0] ?? [];
  const me = v.pieces.find((p) => p.id === 'you');
  return (
    <Play
      top={
        <GameTop
          title="Unknown"
          step={v.phase === 'choose' ? `Round ${v.round} of ${v.maxRounds}` : 'The rules revealed'}
          score={v.score}
        />
      }
      stage={
        <div className="hrr-board" style={{ gridTemplateColumns: `repeat(${v.size}, 1fr)` }}>
          {v.board.map((c, i) => {
            const x = i % v.size;
            const y = Math.floor(i / v.size);
            const here = v.pieces.filter((p) => p.pos[0] === x && p.pos[1] === y);
            const goal = x === v.goal[0] && y === v.goal[1];
            return (
              <div key={`${x},${y}`} className={`hrr-sq hrr-${goal ? 'goal' : v.colours[c]}`}>
                {goal && <Star />}
                {here.map((p) => (
                  <span key={p.id} className={`hrr-piece${p.id === 'you' ? ' me' : ''}`} title={nameOf(p.id)}>
                    {nameOf(p.id)[0]}
                  </span>
                ))}
              </div>
            );
          })}
        </div>
      }
      controls={
        <>
          {v.phase === 'choose' ? (
            <>
              <p className="muted small center">
                {v.myChoice
                  ? `You chose ${ARROW[v.myChoice]}. Waiting for the others…`
                  : `You stand on ${me ? v.colours[v.board[me.pos[1] * v.size + me.pos[0]] ?? 0] : '?'}. Pick a direction.`}
              </p>
              <div className="hrr-pad">
                {(['up', 'left', 'right', 'down'] as Dir[]).map((d) => (
                  <button
                    key={d}
                    type="button"
                    className={`btn hrr-${d}${v.myChoice === d ? ' primary' : ''}`}
                    disabled={!choosing}
                    onClick={() => go(d)}
                  >
                    {ARROW[d]}
                  </button>
                ))}
              </div>
              <KeyHint>Arrow keys choose a direction.</KeyHint>
            </>
          ) : (
            <button type="button" className="btn primary" onClick={() => move({ type: 'close' })}>
              See results
            </button>
          )}
          <div>
            <div className="label">Colours {v.phase === 'choose' ? '(tap to note your guess)' : ''}</div>
            <ul className="list hrr-notes">
              {v.colours.map((name, c) => {
                const known = knownOf(c);
                return (
                  <li key={name}>
                    <span>
                      <span className={`hrr-swatch hrr-${name}`} /> {name}
                    </span>
                    {known ? (
                      <strong>{known}</strong>
                    ) : (
                      <button type="button" className="btn small ghost" onClick={() => cycle(c)}>
                        {(notes[c] ?? -1) >= 0 ? `${v.ruleNames[notes[c] ?? 0]}?` : '?'}
                      </button>
                    )}
                  </li>
                );
              })}
            </ul>
          </div>
          {last.length > 0 && (
            <div>
              <div className="label">Last round</div>
              <ul className="list hrr-log">
                {last.map((s) => (
                  <li key={s.player} className={s.player === 'you' ? 'me' : ''}>
                    <span>
                      <span className={`hrr-swatch hrr-${v.colours[s.colour]}`} /> {nameOf(s.player)}
                    </span>
                    <span className="small">{describe(s)}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </>
      }
    />
  );
}
