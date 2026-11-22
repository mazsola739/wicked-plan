// Monster Alliance: bid for weapons, ally up, beat monsters together. Rules: docs/games/monster-alliance.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, NumberPad, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './MonsterAlliance.css';

interface Monster {
  name: string;
  strength: number;
  reward: number;
  bonus: number;
}

interface Outcome {
  members: number[];
  played: number[][];
  power: number;
  won: boolean;
  shares: number[];
}

interface MaView {
  phase: 'bid' | 'ally' | 'attack' | 'over';
  round: number;
  rounds: number;
  timeLeftMs: number;
  phaseTimeMs: number;
  me: number;
  monster: Monster | null;
  fighters: { index: number; id: string; points: number; cards: number; done: boolean }[];
  hand: number[];
  acted: boolean;
  groups: number[][];
  last: Outcome[];
  log: string[];
  score: number;
}

const PHASE: Record<string, string> = { bid: 'Bid for a weapon', ally: 'Form alliances', attack: 'Attack!' };

function names(line: string): string {
  return line.replace(/\b(you|bot-\d+)\b/g, (id) => nameOf(id));
}

export default function MonsterAlliance({ view }: ScreenProps) {
  const v = view as MaView;
  const [picked, setPicked] = useState<number[]>([]);
  const [step, setStep] = useState(`${v.round}${v.phase}`);
  if (step !== `${v.round}${v.phase}`) {
    setStep(`${v.round}${v.phase}`);
    setPicked([]);
  }
  const toggle = (i: number, max: number) =>
    setPicked((cur) => (cur.includes(i) ? cur.filter((c) => c !== i) : cur.length < max ? [...cur, i] : cur));
  const confirm = () => {
    if (v.phase === 'ally') move({ type: 'ally', partners: picked });
    else if (v.phase === 'attack') move({ type: 'attack', cards: picked });
  };
  useKeys((key) => {
    if (v.acted || (v.phase !== 'attack' && v.phase !== 'ally')) return false;
    if (/^[1-9]$/.test(key)) {
      const i = Number(key) - 1;
      if (v.phase === 'attack' && i < v.hand.length) toggle(i, 99);
      else if (v.phase === 'ally') {
        const other = v.fighters.filter((f) => f.index !== v.me)[i];
        if (other) toggle(other.index, 2);
      }
    } else if (key === 'Enter') confirm();
    else return false;
    return true;
  });
  const me = v.fighters[v.me];
  const myGroup = v.groups.find((g) => g.includes(v.me)) ?? [v.me];
  const power = picked.reduce((sum, i) => sum + (v.hand[i] ?? 0), 0);
  const m = v.monster;
  return (
    <Play
      top={
        <>
          <GameTop title="Monster Alliance" step={`Monster ${v.round} of ${v.rounds} · ${PHASE[v.phase] ?? ''}`} score={v.score} />
          <TimerBar leftMs={v.timeLeftMs} totalMs={v.phaseTimeMs} />
        </>
      }
      stage={
        <>
          {m && (
            <section className="ma-monster">
              <div className="ma-face">
                <svg viewBox="0 0 100 100" role="img" aria-label="Monster">
                  <path d="M15 85 Q10 40 30 25 L35 8 L45 22 Q50 18 55 22 L65 8 L70 25 Q90 40 85 85 Z" />
                  <circle cx="38" cy="45" r="7" className="ma-eye" />
                  <circle cx="62" cy="45" r="7" className="ma-eye" />
                  <path d="M30 68 L38 62 L46 70 L54 62 L62 70 L70 62" className="ma-teeth" />
                </svg>
              </div>
              <div>
                <h3>{m.name}</h3>
                <p>
                  Strength <strong>{m.strength}</strong> · Reward <strong>{m.reward}</strong>
                </p>
                {v.phase === 'bid' && (
                  <p className="small muted">
                    Extra weapon for auction: <strong className="ma-bonus">{m.bonus}</strong>
                  </p>
                )}
              </div>
            </section>
          )}
          <ul className="list ma-players">
            {v.fighters.map((f) => {
              const g = v.groups.find((x) => x.includes(f.index));
              return (
                <li key={f.id} className={f.index === v.me ? 'me' : ''}>
                  <span>
                    {nameOf(f.id)}
                    {g && g.length > 1 && (
                      <small className="ma-ally">
                        {' '}
                        with{' '}
                        {g
                          .filter((x) => x !== f.index)
                          .map((x) => nameOf(v.fighters[x]?.id ?? ''))
                          .join(', ')}
                      </small>
                    )}
                  </span>
                  <span className="small">
                    {f.points} pts · {f.cards} cards {f.done ? '✓' : '…'}
                  </span>
                </li>
              );
            })}
          </ul>
          {v.last.length > 0 && (
            <div>
              <div className="label">Last attack</div>
              <ul className="ma-last small">
                {v.last.map((o) => (
                  <li key={o.members.join('-')} className={o.won ? 'won' : ''}>
                    {o.members.map((x) => nameOf(v.fighters[x]?.id ?? '')).join(' + ')}: power {o.power} [
                    {o.played.map((c) => c.join('+') || '0').join(' | ')}]{o.won ? ` → +${o.shares.join(' / +')}` : ''}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </>
      }
      controls={
        <>
          {v.acted && v.phase !== 'over' && <p className="muted center">Waiting for the others…</p>}
          {!v.acted && v.phase === 'bid' && (
            <>
              <p className="muted small center">Bid points for the extra weapon (you have {me?.points ?? 0}).</p>
              <NumberPad round={v.round} onSubmit={(amount) => move({ type: 'bid', amount })} />
            </>
          )}
          {!v.acted && v.phase === 'ally' && (
            <>
              <p className="muted small center">Pick up to two partners. An alliance forms only if they pick you too.</p>
              <div className="ma-choices">
                {v.fighters
                  .filter((f) => f.index !== v.me)
                  .map((f) => (
                    <button
                      key={f.id}
                      type="button"
                      className={`btn${picked.includes(f.index) ? ' primary' : ''}`}
                      onClick={() => toggle(f.index, 2)}
                    >
                      {nameOf(f.id)} · {f.cards} cards
                    </button>
                  ))}
              </div>
              <button type="button" className="btn primary" onClick={confirm}>
                {picked.length ? 'Ask them' : 'Fight alone'}
              </button>
              <KeyHint>Keys: 1–7 pick players, Enter confirms.</KeyHint>
            </>
          )}
          {!v.acted && v.phase === 'attack' && (
            <>
              <p className="muted small center">
                {myGroup.length > 1
                  ? `Your alliance: ${myGroup.map((x) => nameOf(v.fighters[x]?.id ?? '')).join(' + ')}.`
                  : 'You fight alone.'}{' '}
                The monster's strength is {m?.strength ?? 0}.
              </p>
              <div className="cards ma-hand">
                {v.hand.map((c, i) => (
                  <button
                    // biome-ignore lint/suspicious/noArrayIndexKey: hand cards are positional
                    key={i}
                    type="button"
                    className={`card-btn${picked.includes(i) ? ' picked' : ''}`}
                    onClick={() => toggle(i, 99)}
                  >
                    {c}
                    <small>{i + 1}</small>
                  </button>
                ))}
              </div>
              <button type="button" className="btn primary" onClick={confirm}>
                {picked.length ? `Attack with ${power}` : 'Hold back'}
              </button>
              <KeyHint>Keys: 1–9 pick cards, Enter attacks.</KeyHint>
            </>
          )}
          {v.phase !== 'attack' && v.hand.length > 0 && (
            <p className="small center">
              Your weapons: <strong>{v.hand.join(' · ')}</strong>
            </p>
          )}
          <div className="ma-log">
            {v.log
              .slice(-5)
              .reverse()
              .map((line, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows at the end
                <p key={i} className="small">
                  {names(line)}
                </p>
              ))}
          </div>
        </>
      }
    />
  );
}
