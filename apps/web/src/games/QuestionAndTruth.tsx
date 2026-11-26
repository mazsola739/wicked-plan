// Question and Truth: buy answers with tokens, then guess the opponent's row. Rules page: docs/games/question-and-truth.md

import { useState } from 'react';
import { t } from '../i18n';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './QuestionAndTruth.css';

interface LogLine {
  question: string;
  answer: string;
  cost: number;
}

interface TruthView {
  row: number[];
  tokens: number;
  theirTokens: number;
  opponent: string;
  yourTurn: boolean;
  guessed: boolean;
  theyGuessed: boolean;
  log: LogLine[];
  theirLog: LogLine[];
  over: boolean;
}

type Kind = 'cardAt' | 'whereIs' | 'sum' | 'leftOf' | 'bigger' | 'leftHalf';

/** The question menu: what the one or two numbers mean, and the cost. */
const MENU: { kind: Kind; label: string; cost: number; args: ('position' | 'card')[]; text: (a: number, b: number) => string }[] = [
  { kind: 'cardAt', label: 'Card at…', cost: 3, args: ['position'], text: (a) => t('Which card is at position {a}?', { a }) },
  { kind: 'whereIs', label: 'Where is…', cost: 3, args: ['card'], text: (a) => t('Where is card {a}?', { a }) },
  {
    kind: 'sum',
    label: 'Sum of two',
    cost: 2,
    args: ['position', 'position'],
    text: (a, b) => t('Sum of positions {a} and {b}?', { a, b }),
  },
  { kind: 'leftOf', label: 'Left of?', cost: 1, args: ['card', 'card'], text: (a, b) => t('Is card {a} left of card {b}?', { a, b }) },
  {
    kind: 'bigger',
    label: 'Bigger?',
    cost: 1,
    args: ['position', 'position'],
    text: (a, b) => t('Is position {a} bigger than {b}?', { a, b }),
  },
  { kind: 'leftHalf', label: 'Left half?', cost: 1, args: ['card'], text: (a) => t('Is card {a} in the left half?', { a }) },
];

const NUMS = [1, 2, 3, 4, 5, 6, 7, 8];

function question(kind: Kind, a: number, b: number) {
  switch (kind) {
    case 'cardAt':
      return { kind, pos: a };
    case 'whereIs':
    case 'leftHalf':
      return { kind, card: a };
    default:
      return { kind, a, b };
  }
}

export default function QuestionAndTruth({ view: raw }: ScreenProps) {
  const view = raw as TruthView;
  const them = nameOf(view.opponent);
  const [kind, setKind] = useState<Kind>('leftOf');
  const [a, setA] = useState<number | null>(null);
  const [b, setB] = useState<number | null>(null);
  const [guess, setGuess] = useState<(number | null)[]>(Array(8).fill(null));
  const [slot, setSlot] = useState(0);
  // The final guess takes two taps.
  const [sure, setSure] = useState(false);
  if (sure && !(view.yourTurn && guess.every((g) => g !== null))) setSure(false);
  const item = (MENU.find((m) => m.kind === kind) ?? MENU[0]) as (typeof MENU)[number];
  const two = item.args.length === 2;
  const ready = view.yourTurn && a !== null && (!two || (b !== null && b !== a)) && item.cost <= view.tokens;
  const full = guess.every((g) => g !== null);

  const put = (card: number) => {
    setGuess((g) => g.map((c, i) => (i === slot ? card : c === card ? null : c)));
    setSlot((s) => Math.min(7, s + 1));
  };
  const ask = () => {
    if (!ready || a === null) return;
    move({ type: 'ask', question: question(kind, a, b ?? 0) });
    setA(null);
    setB(null);
  };
  const pickKind = (k: Kind) => {
    setKind(k);
    setA(null);
    setB(null);
  };

  useKeys((key) => {
    const n = Number(key);
    if (n >= 1 && n <= 8) put(n);
    else if (key === 'Backspace') setGuess((g) => g.map((c, i) => (i === slot ? null : c)));
    else if (key === 'ArrowLeft') setSlot((s) => Math.max(0, s - 1));
    else if (key === 'ArrowRight') setSlot((s) => Math.min(7, s + 1));
    else return false;
    return true;
  });

  const status = view.guessed
    ? `${t('You have guessed.')} ${view.theyGuessed ? '' : t('{name} goes on…', { name: them })}`
    : view.yourTurn
      ? t('Your turn: ask a question or make your final guess.')
      : t('{name} is asking…', { name: them });

  return (
    <Play
      top={
        <GameTop
          title="Question and Truth"
          step={`${t('Tokens: {name} {n}', { name: them, n: view.theirTokens })}${view.theyGuessed ? ` · ${t('guessed')}` : ''}`}
          score={view.tokens}
        />
      }
      stage={
        <div className="question-and-truth-stage">
          <div className="label">{t('Your secret row')}</div>
          <div className="question-and-truth-row mine">
            {view.row.map((c, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: positions are fixed
              <span key={i}>{c}</span>
            ))}
          </div>
          <div className="label">{t("Your guess of {name}'s row (tap a slot, then a card)", { name: them })}</div>
          <div className="question-and-truth-row">
            {guess.map((c, i) => (
              <button
                // biome-ignore lint/suspicious/noArrayIndexKey: positions are fixed
                key={i}
                type="button"
                className={`question-and-truth-slot${i === slot ? ' on' : ''}`}
                onClick={() => setSlot(i)}
              >
                <small>{i + 1}</small>
                {c ?? ''}
              </button>
            ))}
          </div>
          <div className="question-and-truth-row">
            {NUMS.map((n) => (
              <button key={n} type="button" className={`key${guess.includes(n) ? ' used' : ''}`} onClick={() => put(n)}>
                {n}
              </button>
            ))}
          </div>
          <Log title={t('Your questions')} lines={view.log} empty={t('No questions yet.')} />
          <Log title={t("{name}'s questions about your row", { name: them })} lines={view.theirLog} empty={t('None yet.')} />
        </div>
      }
      controls={
        <>
          <p className="ask">{status}</p>
          {!view.guessed && (
            <>
              <div className="question-and-truth-menu">
                {MENU.map((m) => (
                  <button
                    key={m.kind}
                    type="button"
                    className={`btn small${m.kind === kind ? ' primary' : ''}`}
                    disabled={m.cost > view.tokens}
                    onClick={() => pickKind(m.kind)}
                  >
                    {t(m.label)} <span className="question-and-truth-cost">{m.cost}</span>
                  </button>
                ))}
              </div>
              {item.args.map((arg, k) => {
                const value = k === 0 ? a : b;
                const set = k === 0 ? setA : setB;
                return (
                  <div key={`${kind}-${arg}-${k === 0 ? 'a' : 'b'}`}>
                    <div className="label">{t(arg)}</div>
                    <div className="question-and-truth-row">
                      {NUMS.map((n) => (
                        <button key={n} type="button" className={`key${value === n ? ' ok' : ''}`} onClick={() => set(n)}>
                          {n}
                        </button>
                      ))}
                    </div>
                  </div>
                );
              })}
              <button type="button" className="btn primary" disabled={!ready} onClick={ask}>
                {a !== null && (!two || b !== null) ? item.text(a, b ?? 0) : t('Ask')} ·{' '}
                {item.cost > 1 ? t('{n} tokens', { n: item.cost }) : t('{n} token', { n: item.cost })}
              </button>
              <button
                type="button"
                className={`btn${sure ? ' primary' : ''}`}
                disabled={!view.yourTurn || !full}
                onClick={() => (sure ? move({ type: 'guess', order: guess }) : setSure(true))}
              >
                {sure ? t('Tap again: no more questions after this') : t('Final guess')}
              </button>
            </>
          )}
          <KeyHint>{t('Keys: 1–8 fill the guess slot, ← → move, Backspace clears.')}</KeyHint>
        </>
      }
    />
  );
}

function Log({ title, lines, empty }: { title: string; lines: LogLine[]; empty: string }) {
  return (
    <div>
      <div className="label">{title}</div>
      {lines.length === 0 ? (
        <p className="muted small">{empty}</p>
      ) : (
        <ul className="list question-and-truth-log">
          {lines.map((l, i) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows
            <li key={i}>
              <span>{t(l.question)}</span>
              <strong>{t(l.answer)}</strong>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
