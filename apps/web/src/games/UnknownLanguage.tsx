// Unknown Language: crack a picture script from examples. Rules page: docs/games/unknown-language.md

import { GameTop, KeyHint, move, Play, RoundDots, type ScreenProps, TimerBar, useKeys } from './common';
import './UnknownLanguage.css';

interface ScriptWord {
  strokes: [number, number, number, number][];
  /** 0 bar under, 1 dot above, 2 ring left. */
  mark: number;
}

type Question =
  | { kind: 'read'; script: ScriptWord[]; options: string[] }
  | { kind: 'write'; english: string; options: ScriptWord[][] }
  | { kind: 'word'; word: ScriptWord; options: string[] };

interface LangView {
  stage: number;
  question: number;
  questions: number;
  examples: { script: ScriptWord[]; english: string }[];
  current: Question | null;
  last: { question: Question; given: number | null; solution: number; correct: boolean; points: number } | null;
  results: boolean[];
  score: number;
  timeLeftMs: number;
  questionTimeMs: number;
  over: boolean;
}

function Glyph({ word }: { word: ScriptWord }) {
  return (
    <svg viewBox="-13 -12 36 42" className="ul-glyph" aria-hidden="true">
      {word.strokes.map(([x1, y1, x2, y2]) => (
        <line key={`${x1}${y1}${x2}${y2}`} x1={x1 * 10} y1={y1 * 10} x2={x2 * 10} y2={y2 * 10} />
      ))}
      {word.mark === 0 && <line x1={3} y1={27} x2={17} y2={27} className="ul-mark" />}
      {word.mark === 1 && <circle cx={10} cy={-7} r={2.6} className="ul-dot" />}
      {word.mark === 2 && <circle cx={-8} cy={10} r={3.2} className="ul-ring" />}
    </svg>
  );
}

function Script({ words }: { words: ScriptWord[] }) {
  return (
    <span className="ul-script">
      {words.map((w, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: words are positional
        <Glyph key={i} word={w} />
      ))}
    </span>
  );
}

const STAGE_TEXT = ['Things and actions.', 'Now with qualities.', 'Known roots with a new mark: what do they mean?'];

function prompt(q: Question): React.ReactNode {
  if (q.kind === 'read')
    return (
      <>
        <p className="ask">What does this say?</p>
        <div className="ul-ask-script">
          <Script words={q.script} />
        </div>
      </>
    );
  if (q.kind === 'write') return <p className="ask">How is this written? “{q.english}”</p>;
  return (
    <>
      <p className="ask">What does this word mean?</p>
      <div className="ul-ask-script">
        <Glyph word={q.word} />
      </div>
    </>
  );
}

function OptionBody({ q, i }: { q: Question; i: number }) {
  if (q.kind === 'write') return <Script words={q.options[i] ?? []} />;
  return <span>{q.options[i]}</span>;
}

export default function UnknownLanguage({ view: raw }: ScreenProps) {
  const view = raw as LangView;
  const q = view.current;
  const answer = (option: number) => q && move({ type: 'answer', option });
  useKeys((key) => {
    if (/^[1-4]$/.test(key)) answer(Number(key) - 1);
    else return false;
    return true;
  });
  const last = view.last;
  return (
    <Play
      top={
        <>
          <GameTop
            title="Unknown Language"
            step={`Stage ${view.stage} of 3 · question ${view.question} of ${view.questions}`}
            score={view.score}
          />
          <RoundDots results={view.results} total={view.questions} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.questionTimeMs} />
        </>
      }
      stage={
        <div className="ul-wall">
          <div className="label">Carvings with a translation</div>
          <ul className="ul-examples">
            {view.examples.map((e) => (
              <li key={e.english}>
                <Script words={e.script} />
                <span className="ul-english">{e.english}</span>
              </li>
            ))}
          </ul>
        </div>
      }
      controls={
        <>
          <p className="muted small center">{STAGE_TEXT[view.stage - 1]}</p>
          {q && prompt(q)}
          {q && (
            <div className="ul-options">
              {q.options.map((_, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: options are positional
                <button key={i} type="button" className={`btn ul-option${q.kind === 'write' ? ' script' : ''}`} onClick={() => answer(i)}>
                  <span className="ul-num">{i + 1}</span>
                  <OptionBody q={q} i={i} />
                </button>
              ))}
            </div>
          )}
          {last && (
            <div className={`last ${last.correct ? 'ok' : 'bad'}`}>
              {last.correct ? `Right, +${last.points}` : last.given === null ? 'Out of time. ' : 'Not right. '}
              {!last.correct && (
                <>
                  It was: <OptionBody q={last.question} i={last.solution} />
                </>
              )}
            </div>
          )}
          <KeyHint>Keys: 1–4 pick an option.</KeyHint>
        </>
      }
    />
  );
}
