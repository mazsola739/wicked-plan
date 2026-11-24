// Monster Alliance: bid for weapons, ally up, beat monsters together. Rules: docs/games/monster-alliance.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, NumberPad, Play, type ScreenProps, useKeys } from './common';
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

/**
 * The monster: a faceted (low-poly) horned head in red, lit from the top left like the animals of
 * Animal Tile Auction: white and black overlays make the facets, `eye` shapes are cut in the page's
 * near-black, with yellow slit pupils, white fangs and white glints.
 */
function MonsterFace() {
  return (
    <svg viewBox="0 0 40 40" role="img" aria-label="Monster">
      <path d="M9,12.5 5.5,7 4,1 9.5,5 15,9Z M31,12.5 34.5,7 36,1 30.5,5 25,9Z" />
      <path fill="#fff" fillOpacity=".35" d="M9,12.5 5.5,7 4,1 12,10.7Z" />
      <path fill="#fff" fillOpacity=".12" d="M4,1 9.5,5 15,9 12,10.7Z M36,1 30.5,5 25,9 28,10.7Z" />
      <path fill="#000" fillOpacity=".3" d="M31,12.5 34.5,7 36,1 28,10.7Z" />
      <path d="M13,8 20,7 27,8 33,13 36,20 38.5,23 34.5,25 33,29 26,35 20,36.5 14,35 7,29 5.5,25 1.5,23 4,20 7,13Z" />
      <path fill="#fff" fillOpacity=".4" d="M13,8 20,7 20,15 7,13Z" />
      <path fill="#fff" fillOpacity=".15" d="M27,8 20,7 20,15 33,13Z M7,13 20,15 14,24 4,20Z" />
      <path fill="#000" fillOpacity=".14" d="M33,13 20,15 26,24 36,20Z M4,20 14,24 20,36.5 14,35 7,29 5.5,25Z" />
      <path fill="#000" fillOpacity=".3" d="M36,20 26,24 20,36.5 26,35 33,29 34.5,25Z" />
      <path fill="#fff" fillOpacity=".45" d="M4,20 1.5,23 5.5,25Z" />
      <path fill="#fff" fillOpacity=".38" d="M20,18.5 23.2,24.2 20,26.4 16.8,24.2Z" />
      <path fill="#fff" fillOpacity=".14" d="M14,24 16.8,24.2 20,26.4 23.2,24.2 26,24 27.5,30 20,34 12.5,30Z" />
      <path fill="#000" fillOpacity=".35" d="M36,20 38.5,23 34.5,25Z M8,13.2 20,17.8 32,13.2 32.5,15 20,19.8 7.5,15Z" />
      <path
        className="eye"
        d="M9,16.5 17.5,20.3 16.5,23.5 11,22.5Z M31,16.5 22.5,20.3 23.5,23.5 29,22.5Z M9.5,27 20,29.5 30.5,27 28.5,32 20,34.5 11.5,32Z M17.6,25.2 19.4,25.9 18.4,26.9Z M22.4,25.2 20.6,25.9 21.6,26.9Z"
      />
      <path className="ma-eye" d="M13.8,19.6 14.9,21.1 13.8,22.6 12.7,21.1Z M26.2,19.6 27.3,21.1 26.2,22.6 25.1,21.1Z" />
      <path
        className="ma-teeth"
        d="M12.2,27.6 15.8,28.5 13.8,32Z M27.8,27.6 24.2,28.5 26.2,32Z M16.6,33.5 18.1,31.2 19.6,34.2Z M20.4,34.2 21.9,31.2 23.4,33.5Z"
      />
      <path fill="#fff" d="M11.6,18.7 12.2,19.3 11.6,19.9 11,19.3Z M24,20.2 24.6,20.8 24,21.4 23.4,20.8Z" />
    </svg>
  );
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
      top={<GameTop title="Monster Alliance" step={`Monster ${v.round} of ${v.rounds} · ${PHASE[v.phase] ?? ''}`} score={v.score} />}
      stage={
        <>
          {m && (
            <section className="ma-monster">
              <div className="ma-face">
                <MonsterFace />
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
