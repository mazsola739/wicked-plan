// Fragments of Memory: study a scene, then the team answers 10 questions in turn.
// Rules page: docs/games/memory-scene-quiz.md

import type { ReactNode } from 'react';
import './MemorySceneQuiz.css';
import { t } from '../i18n';
import { GameTop, KeyHint, move, nameOf, Play, RoundDots, type ScreenProps, useKeys } from './common';

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
  given: number;
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

/**
 * The shapes as faceted gems drawn at r = 20 around (0, 0), lit from the top left: white and black
 * overlays make the facets, with a white glint. The first path is the silhouette in the object's colour.
 */
const SHAPES: Record<string, ReactNode> = {
  circle: (
    <>
      <path d="M0,-18 9,-15.6 15.6,-9 18,0 15.6,9 9,15.6 0,18 -9,15.6 -15.6,9 -18,0 -15.6,-9 -9,-15.6Z" />
      <path fill="#fff" fillOpacity=".1" d="M-15.6,9 -18,0 -9.5,0 -8.2,4.8Z" />
      <path fill="#fff" fillOpacity=".2" d="M0,-18 9,-15.6 4.7,-8.2 0,-9.5Z" />
      <path fill="#fff" fillOpacity=".3" d="M-18,0 -15.6,-9 -8.2,-4.8 -9.5,0Z" />
      <path fill="#fff" fillOpacity=".4" d="M-15.6,-9 -9,-15.6 -4.8,-8.2 -8.2,-4.8Z M-9,-15.6 0,-18 0,-9.5 -4.8,-8.2Z" />
      <path fill="#000" fillOpacity=".12" d="M-9,15.6 -15.6,9 -8.2,4.8 -4.7,8.2Z" />
      <path fill="#000" fillOpacity=".2" d="M15.6,-9 18,0 9.5,0 8.2,-4.7Z" />
      <path fill="#000" fillOpacity=".28" d="M18,0 15.6,9 8.2,4.7 9.5,0Z M0,18 -9,15.6 -4.7,8.2 0,9.5Z" />
      <path fill="#000" fillOpacity=".35" d="M15.6,9 9,15.6 4.7,8.2 8.2,4.7Z M9,15.6 0,18 0,9.5 4.7,8.2Z" />
      <path fill="#fff" d="M-6.5,-7.8 -5.2,-6.5 -6.5,-5.2 -7.8,-6.5Z" />
    </>
  ),
  square: (
    <>
      <path d="M-16,-16 16,-16 16,16 -16,16Z" />
      <path fill="#fff" fillOpacity=".1" d="M-8,-8 8,-8 0,0Z M-8,8 -8,-8 0,0Z" />
      <path fill="#fff" fillOpacity=".2" d="M-16,16 -16,-16 -8,-8 -8,8Z" />
      <path fill="#fff" fillOpacity=".3" d="M-16,-16 16,-16 8,-8 -8,-8Z" />
      <path fill="#000" fillOpacity=".28" d="M16,-16 16,16 8,8 8,-8Z" />
      <path fill="#000" fillOpacity=".35" d="M16,16 -16,16 -8,8 8,8Z" />
      <path fill="#fff" d="M-4.6,-5.9 -3.3,-4.6 -4.6,-3.3 -5.9,-4.6Z" />
    </>
  ),
  triangle: (
    <>
      <path d="M0,-20 19,16 -19,16Z" />
      <path fill="#fff" fillOpacity=".1" d="M-8,9 0,-6.1 0,4Z" />
      <path fill="#fff" fillOpacity=".4" d="M-19,16 0,-20 0,-6.1 -8,9Z" />
      <path fill="#000" fillOpacity=".12" d="M0,-20 19,16 8,9 0,-6.1Z M8,9 -8,9 0,4Z" />
      <path fill="#000" fillOpacity=".35" d="M19,16 -19,16 -8,9 8,9Z" />
      <path fill="#fff" d="M-3.2,-1.1 -1.9,0.2 -3.2,1.5 -4.5,0.2Z" />
    </>
  ),
  star: (
    <>
      <path d="M0,-22 5.8,-8 20.9,-6.8 9.4,3.1 12.9,17.8 0,9.9 -12.9,17.8 -9.4,3.1 -20.9,-6.8 -5.8,-8Z" />
      <path fill="#fff" fillOpacity=".3" d="M5.8,-8 20.9,-6.8 0,0Z M-12.9,17.8 -9.4,3.1 0,0Z" />
      <path fill="#fff" fillOpacity=".4" d="M-20.9,-6.8 -5.8,-8 0,0Z M-5.8,-8 0,-22 0,0Z" />
      <path fill="#000" fillOpacity=".2" d="M0,-22 5.8,-8 0,0Z M-9.4,3.1 -20.9,-6.8 0,0Z" />
      <path fill="#000" fillOpacity=".28" d="M9.4,3.1 12.9,17.8 0,0Z M12.9,17.8 0,9.9 0,0Z" />
      <path fill="#000" fillOpacity=".35" d="M20.9,-6.8 9.4,3.1 0,0Z M0,9.9 -12.9,17.8 0,0Z" />
      <path fill="#fff" d="M-3.6,-8.7 -2.3,-7.4 -3.6,-6.1 -4.9,-7.4Z" />
    </>
  ),
  heart: (
    <>
      <path d="M0,-10 3.5,-14 8,-16 13.5,-14.5 17,-9 16,-2 9,7 0,16 -9,7 -16,-2 -17,-9 -13.5,-14.5 -8,-16 -3.5,-14Z" />
      <path fill="#fff" fillOpacity=".1" d="M-17,-9 -8,-7.5 -16,-2Z" />
      <path fill="#fff" fillOpacity=".3" d="M-13.5,-14.5 -8,-16 -3.5,-14 -8,-7.5Z" />
      <path fill="#fff" fillOpacity=".4" d="M-17,-9 -13.5,-14.5 -8,-7.5Z M13.5,-14.5 8,-16 3.5,-14 8,-7.5Z M3.5,-14 0,-10 0,1 8,-7.5Z" />
      <path fill="#000" fillOpacity=".12" d="M-16,-2 -8,-7.5 0,1 -9,7Z M-9,7 0,1 0,16Z M17,-9 13.5,-14.5 8,-7.5Z" />
      <path fill="#000" fillOpacity=".28" d="M17,-9 8,-7.5 16,-2Z M9,7 0,1 0,16Z" />
      <path fill="#000" fillOpacity=".35" d="M16,-2 8,-7.5 0,1 9,7Z" />
      <path fill="#fff" d="M-10.4,-12.5 -9.1,-11.2 -10.4,-9.9 -11.7,-11.2Z" />
    </>
  ),
  moon: (
    <>
      <path d="M12,-16 0.2,-20 -11.7,-16.3 -19,-6.2 -19,6.2 -11.7,16.3 0.2,20 12,16 6.9,13.1 3,8.5 1,3 1,-3 3,-8.5 6.9,-13.1Z" />
      <path fill="#fff" fillOpacity=".1" d="M12,-16 0.2,-20 3.2,-16.9Z M3.2,16.9 12,16 6.9,13.1Z" />
      <path fill="#fff" fillOpacity=".2" d="M-19,-6.2 -19,6.2 -10,4.8 -10,-4.8Z" />
      <path fill="#fff" fillOpacity=".4" d="M-11.7,-16.3 -19,-6.2 -10,-4.8 -5,-12.8Z" />
      <path fill="#fff" fillOpacity=".5" d="M0.2,-20 -11.7,-16.3 -5,-12.8 3.2,-16.9Z" />
      <path fill="#000" fillOpacity=".12" d="M-19,6.2 -11.7,16.3 -5,12.8 -10,4.8Z M-10,4.8 -5,12.8 3,8.5 1,3Z" />
      <path fill="#000" fillOpacity=".28" d="M-10,-4.8 -10,4.8 1,3 1,-3Z" />
      <path
        fill="#000"
        fillOpacity=".35"
        d="M12,-16 3.2,-16.9 6.9,-13.1Z M3.2,-16.9 -5,-12.8 3,-8.5 6.9,-13.1Z M-5,-12.8 -10,-4.8 1,-3 3,-8.5Z M-11.7,16.3 0.2,20 3.2,16.9 -5,12.8Z M0.2,20 12,16 3.2,16.9Z"
      />
      <path fill="#fff" d="M-12.6,-9.7 -11.3,-8.4 -12.6,-7.1 -13.9,-8.4Z" />
    </>
  ),
};

/** One shape centred on (0, 0); anything unknown is the moon. */
function Shape({ shape, r }: { shape: string; r: number }) {
  return <g transform={`scale(${r / 20})`}>{SHAPES[shape] ?? SHAPES.moon}</g>;
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
    <svg className="drawing msq-scene" viewBox={`-4 -4 ${w + 8} ${h + 8}`} role="img" aria-label={t('Scene to remember')}>
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
  const who = (seat: number) => (seat === you ? t('You') : nameOf(view.players[seat]!));
  return (
    <Play
      top={
        <>
          <GameTop
            title="Memory Scene"
            step={studying ? t('Memorise the scene!') : t('Question {n} of {total}', { n: view.number, total: view.questions })}
            score={view.score}
          />
          <RoundDots results={view.history.map((h) => h.correct)} total={view.questions} />
        </>
      }
      stage={<Scene view={view} />}
      controls={
        studying ? (
          <>
            <p className="ask">{t('Shapes, colours, how many, and where. Then 10 questions, one wrong answer fails the team.')}</p>
            <button type="button" className="btn primary" disabled={iAmReady} onClick={() => move({ type: 'ready' })}>
              {iAmReady ? t('Waiting for the team…') : t('Ready')}
            </button>
            <p className="muted small center">
              {t('{n} of {total} ready', { n: view.ready.filter(Boolean).length, total: view.players.length })}
            </p>
            <KeyHint>{t('Keys: Enter = Ready.')}</KeyHint>
          </>
        ) : (
          <>
            {last && (
              <p className={`last ${last.correct ? 'ok' : 'bad'}`}>
                {who(last.seat)}: {t(last.options[last.given] ?? '')}{' '}
                {last.correct ? '✓' : `✗ (${t('it was {answer}', { answer: t(last.options[last.answer] ?? '') })})`}
              </p>
            )}
            {view.question && (
              <>
                <p className="muted small center">
                  {view.yourTurn ? t('Your question') : t('{name} answers this one', { name: who(view.answerer ?? 0) })}
                </p>
                <p className="ask msq-question">{t(view.question.text)}</p>
                <div className="msq-options">
                  {view.question.options.map((o, i) => (
                    <button key={o} type="button" className="btn msq-option" disabled={!view.yourTurn} onClick={() => answer(i)}>
                      <span className="msq-key">{i + 1}</span> {t(o)}
                    </button>
                  ))}
                </div>
                <KeyHint>{t('Keys: 1–4 answer.')}</KeyHint>
              </>
            )}
          </>
        )
      }
    />
  );
}
