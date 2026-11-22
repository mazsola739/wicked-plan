// Secret Code: crack four dice from exact / near hints. Rules page: docs/games/secret-code.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './SecretCode.css';

interface Guess {
  code: number[];
  exact: number;
  near: number;
}

interface SecretCodeView {
  round: number;
  rounds: number;
  total: number;
  timeLeftMs: number;
  roundMs: number;
  maxGuesses: number;
  guesses: Guess[];
  done: boolean;
  rivals: { id: string; guesses: number; solved: boolean }[];
  revealing: boolean;
  revealLeftMs: number;
  code: number[] | null;
  scores: { id: string; points: number; roundPoints: number }[];
  over: boolean;
}

const PIPS: Record<number, [number, number][]> = {
  1: [[2, 2]],
  2: [
    [1, 1],
    [3, 3],
  ],
  3: [
    [1, 1],
    [2, 2],
    [3, 3],
  ],
  4: [
    [1, 1],
    [3, 1],
    [1, 3],
    [3, 3],
  ],
  5: [
    [1, 1],
    [3, 1],
    [2, 2],
    [1, 3],
    [3, 3],
  ],
  6: [
    [1, 1],
    [3, 1],
    [1, 2],
    [3, 2],
    [1, 3],
    [3, 3],
  ],
};

function Die({ face }: { face: number | null }) {
  return (
    <svg className={`sc-die${face ? '' : ' empty'}`} viewBox="0 0 40 40" aria-label={face ? `${face}` : 'empty'} role="img">
      <rect x="2" y="2" width="36" height="36" rx="8" />
      {face && PIPS[face]?.map(([x, y]) => <circle key={`${x}${y}`} cx={x * 10} cy={y * 10} r="3.6" />)}
    </svg>
  );
}

function Hint({ exact, near }: { exact: number; near: number }) {
  return (
    <div className="sc-hint" title={`${exact} exact, ${near} near`}>
      <span className="sc-exact">{exact}</span>
      <span className="sc-near">{near}</span>
    </div>
  );
}

export default function SecretCode({ view: raw }: ScreenProps) {
  const view = raw as SecretCodeView;
  const [entry, setEntry] = useState<number[]>([]);
  const [forCount, setForCount] = useState(`${view.round}/${view.guesses.length}`);
  const key = `${view.round}/${view.guesses.length}`;
  if (forCount !== key) {
    setForCount(key);
    setEntry([]);
  }
  const canGuess = !view.revealing && !view.done;
  const add = (d: number) => canGuess && setEntry((e) => (e.length < 4 ? [...e, d] : e));
  const submit = () => entry.length === 4 && move({ type: 'guess', code: entry });
  useKeys((k) => {
    if (/^[1-6]$/.test(k)) add(Number(k));
    else if (k === 'Backspace') setEntry((e) => e.slice(0, -1));
    else if (k === 'Enter') view.revealing ? move({ type: 'next' }) : submit();
    else return false;
    return true;
  });
  const me = view.scores[0];
  const left = view.maxGuesses - view.guesses.length;
  return (
    <Play
      top={
        <>
          <GameTop title="Secret Code" step={`Code ${view.round} of ${view.rounds}`} score={me?.points ?? 0} />
          <TimerBar leftMs={view.revealing ? view.revealLeftMs : view.timeLeftMs} totalMs={view.revealing ? 4000 : view.roundMs} />
        </>
      }
      stage={
        <div className="sc-board">
          <div className="sc-cup">
            {view.code ? (
              <div className="sc-row">
                {view.code.map((d, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: dice are positional
                  <Die key={i} face={d} />
                ))}
              </div>
            ) : (
              <div className="sc-hidden">? ? ? ?</div>
            )}
            <div className="sc-total">
              Total <strong>{view.total}</strong> pips
            </div>
          </div>
          <ol className="sc-guesses">
            {view.guesses.map((g, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: guesses only grow
              <li key={i} className={g.exact === 4 ? 'solved' : ''}>
                <span className="sc-n">{i + 1}</span>
                <div className="sc-row">
                  {g.code.map((d, j) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: dice are positional
                    <Die key={j} face={d} />
                  ))}
                </div>
                <Hint exact={g.exact} near={g.near} />
              </li>
            ))}
            {canGuess && (
              <li className="sc-entry">
                <span className="sc-n">{view.guesses.length + 1}</span>
                <div className="sc-row">
                  {[0, 1, 2, 3].map((i) => (
                    <button
                      key={i}
                      type="button"
                      className="sc-slot"
                      aria-label={`Remove die ${i + 1}`}
                      onClick={() => setEntry((e) => e.filter((_, j) => j !== i))}
                    >
                      <Die face={entry[i] ?? null} />
                    </button>
                  ))}
                </div>
                <span className="sc-left muted small">{left} left</span>
              </li>
            )}
          </ol>
          <p className="muted small center">
            <span className="sc-exact">n</span> right face, right place · <span className="sc-near">n</span> right face, wrong place
          </p>
        </div>
      }
      controls={
        <>
          {view.revealing ? (
            <>
              <ol className="list">
                {view.scores.map((s) => (
                  <li key={s.id} className={s === me ? 'me' : ''}>
                    <span>{nameOf(s.id)}</span>
                    <span className="score">{s.roundPoints > 0 ? `+${s.roundPoints}` : 'not solved'}</span>
                  </li>
                ))}
              </ol>
              <button type="button" className="btn primary" onClick={() => move({ type: 'next' })}>
                {view.round < view.rounds ? 'Next code' : 'Finish'}
              </button>
            </>
          ) : (
            <>
              <div className="sc-faces">
                {[1, 2, 3, 4, 5, 6].map((d) => (
                  <button key={d} type="button" className="key" disabled={!canGuess || entry.length === 4} onClick={() => add(d)}>
                    <Die face={d} />
                  </button>
                ))}
              </div>
              <div className="row">
                <button type="button" className="btn ghost" disabled={entry.length === 0} onClick={() => setEntry((e) => e.slice(0, -1))}>
                  ⌫
                </button>
                <button type="button" className="btn primary" disabled={!canGuess || entry.length !== 4} onClick={submit}>
                  Guess
                </button>
              </div>
            </>
          )}
          {view.rivals.length > 0 && (
            <ul className="list sc-rivals">
              {view.rivals.map((r) => (
                <li key={r.id}>
                  <span>{nameOf(r.id)}</span>
                  <span className="muted small">
                    {r.solved ? `cracked it in ${r.guesses}` : `${r.guesses} guess${r.guesses === 1 ? '' : 'es'}`}
                  </span>
                  <span className="score">{view.scores.find((s) => s.id === r.id)?.points ?? 0}</span>
                </li>
              ))}
            </ul>
          )}
          <KeyHint>Keys: 1–6 add a die, Backspace removes, Enter guesses.</KeyHint>
        </>
      }
    />
  );
}
