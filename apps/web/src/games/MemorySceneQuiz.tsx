// Memory Scene Quiz: study a scene, then the team answers 10 questions in turn.
// Rules page: docs/games/memory-scene-quiz.md

import './MemorySceneQuiz.css';
import { GameTop, KeyHint, move, nameOf, Play, RoundDots, type ScreenProps, TimerBar, useKeys } from './common';

interface SceneObject {
  shape: string;
  colour: string;
  count: number;
  row: number;
  col: number;
}

interface Question {
  kind: string;
  text: string;
  options: string[];
}

interface Answered {
  text: string;
  options: string[];
  seat: number;
  given: number | null;
  answer: number;
  correct: boolean;
}

interface SceneView {
  players: string[];
  you: number | null;
  phase: 'study' | 'questions' | 'over';
  cols: number;
  rows: number;
  objects: SceneObject[] | null;
  ready: boolean[];
  question: Question | null;
  number: number;
  questions: number;
  answerer: number | null;
  yourTurn: boolean;
  history: Answered[];
  timeLeftMs: number;
  phaseMs: number;
  score: number;
  failed: boolean;
}

const CELL = 100;
const SPOTS: [number, number][][] = [
  [[50, 50]],
  [
    [28, 50],
    [72, 50],
  ],
  [
    [50, 28],
    [28, 70],
    [72, 70],
  ],
];

function starPoints(r: number): string {
  return Array.from({ length: 10 }, (_, i) => {
    const a = (Math.PI / 5) * i - Math.PI / 2;
    const d = i % 2 ? r * 0.45 : r;
    return `${(d * Math.cos(a)).toFixed(1)},${(d * Math.sin(a)).toFixed(1)}`;
  }).join(' ');
}

/** One shape centred on (0, 0). */
function Shape({ shape, r }: { shape: string; r: number }) {
  switch (shape) {
    case 'circle':
      return <circle r={r * 0.9} />;
    case 'square':
      return <rect x={-r * 0.8} y={-r * 0.8} width={r * 1.6} height={r * 1.6} rx={2} />;
    case 'triangle':
      return <polygon points={`0,${-r} ${r * 0.95},${r * 0.8} ${-r * 0.95},${r * 0.8}`} />;
    case 'star':
      return <polygon points={starPoints(r * 1.1)} />;
    case 'heart':
      return (
        <path
          d={`M 0 ${r * 0.9} C ${-r * 1.5} ${-r * 0.1} ${-r * 0.6} ${-r * 1.2} 0 ${-r * 0.4} C ${r * 0.6} ${-r * 1.2} ${r * 1.5} ${-r * 0.1} 0 ${r * 0.9} Z`}
        />
      );
    default:
      return (
        <path
          d={`M ${r * 0.6} ${-r * 0.8} A ${r} ${r} 0 1 0 ${r * 0.6} ${r * 0.8} A ${r * 0.85} ${r * 0.85} 0 0 1 ${r * 0.6} ${-r * 0.8} Z`}
        />
      );
  }
}

function Scene({ view }: { view: SceneView }) {
  const w = view.cols * CELL;
  const h = view.rows * CELL;
  const cells = Array.from({ length: view.cols * view.rows }, (_, i) => ({
    id: `c${i}`,
    x: (i % view.cols) * CELL,
    y: Math.floor(i / view.cols) * CELL,
  }));
  return (
    <svg className="drawing msq-scene" viewBox={`-4 -4 ${w + 8} ${h + 8}`} role="img" aria-label="Scene to remember">
      {cells.map((c) => (
        <rect key={c.id} className="msq-cell" x={c.x + 3} y={c.y + 3} width={CELL - 6} height={CELL - 6} rx={10} />
      ))}
      {view.objects
        ? view.objects.map((o) => (
            <g key={`${o.row}-${o.col}`} className={`msq-shape ${o.colour}`} transform={`translate(${o.col * CELL} ${o.row * CELL})`}>
              {(SPOTS[o.count - 1] ?? []).map(([x, y]) => (
                <g key={`${x}-${y}`} transform={`translate(${x} ${y})`}>
                  <Shape shape={o.shape} r={o.count === 1 ? 26 : 17} />
                </g>
              ))}
            </g>
          ))
        : cells.map((c) => (
            <text key={c.id} className="msq-hidden" x={c.x + CELL / 2} y={c.y + CELL / 2}>
              ?
            </text>
          ))}
    </svg>
  );
}

export default function MemorySceneQuiz({ view: raw }: ScreenProps) {
  const view = raw as SceneView;
  const you = view.you ?? 0;
  const studying = view.phase === 'study';
  const iAmReady = view.ready[you] ?? false;
  const answer = (choice: number) =>
    view.yourTurn && view.question && choice < view.question.options.length && move({ type: 'answer', choice });
  useKeys((key) => {
    if (studying && key === 'Enter') move({ type: 'ready' });
    else if (/^[1-4]$/.test(key)) answer(Number(key) - 1);
    else return false;
    return true;
  });
  const last = view.history[view.history.length - 1];
  const who = (seat: number) => (seat === you ? 'You' : nameOf(view.players[seat]!));
  return (
    <Play
      top={
        <>
          <GameTop
            title="Memory Scene"
            step={studying ? 'Memorise the scene!' : `Question ${view.number} of ${view.questions}`}
            score={view.score}
          />
          <RoundDots results={view.history.map((h) => h.correct)} total={view.questions} />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.phaseMs} />
        </>
      }
      stage={<Scene view={view} />}
      controls={
        studying ? (
          <>
            <p className="ask">Shapes, colours, how many, and where. Then 10 questions, one wrong answer fails the team.</p>
            <button type="button" className="btn primary" disabled={iAmReady} onClick={() => move({ type: 'ready' })}>
              {iAmReady ? 'Waiting for the team…' : 'Ready'}
            </button>
            <p className="muted small center">
              {view.ready.filter(Boolean).length} of {view.players.length} ready
            </p>
            <KeyHint>Keys: Enter = Ready.</KeyHint>
          </>
        ) : (
          <>
            {last && (
              <p className={`last ${last.correct ? 'ok' : 'bad'}`}>
                {who(last.seat)}: {last.given === null ? 'no answer' : last.options[last.given]}{' '}
                {last.correct ? '✓' : `✗ (it was ${last.options[last.answer]})`}
              </p>
            )}
            {view.question && (
              <>
                <p className="muted small center">{view.yourTurn ? 'Your question' : `${who(view.answerer ?? 0)} answers this one`}</p>
                <p className="ask msq-question">{view.question.text}</p>
                <div className="msq-options">
                  {view.question.options.map((o, i) => (
                    <button key={o} type="button" className="btn msq-option" disabled={!view.yourTurn} onClick={() => answer(i)}>
                      <span className="msq-key">{i + 1}</span> {o}
                    </button>
                  ))}
                </div>
                <KeyHint>Keys: 1–4 answer.</KeyHint>
              </>
            )}
          </>
        )
      }
    />
  );
}
