import { createContext, useContext, useEffect, useRef, useState } from 'react';
import { GameIcon } from '../GameIcon';

export { move } from '../play';

/** What every game screen gets: its own view (the shape is the game's own). */
export interface ScreenProps {
  // biome-ignore lint/suspicious/noExplicitAny: each screen casts to its own view type
  view: any;
}

const BOT_NAMES = ['Rex', 'Ivy', 'Max', 'Zoe', 'Kai', 'Lou', 'Nia', 'Oto', 'Pip'];

/** Display name of a seat: "you" → "You", "bot-1" → "Rex", … */
export function nameOf(id: string): string {
  if (id === 'you') return 'You';
  const n = Number(id.replace('bot-', ''));
  return BOT_NAMES[(n - 1) % BOT_NAMES.length] ?? id;
}

/** True inside a picker card's still preview: the screen must not listen to keys. */
export const PreviewContext = createContext(false);

/** The id of the game being played, so GameTop can show its icon beside the title. */
export const GameIdContext = createContext<string | null>(null);

/** Keyboard shortcuts for desktop; ignored while typing in a field. */
export function useKeys(handler: (key: string) => boolean | undefined): void {
  const ref = useRef(handler);
  ref.current = handler;
  const preview = useContext(PreviewContext);
  useEffect(() => {
    if (preview) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey || e.altKey || e.target instanceof HTMLInputElement) return;
      if (ref.current(e.key)) e.preventDefault();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [preview]);
}

/** The game layout: header on top; on a wide screen the board sits beside the controls. */
export function Play({ top, stage, controls }: { top: React.ReactNode; stage: React.ReactNode; controls: React.ReactNode }) {
  return (
    <div className="screen game">
      {top}
      <div className="play">
        <div className="stage">{stage}</div>
        <div className="controls">{controls}</div>
      </div>
    </div>
  );
}

export function KeyHint({ children }: { children: React.ReactNode }) {
  return <p className="key-hint muted small center">{children}</p>;
}

export function TimerBar({ leftMs, totalMs }: { leftMs: number; totalMs: number }) {
  const ratio = totalMs > 0 ? Math.max(0, Math.min(1, leftMs / totalMs)) : 0;
  const secs = Math.ceil(leftMs / 1000);
  const text = secs >= 60 ? `${Math.floor(secs / 60)}:${String(secs % 60).padStart(2, '0')}` : `${secs} s`;
  return (
    <div className="timer">
      <div className={`timer-fill${ratio < 0.25 ? ' low' : ''}`} style={{ width: `${ratio * 100}%` }} />
      <span>{text}</span>
    </div>
  );
}

export function GameTop({ title, step, score }: { title: string; step: string; score: number }) {
  const id = useContext(GameIdContext);
  return (
    <header className="game-top">
      <div className="game-title">
        {id && <GameIcon id={id} className="title-icon" />}
        <div>
          <div className="label">{step}</div>
          <h2>{title}</h2>
        </div>
      </div>
      <div className="score big">{score}</div>
    </header>
  );
}

/** Round results as dots: green right, red wrong, grey to come. */
export function RoundDots({ results, total }: { results: boolean[]; total: number }) {
  return (
    <div className="dots">
      {Array.from({ length: total }, (_, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: fixed list of rounds
        <span key={i} className={`dot ${results[i] === undefined ? '' : results[i] ? 'ok' : 'bad'}`} />
      ))}
    </div>
  );
}

/** Digits keypad for "how many" answers. The value resets when the round changes. */
export function NumberPad({ round, onSubmit }: { round: number; onSubmit: (n: number) => void }) {
  const [value, setValue] = useState('');
  const [forRound, setForRound] = useState(round);
  if (forRound !== round) {
    setForRound(round);
    setValue('');
  }
  const press = (d: string) => setValue((v) => (v.length < 3 ? (v === '0' ? d : v + d) : v));
  useKeys((key) => {
    if (/^[0-9]$/.test(key)) press(key);
    else if (key === 'Backspace') setValue((v) => v.slice(0, -1));
    else if (key === 'Enter' && value !== '') onSubmit(Number(value));
    else return false;
    return true;
  });
  return (
    <div className="pad">
      <div className="pad-display">{value || '–'}</div>
      <div className="pad-keys">
        {['1', '2', '3', '4', '5', '6', '7', '8', '9'].map((d) => (
          <button key={d} type="button" className="key" onClick={() => press(d)}>
            {d}
          </button>
        ))}
        <button type="button" className="key" onClick={() => setValue((v) => v.slice(0, -1))}>
          ⌫
        </button>
        <button type="button" className="key" onClick={() => press('0')}>
          0
        </button>
        <button type="button" className="key ok" disabled={value === ''} onClick={() => onSubmit(Number(value))}>
          OK
        </button>
      </div>
      <KeyHint>Or type the number and press Enter.</KeyHint>
    </div>
  );
}
