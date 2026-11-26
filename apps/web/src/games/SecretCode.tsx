// Secret Code: crack four dice from exact / near hints. Rules page: docs/games/secret-code.md

import { useState } from 'react';
import { t } from '../i18n';
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

/**
 * A die face cut like a gem (docs/design.md, "Icons are faceted"): bevels lit from the top left,
 * a white glint, hexagonal pips shaded on their lower right. An empty slot is a dashed outline.
 */
function Die({ face }: { face: number | null }) {
  return (
    <svg className={`sc-die${face ? '' : ' empty'}`} viewBox="0 0 40 40" aria-label={face ? `${face}` : t('empty')} role="img">
      {face ? (
        <>
          <path className="sc-die-body" d="M7,2 33,2 38,7 38,33 33,38 7,38 2,33 2,7Z" />
          <path fill="#fff" fillOpacity=".5" d="M2,7 7,2 9,5 5,9Z" />
          <path fill="#fff" fillOpacity=".4" d="M7,2 33,2 31,5 9,5Z" />
          <path fill="#fff" fillOpacity=".3" d="M2,33 2,7 5,9 5,31Z" />
          <path fill="#000" fillOpacity=".25" d="M38,7 38,33 35,31 35,9Z" />
          <path fill="#000" fillOpacity=".3" d="M33,38 7,38 9,35 31,35Z" />
          <path fill="#000" fillOpacity=".35" d="M38,33 33,38 31,35 35,31Z" />
          <path fill="#fff" d="M5.2,4.1 6.3,5.2 5.2,6.3 4.1,5.2Z" />
          {PIPS[face]?.map(([x, y]) => (
            <g key={`${x}${y}`} transform={`translate(${x * 10} ${y * 10})`}>
              <path className="sc-pip" d="M0,-4.1 3.55,-2.05 3.55,2.05 0,4.1 -3.55,2.05 -3.55,-2.05Z" />
              <path fill="#000" fillOpacity=".3" d="M3.55,-2.05 3.55,2.05 0,4.1 -3.55,2.05Z" />
            </g>
          ))}
        </>
      ) : (
        <rect x="2" y="2" width="36" height="36" rx="8" />
      )}
    </svg>
  );
}

function Hint({ exact, near }: { exact: number; near: number }) {
  return (
    <div className="sc-hint" title={t('{exact} exact, {near} near', { exact, near })}>
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
          <GameTop title="Secret Code" step={t('Code {n} of {total}', { n: view.round, total: view.rounds })} score={me?.points ?? 0} />
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
              {t('Total')} <strong>{view.total}</strong> {t('pips')}
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
                      aria-label={t('Remove die {n}', { n: i + 1 })}
                      onClick={() => setEntry((e) => e.filter((_, j) => j !== i))}
                    >
                      <Die face={entry[i] ?? null} />
                    </button>
                  ))}
                </div>
                <span className="sc-left muted small">{t('{n} left', { n: left })}</span>
              </li>
            )}
          </ol>
          <p className="muted small center">
            <span className="sc-exact">n</span> {t('right face, right place')} · <span className="sc-near">n</span>{' '}
            {t('right face, wrong place')}
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
                    <span className="score">{s.roundPoints > 0 ? `+${s.roundPoints}` : t('not solved')}</span>
                  </li>
                ))}
              </ol>
              <button type="button" className="btn primary" onClick={() => move({ type: 'next' })}>
                {view.round < view.rounds ? t('Next code') : t('Finish')}
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
                  {t('Guess')}
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
                    {r.solved
                      ? t('cracked it in {n}', { n: r.guesses })
                      : t(r.guesses === 1 ? '{n} guess' : '{n} guesses', { n: r.guesses })}
                  </span>
                  <span className="score">{view.scores.find((s) => s.id === r.id)?.points ?? 0}</span>
                </li>
              ))}
            </ul>
          )}
          <KeyHint>{t('Keys: 1–6 add a die, Backspace removes, Enter guesses.')}</KeyHint>
        </>
      }
    />
  );
}
