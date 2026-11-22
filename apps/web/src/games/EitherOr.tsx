// Either Or. Rules page: docs/games/either-or.md

import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './EitherOr.css';

type Kind = 'majority' | 'minority' | 'lowestUnique' | 'shareGrab' | 'finger' | 'loneWolf';
interface Round {
  kind: Kind;
  prompt: string;
  options: string[];
}
interface Outcome extends Round {
  number: number;
  picks: number[];
  gains: number[];
  note: string;
}
interface EitherView {
  players: string[];
  me: number;
  number: number;
  total: number;
  round: Round | null;
  myPick: number | null;
  done: boolean[];
  scores: number[];
  timeLeftMs: number;
  roundMs: number;
  last: Outcome | null;
  over: boolean;
}

const TITLES: Record<Kind, string> = {
  majority: 'Majority wins · +1',
  minority: 'Minority wins · +2',
  lowestUnique: 'Surprise: Lowest Unique',
  shareGrab: 'Surprise: Share or Grab',
  finger: 'Surprise: Point the Finger',
  loneWolf: 'Surprise: Lone Wolf · alone scores +4',
};
const LETTERS = 'ABCDEFGH';
const said = (line: string) => line.replace(/\bbot-\d+\b|\byou\b/g, nameOf);

/** The choices of a round as [value, label]. */
function choices(round: Round, view: EitherView): [number, string][] {
  if (round.kind === 'lowestUnique') return [1, 2, 3, 4, 5].map((v) => [v, String(v)]);
  if (round.kind === 'finger') return view.players.flatMap((p, i) => (i === view.me ? [] : [[i, nameOf(p)] as [number, string]]));
  return round.options.map((o, i) => [i, o]);
}

function pickLabel(o: Outcome, view: EitherView, v: number): string {
  if (o.kind === 'lowestUnique') return String(v);
  if (o.kind === 'finger') return `→ ${nameOf(view.players[v] ?? '')}`;
  return o.options[v] ?? '';
}

export default function EitherOr({ view: raw }: ScreenProps) {
  const view = raw as EitherView;
  const round = view.round;
  const open = round !== null && view.myPick === null;
  const list = round ? choices(round, view) : [];
  const pick = (value: number) => open && move({ type: 'pick', value });
  useKeys((key) => {
    if (!round || !open) return false;
    const k = key.toUpperCase();
    if (round.kind === 'finger') {
      const seat = LETTERS.indexOf(k);
      if (key.length !== 1 || seat < 0 || !list.some(([v]) => v === seat)) return false;
      pick(seat);
    } else if (round.kind === 'lowestUnique' && /^[1-5]$/.test(key)) pick(Number(key));
    else if (/^[12]$/.test(key) && round.options.length === 2) pick(Number(key) - 1);
    else return false;
    return true;
  });
  const waiting = view.players.filter((_, i) => !view.done[i]).map(nameOf);
  return (
    <Play
      top={
        <>
          <GameTop
            title="Either Or"
            step={view.over ? 'Game over' : `Round ${view.number} of ${view.total}`}
            score={view.scores[view.me] ?? 0}
          />
          {!view.over && <TimerBar leftMs={view.timeLeftMs} totalMs={view.roundMs} />}
        </>
      }
      stage={
        <>
          {round && (
            <div className={`eo-round${round.kind === 'majority' || round.kind === 'minority' ? '' : ' surprise'}`}>
              <div className="label">{TITLES[round.kind]}</div>
              <p className="eo-prompt">{round.prompt}</p>
            </div>
          )}
          <ul className="eo-scores">
            {view.players.map((p, i) => (
              <li key={p} className={i === view.me ? 'me' : ''}>
                <span>
                  {round?.kind === 'finger' && <em>{LETTERS[i]} </em>}
                  {nameOf(p)}
                </span>
                <span className={`eo-tick${view.done[i] ? ' on' : ''}`}>{view.done[i] && !view.over ? '✓' : ''}</span>
                <strong className="score">{view.scores[i]}</strong>
              </li>
            ))}
          </ul>
        </>
      }
      controls={
        <>
          {round && (
            <div className={`eo-choices n${list.length}`}>
              {list.map(([v, label], i) => (
                <button
                  key={v}
                  type="button"
                  className={`btn eo-choice${view.myPick === v ? ' on' : ''}${list.length === 2 ? ` side${i}` : ''}`}
                  disabled={!open}
                  onClick={() => pick(v)}
                >
                  {label}
                </button>
              ))}
            </div>
          )}
          {round && !open && <p className="muted center">Waiting for {waiting.join(', ') || 'the others'}…</p>}
          {view.last && (
            <div className="card">
              <div className="label">
                Round {view.last.number}: {TITLES[view.last.kind]}
              </div>
              {view.last.options.length > 0 && view.last.kind !== 'shareGrab' && <p className="muted small">{view.last.prompt}</p>}
              <ul className="list">
                {view.players.map((p, i) => (
                  <li key={p}>
                    <span>{nameOf(p)}</span>
                    <span>{pickLabel(view.last as Outcome, view, view.last?.picks[i] ?? 0)}</span>
                    <strong className={`score${(view.last?.gains[i] ?? 0) < 0 ? ' eo-minus' : ''}`}>
                      {(view.last?.gains[i] ?? 0) > 0 ? '+' : ''}
                      {view.last?.gains[i]}
                    </strong>
                  </li>
                ))}
              </ul>
              <p className="ask small">{said(view.last.note)}</p>
            </div>
          )}
          <KeyHint>Keys: 1 / 2 for the answers, 1–5 for numbers, A–H for players.</KeyHint>
        </>
      }
    />
  );
}
