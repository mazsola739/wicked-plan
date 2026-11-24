// Weight Balance Puzzle: a team shares 12 weighings to find five secret weights.
// Rules page: docs/games/weight-balance-puzzle.md

import { useState } from 'react';
import './WeightBalancePuzzle.css';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';

interface Load {
  left: number[];
  right: number[];
  leftExtra: number;
  rightExtra: number;
}

interface Weighing {
  seat: number;
  load: Load;
  result: number;
}

interface BalanceView {
  players: string[];
  you: number | null;
  phase: 'weigh' | 'answer' | 'over';
  turn: number;
  yourTurn: boolean;
  weighingsLeft: number;
  weighings: number;
  log: Weighing[];
  maxWeight: number;
  maxExtra: number;
  yourAnswer: number[] | null;
  answered: boolean[];
  score: number;
  weights: number[] | null;
  answers: (number[] | null)[] | null;
}

const LETTERS = ['A', 'B', 'C', 'D', 'E'];
const EMPTY: Load = { left: [], right: [], leftExtra: 0, rightExtra: 0 };

const panText = (blocks: number[], extra: number) =>
  [...blocks.map((b) => LETTERS[b]), ...(extra > 0 ? [`${extra} g`] : [])].join(' + ') || 'nothing';

const resultText = (r: number) => (r > 0 ? 'left heavier' : r < 0 ? 'right heavier' : 'balanced');

/**
 * The scale: a beam tilted by `tilt` (1 left down, -1 right down, 0 level, null unknown). The metal
 * parts are faceted like the game icons: lit from the top left with white and black overlays.
 */
function Scale({ load, tilt }: { load: Load; tilt: number | null }) {
  const angle = tilt === null ? 0 : tilt * -9;
  const pan = (blocks: number[], extra: number, x: number) => (
    <g transform={`translate(${x} 0)`}>
      <line className="wb-string" x1={-40} y1={0} x2={-50} y2={60} />
      <line className="wb-string" x1={40} y1={0} x2={50} y2={60} />
      <path className="wb-pan" d="M -62 60 L -36 68.6 L -12 72.5 L 12 72.5 L 36 68.6 L 62 60 Z" />
      <path fill="#fff" fillOpacity=".18" d="M -62 60 L -36 60 L -36 68.6 Z" />
      <path fill="#fff" fillOpacity=".1" d="M -36 60 L -12 60 L -12 72.5 L -36 68.6 Z" />
      <path fill="#000" fillOpacity=".15" d="M 12 60 L 36 60 L 36 68.6 L 12 72.5 Z" />
      <path fill="#000" fillOpacity=".3" d="M 36 60 L 62 60 L 36 68.6 Z" />
      {blocks.map((b, i) => (
        <g key={b} transform={`translate(${-48 + (i % 3) * 32} ${32 - Math.floor(i / 3) * 30})`}>
          <rect className="wb-block" width={28} height={28} rx={4} />
          <text className="wb-letter" x={14} y={15}>
            {LETTERS[b]}
          </text>
        </g>
      ))}
      {extra > 0 && (
        <g transform={`translate(${blocks.length < 3 ? -48 + blocks.length * 32 : 30} ${blocks.length < 3 ? 34 : 2})`}>
          <path className="wb-weight" d="M 4 26 L 8 6 L 20 6 L 24 26 Z M 11 6 L 12 2.5 L 16 2.5 L 17 6 Z" />
          <path fill="#fff" fillOpacity=".5" d="M 8 6 L 20 6 L 18 8 L 10 8 Z" />
          <path fill="#fff" fillOpacity=".35" d="M 4 26 L 8 6 L 10 8 L 7.2 24.5 Z M 11 6 L 12 2.5 L 14 2.5 L 14 6 Z" />
          <path fill="#fff" fillOpacity=".12" d="M 10 8 L 18 8 L 7.2 24.5 Z" />
          <path fill="#000" fillOpacity=".22" d="M 20 6 L 24 26 L 20.8 24.5 L 18 8 Z M 14 2.5 L 16 2.5 L 17 6 L 14 6 Z" />
          <path fill="#000" fillOpacity=".32" d="M 4 26 L 24 26 L 20.8 24.5 L 7.2 24.5 Z" />
          <path fill="#fff" d="M 11.4 9.4 L 12.1 10.1 L 11.4 10.8 L 10.7 10.1 Z" />
          <text className="wb-grams" x={14} y={18}>
            {extra}
          </text>
        </g>
      )}
    </g>
  );
  return (
    <svg className="drawing wb-scale" viewBox="-200 -40 400 200" role="img" aria-label="Balance scale">
      <path className="wb-stand" d="M -40 150 L 40 150 L 6 0 L -6 0 Z" />
      <path fill="#fff" fillOpacity=".1" d="M -40 150 L 0 150 L 0 0 L -6 0 Z" />
      <path fill="#000" fillOpacity=".25" d="M 0 150 L 40 150 L 6 0 L 0 0 Z" />
      <g transform={`rotate(${angle})`}>
        <path className="wb-beam" d="M -150 0 L -146 -5 L 146 -5 L 150 0 L 146 5 L -146 5 Z" />
        <path fill="#fff" fillOpacity=".4" d="M -148.8 -1.5 L -146 -5 L 146 -5 L 148.8 -1.5 Z" />
        <path fill="#000" fillOpacity=".28" d="M -148.8 1.5 L -146 5 L 146 5 L 148.8 1.5 Z" />
        <g transform="translate(-130 0)">
          <g transform={`rotate(${-angle})`}>{pan(load.left, load.leftExtra, 0)}</g>
        </g>
        <g transform="translate(130 0)">
          <g transform={`rotate(${-angle})`}>{pan(load.right, load.rightExtra, 0)}</g>
        </g>
      </g>
      <path className="wb-pivot" d="M 7.4 3.1 L 3.1 7.4 L -3.1 7.4 L -7.4 3.1 L -7.4 -3.1 L -3.1 -7.4 L 3.1 -7.4 L 7.4 -3.1 Z" />
      <path fill="#000" fillOpacity=".2" d="M 0 0 L 7.4 -3.1 L 7.4 3.1 Z M 0 0 L 3.1 7.4 L -3.1 7.4 Z" />
      <path fill="#000" fillOpacity=".35" d="M 0 0 L 7.4 3.1 L 3.1 7.4 Z" />
      <path fill="#000" fillOpacity=".12" d="M 0 0 L 3.1 -7.4 L 7.4 -3.1 Z M 0 0 L -3.1 7.4 L -7.4 3.1 Z" />
      {tilt !== null && (
        <text className="wb-result" y={-22}>
          {resultText(tilt)}
        </text>
      )}
    </svg>
  );
}

function Log({ view }: { view: BalanceView }) {
  if (view.log.length === 0) return <p className="muted small center">No weighings yet.</p>;
  return (
    <ol className="list wb-log">
      {view.log
        .map((w, i) => ({ w, n: i + 1 }))
        .reverse()
        .map(({ w, n }) => (
          <li key={n}>
            <span className="muted">
              {n}. {w.seat === view.you ? 'You' : nameOf(view.players[w.seat]!)}
            </span>
            <span>
              {panText(w.load.left, w.load.leftExtra)} <b>vs</b> {panText(w.load.right, w.load.rightExtra)}:{' '}
              <strong className="wb-res">{resultText(w.result)}</strong>
            </span>
          </li>
        ))}
    </ol>
  );
}

function Stepper({ label, value, max, onChange }: { label: string; value: number; max: number; onChange: (v: number) => void }) {
  return (
    <div className="wb-stepper">
      <span className="muted small">{label}</span>
      <button type="button" className="key" disabled={value <= 0} onClick={() => onChange(value - 1)}>
        −
      </button>
      <strong>{value} g</strong>
      <button type="button" className="key" disabled={value >= max} onClick={() => onChange(value + 1)}>
        +
      </button>
    </div>
  );
}

function WeighControls({ view, load, setLoad }: { view: BalanceView; load: Load; setLoad: (l: Load) => void }) {
  const side = (b: number) => (load.left.includes(b) ? 'left' : load.right.includes(b) ? 'right' : 'off');
  const cycle = (b: number) => {
    const s = side(b);
    const left = load.left.filter((x) => x !== b);
    const right = load.right.filter((x) => x !== b);
    if (s === 'off') left.push(b);
    else if (s === 'left') right.push(b);
    setLoad({ ...load, left, right });
  };
  const canWeigh = view.yourTurn && load.left.length + load.right.length > 0;
  const weigh = () => {
    if (!canWeigh) return;
    move({ type: 'weigh', ...load });
    setLoad(EMPTY);
  };
  useKeys((key) => {
    const b = LETTERS.indexOf(key.toUpperCase());
    if (b >= 0 && key.length === 1) cycle(b);
    else if (key === 'Enter') weigh();
    else return false;
    return true;
  });
  return (
    <>
      <p className="ask">
        {view.yourTurn ? 'Your weighing: tap blocks to put them on a pan.' : `${nameOf(view.players[view.turn]!)} is weighing…`}
      </p>
      <div className="wb-blocks">
        {LETTERS.map((l, b) => (
          <button key={l} type="button" className={`wb-block-btn ${side(b)}`} disabled={!view.yourTurn} onClick={() => cycle(b)}>
            <strong>{l}</strong>
            <span>{side(b)}</span>
          </button>
        ))}
      </div>
      <div className="wb-steppers">
        <Stepper label="Left" value={load.leftExtra} max={view.maxExtra} onChange={(v) => setLoad({ ...load, leftExtra: v })} />
        <Stepper label="Right" value={load.rightExtra} max={view.maxExtra} onChange={(v) => setLoad({ ...load, rightExtra: v })} />
      </div>
      <div className="row">
        <button type="button" className="btn ghost" disabled={!view.yourTurn} onClick={() => move({ type: 'answer' })}>
          Answer now
        </button>
        <button type="button" className="btn primary" disabled={!canWeigh} onClick={weigh}>
          Weigh
        </button>
      </div>
      <KeyHint>Keys: A–E move a block (off → left → right), Enter weighs.</KeyHint>
    </>
  );
}

function AnswerControls({ view }: { view: BalanceView }) {
  const [draft, setDraft] = useState<number[]>([0, 0, 0, 0, 0]);
  const [sel, setSel] = useState(0);
  const done = view.yourAnswer !== null;
  const set = (v: number) => {
    setDraft((d) => d.map((x, i) => (i === sel ? v : x)));
    setSel((s) => Math.min(4, s + 1));
  };
  const full = draft.every((v) => v > 0);
  const submit = () => full && !done && move({ type: 'submit', weights: draft });
  useKeys((key) => {
    const b = LETTERS.indexOf(key.toUpperCase());
    if (done) return false;
    if (/^[1-9]$/.test(key)) set(Number(key));
    else if (b >= 0 && key.length === 1) setSel(b);
    else if (key === 'Enter') submit();
    else return false;
    return true;
  });
  const shown = view.yourAnswer ?? draft;
  return (
    <>
      <p className="ask">{done ? 'Sent. Waiting for your teammate…' : 'Write down every block’s weight (1–9, all different).'}</p>
      <div className="wb-blocks">
        {LETTERS.map((l, b) => (
          <button
            key={l}
            type="button"
            className={`wb-block-btn${sel === b && !done ? ' sel' : ''}`}
            disabled={done}
            onClick={() => setSel(b)}
          >
            <strong>{l}</strong>
            <span className="wb-val">{shown[b] || '–'}</span>
          </button>
        ))}
      </div>
      {!done && (
        <>
          <div className="numbers wb-pad">
            {Array.from({ length: view.maxWeight }, (_, k) => k + 1).map((v) => (
              <button key={v} type="button" className={`key${draft.includes(v) ? ' used' : ''}`} onClick={() => set(v)}>
                {v}
              </button>
            ))}
          </div>
          <button type="button" className="btn primary" disabled={!full} onClick={submit}>
            Send answer
          </button>
          <KeyHint>Keys: A–E pick a block, 1–9 write its weight, Enter sends.</KeyHint>
        </>
      )}
    </>
  );
}

export default function WeightBalancePuzzle({ view: raw }: ScreenProps) {
  const view = raw as BalanceView;
  const [load, setLoad] = useState<Load>(EMPTY);
  const last = view.log[view.log.length - 1];
  const composing = view.yourTurn && load.left.length + load.right.length + load.leftExtra + load.rightExtra > 0;
  const shown = composing || !last ? load : last.load;
  const tilt = composing || !last ? null : last.result;
  const step = view.phase === 'weigh' ? `Weighing · ${view.weighingsLeft} of ${view.weighings} left` : 'Answer in secret';
  return (
    <Play
      top={<GameTop title="Weight Balance" step={step} score={view.score} />}
      stage={
        <>
          <Scale load={shown} tilt={tilt} />
          <Log view={view} />
        </>
      }
      controls={
        <>
          {view.phase === 'weigh' ? <WeighControls view={view} load={load} setLoad={setLoad} /> : <AnswerControls view={view} />}
          <p className="muted small center">
            Five blocks, different whole weights 1–9 g. 150 points per block everyone gets right; all five right: +50 per unused weighing.
          </p>
        </>
      }
    />
  );
}
