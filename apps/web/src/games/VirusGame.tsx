// The Virus Game: hidden roles, contacts, reports and quarantine votes. Rules: docs/games/virus-game.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './VirusGame.css';

type Role = 'terrorist' | 'officer' | 'journalist' | 'researcher' | 'citizen';

interface AgentView {
  index: number;
  id: string;
  number: number;
  quarantined: boolean;
  revealed: boolean | null;
  done: boolean;
  role: Role | null;
  infected: boolean | null;
}

interface VirusView {
  phase: 'contact' | 'report' | 'vote' | 'reveal' | 'over';
  round: number;
  rounds: number;
  me: number;
  role: Role;
  roleText: string;
  mission: string;
  infected: boolean;
  immune: boolean;
  quarantined: boolean;
  agents: AgentView[];
  reports: { round: number; by: number; target: number }[];
  votes: { round: number; by: number; target: number | null }[];
  newInfections: number[];
  notes: string[];
  log: string[];
  acted: boolean;
  winner: 'terrorists' | 'city' | null;
  score: number;
}

const ACTION: Record<Role, string> = {
  terrorist: 'Infect',
  officer: 'Check',
  journalist: 'Watch',
  researcher: 'Vaccinate',
  citizen: 'Meet',
};
const PHASE: Record<string, string> = { contact: 'Contact', report: 'Report', vote: 'Vote' };

export default function VirusGame({ view }: ScreenProps) {
  const v = view as VirusView;
  const [picked, setPicked] = useState<number | null>(null);
  const [forPhase, setForPhase] = useState(`${v.round}${v.phase}`);
  if (forPhase !== `${v.round}${v.phase}`) {
    setForPhase(`${v.round}${v.phase}`);
    setPicked(null);
  }
  const byIndex = (i: number) => v.agents[i];
  const label = (i: number) => {
    const a = byIndex(i);
    return a ? `#${a.number} ${nameOf(a.id)}` : '?';
  };
  const acting = ['contact', 'report', 'vote'].includes(v.phase) && !v.acted && !(v.phase === 'contact' && v.quarantined);
  const canPick = (a: AgentView) => {
    if (!acting) return false;
    if (v.phase === 'contact') return a.index !== v.me && !a.quarantined;
    if (v.phase === 'report') return a.index !== v.me;
    return !a.quarantined;
  };
  const send = (target: number | null) => {
    if (v.phase === 'contact' && target !== null) move({ type: 'contact', target });
    else if (v.phase === 'report') move({ type: 'report', target });
    else if (v.phase === 'vote') move({ type: 'vote', target });
  };
  useKeys((key) => {
    if (key === 'Enter' && v.phase === 'reveal') move({ type: 'close' });
    else if (key === 'Enter' && acting && picked !== null) send(picked);
    else if (/^[0-9]$/.test(key) && acting) {
      const n = key === '0' ? 10 : Number(key);
      const a = v.agents.find((x) => x.number === n);
      if (a && canPick(a)) setPicked(a.index);
    } else return false;
    return true;
  });
  const sorted = [...v.agents].sort((a, b) => a.number - b.number);
  const status = v.role === 'terrorist' ? 'Carrier' : v.immune ? 'Immune' : v.infected ? 'Infected' : 'Healthy';
  const end = v.phase === 'reveal' || v.phase === 'over';
  const verb = v.phase === 'contact' ? ACTION[v.role] : v.phase === 'report' ? 'Accuse' : 'Quarantine';
  return (
    <Play
      top={
        <GameTop
          title="The Virus Game"
          step={end ? 'Roles revealed' : `Round ${v.round} of ${v.rounds} · ${PHASE[v.phase] ?? ''}`}
          score={v.score}
        />
      }
      stage={
        <>
          <section className={`virus-card virus-${v.role}`}>
            <div className="virus-me">
              <strong>
                #{byIndex(v.me)?.number} · {v.role}
              </strong>
              <span className={`virus-status ${status.toLowerCase()}`}>{status}</span>
            </div>
            <p className="small">{v.roleText}</p>
            <p className="small muted">Mission: {v.mission}</p>
            {v.notes.length > 0 && (
              <ul className="virus-notes small">
                {v.notes.map((n) => (
                  <li key={n}>{n}</li>
                ))}
              </ul>
            )}
          </section>
          <div className="virus-grid">
            {sorted.map((a) => (
              <button
                key={a.id}
                type="button"
                className={`virus-agent${picked === a.index ? ' picked' : ''}${a.quarantined ? ' out' : ''}${a.index === v.me ? ' me' : ''}`}
                disabled={!canPick(a)}
                onClick={() => setPicked(a.index)}
              >
                <span className="virus-num">#{a.number}</span>
                <span>{nameOf(a.id)}</span>
                <small>
                  {a.role && a.index !== v.me ? a.role : a.revealed === true ? 'terrorist' : a.revealed === false ? 'not a terrorist' : ''}
                  {a.quarantined ? ' · quarantine' : ''}
                  {end && a.infected ? ' · infected' : ''}
                </small>
                {!end && a.done && <span className="virus-done">✓</span>}
              </button>
            ))}
          </div>
        </>
      }
      controls={
        <>
          {end ? (
            <>
              <p className={`last ${(v.winner === 'terrorists') === (v.role === 'terrorist') ? 'ok' : 'bad'}`}>
                {v.winner === 'terrorists' ? 'The terrorists win.' : 'The city wins.'} Your score: {v.score}
              </p>
              <button type="button" className="btn primary" onClick={() => move({ type: 'close' })}>
                See results
              </button>
            </>
          ) : acting ? (
            <>
              <p className="muted small center">
                {v.phase === 'contact'
                  ? `Choose whom to meet (${ACTION[v.role].toLowerCase()}).`
                  : v.phase === 'report'
                    ? 'Accuse someone in public, or stay quiet.'
                    : 'Vote someone into quarantine, or nobody.'}
              </p>
              <button type="button" className="btn primary" disabled={picked === null} onClick={() => send(picked)}>
                {picked === null ? `${verb}…` : `${verb} ${label(picked)}`}
              </button>
              {v.phase !== 'contact' && (
                <button type="button" className="btn ghost" onClick={() => send(null)}>
                  {v.phase === 'report' ? 'Stay quiet' : 'Nobody'}
                </button>
              )}
              <KeyHint>Keys: a player's number picks them, Enter confirms.</KeyHint>
            </>
          ) : (
            <p className="muted center">{v.quarantined && v.phase === 'contact' ? 'You are in quarantine.' : 'Waiting for the others…'}</p>
          )}
          {v.newInfections.length > 0 && (
            <p className="small center">New infections per round: {v.newInfections.map((n) => String(n)).join(' · ')}</p>
          )}
          {v.reports.length > 0 && (
            <div>
              <div className="label">Reports</div>
              <ul className="virus-reports small">
                {v.reports
                  .slice()
                  .reverse()
                  .map((r) => (
                    <li key={`${r.round}-${r.by}`}>
                      R{r.round}: {label(r.by)} accuses {label(r.target)}
                    </li>
                  ))}
              </ul>
            </div>
          )}
          <div className="virus-log">
            {v.log
              .slice(-6)
              .reverse()
              .map((line, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows at the end
                <p key={i} className="small">
                  {line}
                </p>
              ))}
          </div>
        </>
      }
    />
  );
}
