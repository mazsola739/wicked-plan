// Dice Prediction: three dice, one hidden from you; predict your own rank. Rules: docs/games/dice-prediction.md

import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './DicePrediction.css';

type Phase = 'build' | 'predict' | 'result' | 'over';
type Kind = 'nothing' | 'pair' | 'straight' | 'triple';

interface DpView {
  phase: Phase;
  round: number;
  rounds: number;
  timeLeftMs: number;
  phaseMs: number;
  players: number;
  open: number;
  secret: number;
  rerolled: boolean;
  built: boolean;
  prediction: number | null;
  seats: { id: string; score: number; open: number; blind: number | null; rerolled: boolean; done: boolean }[];
  log: {
    round: number;
    seats: { id: string; dice: [number, number, number]; kind: Kind; rank: number; prediction: number; points: number }[];
  }[];
}

const KINDS: Record<Kind, string> = { nothing: 'nothing', pair: 'pair', straight: 'straight', triple: 'triple' };
const ORD = ['', '1st', '2nd', '3rd', '4th', '5th', '6th'];

/** Pip positions on a 3×3 grid (0–8) for each face. */
const PIPS: Record<number, number[]> = {
  1: [4],
  2: [0, 8],
  3: [0, 4, 8],
  4: [0, 2, 6, 8],
  5: [0, 2, 4, 6, 8],
  6: [0, 2, 3, 5, 6, 8],
};

function Die({ n, tone, small }: { n: number | null; tone?: 'secret' | 'blind'; small?: boolean }) {
  return (
    <span className={`dice-prediction-die${tone ? ` ${tone}` : ''}${small ? ' small' : ''}`}>
      {n === null ? (
        <b>?</b>
      ) : (
        Array.from({ length: 9 }, (_, i) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: fixed pip grid
          <i key={i} className={PIPS[n]?.includes(i) ? 'on' : ''} />
        ))
      )}
    </span>
  );
}

export default function DicePrediction({ view }: ScreenProps) {
  const v = view as DpView;
  const canBuild = v.phase === 'build' && !v.built;
  const canPredict = v.phase === 'predict' && v.prediction === null;
  useKeys((k) => {
    if (canBuild && (k === 'k' || k === 'K')) move({ type: 'keep' });
    else if (canBuild && (k === 'r' || k === 'R')) move({ type: 'reroll' });
    else if (canPredict && /^[1-6]$/.test(k) && Number(k) <= v.players) move({ type: 'predict', rank: Number(k) });
    else return false;
    return true;
  });
  const me = v.seats.find((s) => s.id === 'you');
  const last = v.log[v.log.length - 1];
  const showResult = (v.phase === 'result' || v.phase === 'over') && last;
  const mine = showResult ? last.seats.find((s) => s.id === 'you') : undefined;
  let ask = '';
  if (v.phase === 'build') ask = v.built ? 'Waiting for the others…' : 'Keep your secret die, or reroll it once?';
  else if (v.phase === 'predict') ask = v.prediction ? `You predict ${ORD[v.prediction]}. Waiting…` : 'Where will your hand rank?';
  else if (mine) ask = `Your hand: ${KINDS[mine.kind]}, ranked ${ORD[mine.rank]} (you said ${ORD[mine.prediction]}): +${mine.points}.`;
  return (
    <Play
      top={
        <>
          <GameTop title="Dice Prediction" step={`Round ${v.round} of ${v.rounds}`} score={me?.score ?? 0} />
          <TimerBar leftMs={v.timeLeftMs} totalMs={v.phaseMs} />
        </>
      }
      stage={
        <div className="card">
          <ul className="list">
            {showResult
              ? [...last.seats]
                  .sort((a, b) => a.rank - b.rank)
                  .map((s) => (
                    <li key={s.id}>
                      <span>
                        {ORD[s.rank]} {nameOf(s.id)}{' '}
                        <span className="muted small">
                          {KINDS[s.kind]} · said {ORD[s.prediction]} · +{s.points}
                        </span>
                      </span>
                      <span className="dice-prediction-dice">
                        <Die n={s.dice[0]} small />
                        <Die n={s.dice[1]} tone="secret" small />
                        <Die n={s.dice[2]} tone="blind" small />
                      </span>
                    </li>
                  ))
              : v.seats.map((s) => (
                  <li key={s.id}>
                    <span>
                      {nameOf(s.id)} <span className="score">{s.score}</span>{' '}
                      <span className="muted small">
                        {s.rerolled ? 'rerolled · ' : ''}
                        {s.done ? 'ready' : 'thinking…'}
                      </span>
                    </span>
                    <span className="dice-prediction-dice">
                      <Die n={s.open} small />
                      <Die n={s.id === 'you' ? v.secret : null} tone="secret" small />
                      <Die n={s.blind} tone="blind" small />
                    </span>
                  </li>
                ))}
          </ul>
          <p className="muted small center">Triple › straight › pair › nothing; then the higher sum.</p>
        </div>
      }
      controls={
        <>
          <div className="dice-prediction-mine">
            <figure>
              <Die n={v.open} />
              <figcaption>open</figcaption>
            </figure>
            <figure>
              <Die n={v.secret} tone="secret" />
              <figcaption>secret</figcaption>
            </figure>
            <figure>
              <Die n={mine ? mine.dice[2] : null} tone="blind" />
              <figcaption>blind</figcaption>
            </figure>
          </div>
          <p className="ask">{ask}</p>
          {canBuild && (
            <div className="row">
              <button type="button" className="btn" onClick={() => move({ type: 'keep' })}>
                Keep
              </button>
              <button type="button" className="btn primary" onClick={() => move({ type: 'reroll' })}>
                Reroll secret
              </button>
            </div>
          )}
          {canPredict && (
            <div className="dice-prediction-ranks">
              {Array.from({ length: v.players }, (_, i) => i + 1).map((r) => (
                <button key={r} type="button" className="key" onClick={() => move({ type: 'predict', rank: r })}>
                  {ORD[r]}
                </button>
              ))}
            </div>
          )}
          <KeyHint>K keep · R reroll · 1–{v.players} predict your rank.</KeyHint>
        </>
      }
    />
  );
}
