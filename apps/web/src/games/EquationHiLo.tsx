// Equation Hi-Lo. Rules page: docs/games/equation-hi-lo.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './EquationHiLo.css';

type Side = 'low' | 'high';
interface Build {
  order: number[];
  ops: string[];
  side: Side;
  value: number | null;
  text: string;
}
interface SeatView {
  id: string;
  chips: number;
  put: number;
  inHand: boolean;
  folded: boolean;
  up: [number, number] | null;
  built: boolean;
}
interface Showdown {
  hand: number;
  cards: (number[] | null)[];
  builds: (Build | null)[];
  gains: number[];
  pot: number;
}
interface HiLoView {
  me: number;
  hand: number;
  hands: number;
  phase: 'bet1' | 'build' | 'bet2' | 'over';
  seats: SeatView[];
  cards: number[] | null;
  turn: number | null;
  toCall: number;
  canRaise: boolean;
  pot: number;
  build: Build | null;
  timeLeftMs: number;
  phaseMs: number;
  log: string[];
  last: Showdown | null;
  over: boolean;
}

const SYMBOLS = ['+', '-', '*', '/'];
const SHOW: Record<string, string> = { '+': '+', '-': '−', '*': '×', '/': '÷' };
const said = (line: string) => line.replace(/\bbot-\d+\b|\byou\b/g, nameOf);

/** The same evaluation as the rules: × ÷ before + −; null when dividing by 0. */
function evaluate(nums: number[], ops: string[]): number | null {
  let sum = 0;
  let term = nums[0] ?? 0;
  for (let i = 0; i < ops.length; i++) {
    const n = nums[i + 1] ?? 0;
    const op = ops[i];
    if (op === '+' || op === '-') {
      sum += term;
      term = op === '+' ? n : -n;
    } else if (op === '*') term *= n;
    else if (n === 0) return null;
    else term /= n;
  }
  return sum + term;
}

const fmt = (v: number | null) => (v === null ? 'invalid' : Number.isInteger(v) ? String(v) : v.toFixed(2));

function Builder({ cards, hand, onSubmit }: { cards: number[]; hand: number; onSubmit: (b: Omit<Build, 'value' | 'text'>) => void }) {
  const [order, setOrder] = useState<number[]>([]);
  const [ops, setOps] = useState<string[]>([]);
  const [side, setSide] = useState<Side>('high');
  const [forHand, setForHand] = useState(hand);
  if (forHand !== hand) {
    setForHand(hand);
    setOrder([]);
    setOps([]);
  }
  const wantNumber = order.length === ops.length && order.length < 4;
  const wantOp = order.length === ops.length + 1 && ops.length < 3;
  const addCard = (i: number) => wantNumber && !order.includes(i) && setOrder([...order, i]);
  const addOp = (o: string) => wantOp && !ops.includes(o) && setOps([...ops, o]);
  const undo = () => (order.length > ops.length ? setOrder(order.slice(0, -1)) : setOps(ops.slice(0, -1)));
  const done = order.length === 4 && ops.length === 3;
  const value = done
    ? evaluate(
        order.map((i) => cards[i] ?? 0),
        ops,
      )
    : null;
  const submit = () => done && onSubmit({ order, ops, side });
  useKeys((key) => {
    if (/^[1-4]$/.test(key)) addCard(Number(key) - 1);
    else if (SYMBOLS.includes(key)) addOp(key);
    else if (key === 'x' || key === 'X') addOp('*');
    else if (key === 'l' || key === 'L') setSide('low');
    else if (key === 'h' || key === 'H') setSide('high');
    else if (key === 'Backspace') undo();
    else if (key === 'Enter') submit();
    else return false;
    return true;
  });
  const tokens: string[] = [];
  for (let i = 0; i < 7; i++) {
    const t = i % 2 === 0 ? order[i / 2] : ops[(i - 1) / 2];
    tokens.push(t === undefined ? '' : i % 2 === 0 ? String(cards[t as number]) : (SHOW[t as string] ?? ''));
  }
  return (
    <div className="ehl-builder">
      <div className="ehl-expr">
        {tokens.map((t, i) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: seven fixed slots
          <span key={i} className={`ehl-slot${i % 2 ? ' op' : ''}`}>
            {t}
          </span>
        ))}
      </div>
      <p className="center small muted">
        {done ? `= ${fmt(value)}` : wantNumber ? 'Tap a card' : 'Tap a symbol'} · aim for {side === 'low' ? '1' : '20'}
      </p>
      <div className="ehl-pick">
        {cards.map((c, i) => (
          <button
            // biome-ignore lint/suspicious/noArrayIndexKey: four fixed cards
            key={i}
            type="button"
            className="ehl-card"
            disabled={!wantNumber || order.includes(i)}
            onClick={() => addCard(i)}
          >
            {c}
          </button>
        ))}
        {SYMBOLS.map((o) => (
          <button key={o} type="button" className="op" disabled={!wantOp || ops.includes(o)} onClick={() => addOp(o)}>
            {SHOW[o]}
          </button>
        ))}
      </div>
      <div className="row">
        <button type="button" className={`btn${side === 'low' ? ' primary' : ''}`} onClick={() => setSide('low')}>
          Low · 1
        </button>
        <button type="button" className={`btn${side === 'high' ? ' primary' : ''}`} onClick={() => setSide('high')}>
          High · 20
        </button>
      </div>
      <div className="row">
        <button type="button" className="btn" disabled={order.length === 0} onClick={undo}>
          ⌫ Undo
        </button>
        <button type="button" className="btn primary" disabled={!done} onClick={submit}>
          Submit
        </button>
      </div>
    </div>
  );
}

export default function EquationHiLo({ view: raw }: ScreenProps) {
  const view = raw as HiLoView;
  const me = view.seats[view.me];
  const myTurn = view.turn === view.me && !view.over;
  const building = view.phase === 'build' && view.cards !== null && !me?.folded && view.build === null;
  useKeys((key) => {
    if (!myTurn) return false;
    const k = key.toUpperCase();
    if (k === 'C') move({ type: 'call' });
    else if (k === 'R' && view.canRaise) move({ type: 'raise' });
    else if (k === 'F') move({ type: 'fold' });
    else return false;
    return true;
  });
  const steps = { bet1: 'betting 1', build: 'build', bet2: 'betting 2', over: '' };
  return (
    <Play
      top={
        <>
          <GameTop
            title="Equation Hi-Lo"
            step={view.over ? 'Game over' : `Hand ${view.hand} of ${view.hands} · ${steps[view.phase]}`}
            score={me?.chips ?? 0}
          />
          {!view.over && <TimerBar leftMs={view.timeLeftMs} totalMs={view.phaseMs} />}
        </>
      }
      stage={
        <>
          <ul className="ehl-seats">
            {view.seats.map((s, i) => (
              <li
                key={s.id}
                className={`${view.turn === i ? 'turn' : ''}${s.folded || !s.inHand ? ' out' : ''}${i === view.me ? ' me' : ''}`}
              >
                <div>
                  <span>{nameOf(s.id)}</span>
                  <strong className="score">{s.chips}</strong>
                </div>
                <div className="ehl-up">
                  {s.up?.map((c, j) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: two face-up cards
                    <span key={j} className="ehl-mini">
                      {c}
                    </span>
                  ))}
                  {s.inHand && (
                    <>
                      <span className="ehl-mini down" />
                      <span className="ehl-mini down" />
                    </>
                  )}
                </div>
                <div className="muted small">
                  {s.folded ? 'folded' : s.inHand ? `in ${s.put}${view.phase === 'build' && s.built ? ' · built ✓' : ''}` : 'out'}
                </div>
              </li>
            ))}
          </ul>
          <div className="ehl-pot">
            Pot <strong>{view.pot}</strong>
          </div>
          {view.cards && (
            <div className="ehl-mine">
              {view.cards.map((c, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: four fixed cards
                <span key={i} className={`ehl-card${i >= 2 ? ' hidden' : ''}`}>
                  {c}
                  <em>{i >= 2 ? 'hidden' : 'face up'}</em>
                </span>
              ))}
            </div>
          )}
          {view.build && (
            <p className="center">
              Your equation: <strong>{view.build.text}</strong> = {fmt(view.build.value)} ({view.build.side})
            </p>
          )}
        </>
      }
      controls={
        <>
          {building && view.cards && <Builder cards={view.cards} hand={view.hand} onSubmit={(b) => move({ type: 'build', ...b })} />}
          {myTurn && (
            <div className="row">
              <button type="button" className="btn primary" onClick={() => move({ type: 'call' })}>
                {view.toCall > 0 ? `Call ${view.toCall}` : 'Check'}
              </button>
              <button type="button" className="btn" disabled={!view.canRaise} onClick={() => move({ type: 'raise' })}>
                Raise 2
              </button>
              <button type="button" className="btn" onClick={() => move({ type: 'fold' })}>
                Fold
              </button>
            </div>
          )}
          {!myTurn && !building && !view.over && (
            <p className="muted center">
              {view.phase === 'build' ? 'Waiting for the others to build…' : `${nameOf(view.seats[view.turn ?? 0]?.id ?? '')} is betting…`}
            </p>
          )}
          {view.last && (
            <div className="card">
              <div className="label">
                Hand {view.last.hand}: pot {view.last.pot}
              </div>
              <ul className="list small">
                {view.seats.map((s, i) => {
                  const b = view.last?.builds[i];
                  const g = view.last?.gains[i] ?? 0;
                  return (
                    <li key={s.id}>
                      <span>{nameOf(s.id)}</span>
                      <span>{b ? `${b.text} = ${fmt(b.value)} · ${b.side}` : '—'}</span>
                      <strong className="score">{g > 0 ? `+${g}` : g}</strong>
                    </li>
                  );
                })}
              </ul>
            </div>
          )}
          <ul className="ehl-log small">
            {view.log.map((l, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows
              <li key={i}>{said(l)}</li>
            ))}
          </ul>
          <KeyHint>Keys: C call, R raise, F fold · build: 1–4 cards, + − * / symbols, L / H side, Enter.</KeyHint>
        </>
      }
    />
  );
}
