// Triple Dice: three dice, one hidden from you; predict your own rank. Rules: docs/games/dice-prediction.md

import { t } from '../i18n';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './DicePrediction.css';

type Phase = 'build' | 'predict' | 'result' | 'over';
type Kind = 'nothing' | 'pair' | 'straight' | 'triple';

interface DpView {
  phase: Phase;
  round: number;
  rounds: number;
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

/** Pip i as a small octagon, and its shaded lower-right half (a pip is a cut gem too). */
const pipAt = (i: number): [number, number] => [10.5 + (i % 3) * 9.5, 10.5 + Math.floor(i / 3) * 9.5];
const pip = (i: number) => {
  const [x, y] = pipAt(i);
  return `M${x - 1.4},${y - 3.4}h2.8l2,2v2.8l-2,2h-2.8l-2,-2v-2.8Z`;
};
const pipShade = (i: number) => {
  const [x, y] = pipAt(i);
  return `M${x + 2.4},${y - 2.4}l1,1v2.8l-2,2h-2.8l-1,-1Z`;
};

/** A faceted die face: a bevelled cube face lit from the top left, white gem pips (or '?'). */
function Die({ n, tone, small }: { n: number | null; tone?: 'secret' | 'blind'; small?: boolean }) {
  const pips = n === null ? [] : (PIPS[n] ?? []);
  return (
    <span className={`dice-prediction-die${tone ? ` ${tone}` : ''}${small ? ' small' : ''}`}>
      <svg viewBox="0 0 40 40" role="img" aria-label={n === null ? t('unknown die') : t('die {n}', { n })}>
        <path className="edge" d="M6,1 34,1 39,6 39,34 34,39 6,39 1,34 1,6Z" />
        <path fill="#fff" fillOpacity=".5" d="M1,6 6,1 8,4 4,8Z" />
        <path fill="#fff" fillOpacity=".3" d="M6,1 34,1 32,4 8,4Z" />
        <path fill="#fff" fillOpacity=".18" d="M1,6 4,8 4,32 1,34Z" />
        <path fill="#fff" fillOpacity=".1" d="M34,1 39,6 36,8 32,4Z M4,8 8,4 32,4 36,8 4,32Z" />
        <path fill="#000" fillOpacity=".12" d="M1,34 4,32 8,36 6,39Z" />
        <path fill="#000" fillOpacity=".2" d="M39,6 39,34 36,32 36,8Z" />
        <path fill="#000" fillOpacity=".3" d="M34,39 6,39 8,36 32,36Z" />
        <path fill="#000" fillOpacity=".35" d="M39,34 34,39 32,36 36,32Z" />
        <path fill="#fff" d="M5,3.6 6.2,4.8 5,6 3.8,4.8Z" />
        {pips.length > 0 && <path className="pip" d={pips.map(pip).join('')} />}
        {pips.length > 0 && <path fill="#000" fillOpacity=".22" d={pips.map(pipShade).join('')} />}
      </svg>
      {n === null && <b aria-hidden="true">?</b>}
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
  if (v.phase === 'build') ask = v.built ? t('Waiting for the others…') : t('Keep your secret die, or reroll it once?');
  else if (v.phase === 'predict')
    ask = v.prediction ? t('You predict {rank}. Waiting…', { rank: t(ORD[v.prediction] ?? '') }) : t('Where will your hand rank?');
  else if (mine)
    ask = t('Your hand: {kind}, ranked {rank} (you said {said}): +{points}.', {
      kind: t(KINDS[mine.kind]),
      rank: t(ORD[mine.rank] ?? ''),
      said: t(ORD[mine.prediction] ?? ''),
      points: mine.points,
    });
  return (
    <Play
      top={<GameTop title="Triple Dice" step={t('Round {n} of {total}', { n: v.round, total: v.rounds })} score={me?.score ?? 0} />}
      stage={
        <div className="card">
          <ul className="list">
            {showResult
              ? [...last.seats]
                  .sort((a, b) => a.rank - b.rank)
                  .map((s) => (
                    <li key={s.id}>
                      <span>
                        {t(ORD[s.rank] ?? '')} {nameOf(s.id)}{' '}
                        <span className="muted small">
                          {t(KINDS[s.kind])} · {t('said {rank}', { rank: t(ORD[s.prediction] ?? '') })} · +{s.points}
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
                        {s.rerolled ? `${t('rerolled')} · ` : ''}
                        {s.done ? t('ready') : t('thinking…')}
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
          <p className="muted small center">{t('Triple › straight › pair › nothing; then the higher sum.')}</p>
        </div>
      }
      controls={
        <>
          <div className="dice-prediction-mine">
            <figure>
              <Die n={v.open} />
              <figcaption>{t('open')}</figcaption>
            </figure>
            <figure>
              <Die n={v.secret} tone="secret" />
              <figcaption>{t('secret')}</figcaption>
            </figure>
            <figure>
              <Die n={mine ? mine.dice[2] : null} tone="blind" />
              <figcaption>{t('blind')}</figcaption>
            </figure>
          </div>
          <p className="ask">{ask}</p>
          {canBuild && (
            <div className="row">
              <button type="button" className="btn" onClick={() => move({ type: 'keep' })}>
                {t('Keep')}
              </button>
              <button type="button" className="btn primary" onClick={() => move({ type: 'reroll' })}>
                {t('Reroll secret')}
              </button>
            </div>
          )}
          {canPredict && (
            <div className="dice-prediction-ranks">
              {Array.from({ length: v.players }, (_, i) => i + 1).map((r) => (
                <button key={r} type="button" className="key" onClick={() => move({ type: 'predict', rank: r })}>
                  {t(ORD[r] ?? '')}
                </button>
              ))}
            </div>
          )}
          <KeyHint>{t('K keep · R reroll · 1–{n} predict your rank.', { n: v.players })}</KeyHint>
        </>
      }
    />
  );
}
