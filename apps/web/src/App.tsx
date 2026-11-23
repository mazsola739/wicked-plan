import { useEffect, useState } from 'react';
import { nameOf } from './games/common';
import { SCREENS } from './games/registry';
import { HowToPlay } from './HowTo';
import { setPaused, start, stop, usePlay } from './play';
import { Snapshot } from './Snapshot';
import type { GameInfo, PlayerResult } from './types';

const KINDS: [string, string][] = [
  ['main', 'Main games'],
  ['mini-solo', 'Mini games'],
  ['mini-team', 'Team games'],
  ['prison', 'Prison games'],
  ['hidden', 'Hidden games'],
  ['exile', 'Exile games'],
  ['final-duel', 'Final duels'],
];

export function App() {
  const play = usePlay();
  const current = play.current;
  const info = play.games.find((g) => g.id === current?.gameId);
  return (
    <div className="app">
      {play.error && <div className="banner error">{play.error}</div>}
      {!play.ready ? (
        <p className="muted center">Loading…</p>
      ) : current && info ? (
        <GameView info={info} view={current.view} results={current.results} />
      ) : (
        <Picker games={play.games} />
      )}
    </div>
  );
}

const LEVELS = { easy: 'Easy', medium: 'Medium', hard: 'Hard' };

function seatsText(g: GameInfo): string {
  const seats = Math.max(g.minPlayers, Math.min(4, g.maxPlayers));
  return seats === 1 ? 'Solo' : `You + ${seats - 1} bot${seats > 2 ? 's' : ''}`;
}

function Picker({ games }: { games: GameInfo[] }) {
  return (
    <div className="screen picker">
      <header>
        <h1 className="title">Wicked Plan</h1>
        <p className="muted">Pick a game. Puzzles, bluffs and duels; in the games for several players you play against bots.</p>
      </header>
      {KINDS.map(([kind, label]) => {
        const list = games.filter((g) => g.kind === kind);
        if (!list.length) return null;
        return (
          <section key={kind}>
            <h2 className="group-title">{label}</h2>
            <ul className="games">
              {list.map((g) => (
                <li key={g.id} className={`game-card ${g.difficulty}`}>
                  <Snapshot id={g.id} />
                  <button type="button" className="game-btn" onClick={() => start(g.id)}>
                    <span className="game-head">
                      <strong>{g.name}</strong>
                      <span className="level">{LEVELS[g.difficulty]}</span>
                    </span>
                    <span className="game-short">{g.short}</span>
                    <em>{seatsText(g)}</em>
                  </button>
                </li>
              ))}
            </ul>
          </section>
        );
      })}
    </div>
  );
}

// The rules open by themselves the first time a game is played on this browser.
// Storage may be blocked; then the rules just open every time.
function seen(id: string): boolean {
  try {
    return localStorage.getItem(`wp-seen-${id}`) === '1';
  } catch {
    return false;
  }
}

function markSeen(id: string): void {
  try {
    localStorage.setItem(`wp-seen-${id}`, '1');
  } catch {}
}

function GameView({ info, view, results }: { info: GameInfo; view: unknown; results: PlayerResult[] | null }) {
  const Screen = SCREENS[info.id];
  const [rules, setRules] = useState(() => !seen(info.id));
  useEffect(() => markSeen(info.id), [info.id]);
  useEffect(() => {
    setPaused(rules);
    return () => setPaused(false);
  }, [rules]);
  return (
    <div className="screen">
      <nav className="game-nav">
        <button type="button" className="btn small ghost" onClick={stop}>
          ← Games
        </button>
        {!rules && (
          <button type="button" className="btn small ghost" onClick={() => setRules(true)}>
            How to play
          </button>
        )}
      </nav>
      {rules ? (
        <HowToPlay id={info.id} name={info.name} onClose={() => setRules(false)} />
      ) : (
        <Board info={info} view={view} results={results} Screen={Screen} />
      )}
    </div>
  );
}

function Board({
  info,
  view,
  results,
  Screen,
}: {
  info: GameInfo;
  view: unknown;
  results: PlayerResult[] | null;
  Screen: (typeof SCREENS)[string] | undefined;
}) {
  return (
    <>
      {results && <Results info={info} results={results} />}
      {/* After the end the final view stays below the results, so reveals (true weights, roles…) can be read. */}
      {view && Screen ? (
        <div className={results ? 'final-view' : undefined}>
          <Screen view={view} />
        </div>
      ) : (
        !results && <p className="muted center">Loading…</p>
      )}
    </>
  );
}

function Results({ info, results }: { info: GameInfo; results: PlayerResult[] }) {
  const mine = results.find((r) => r.playerId === 'you');
  const ranked = [...results].sort((a, b) => a.rank - b.rank || b.points - a.points);
  return (
    <section className="card result">
      <div className="label">{info.name}</div>
      <h2>{heading(results, mine)}</h2>
      <div className="score big">{mine?.points ?? 0} points</div>
      {results.length > 1 && (
        <ol className="list">
          {ranked.map((r) => (
            <li key={r.playerId} className={r.playerId === 'you' ? 'me' : ''}>
              <span>
                {r.rank}. {nameOf(r.playerId)}
              </span>
              <span className="score">{r.points}</span>
            </li>
          ))}
        </ol>
      )}
      <div className="row">
        <button type="button" className="btn ghost" onClick={stop}>
          Back to games
        </button>
        <button type="button" className="btn primary" onClick={() => start(info.id)}>
          Play again
        </button>
      </div>
    </section>
  );
}

function heading(results: PlayerResult[], mine: PlayerResult | undefined): string {
  if (results.length === 1) return 'Finished';
  // Team games give every seat the same rank and points.
  if (results.every((r) => r.rank === 1 && r.points === results[0]?.points)) return 'Team result';
  return mine?.rank === 1 ? 'You won!' : `You came ${ordinal(mine?.rank ?? 0)}`;
}

function ordinal(n: number): string {
  const s = ['th', 'st', 'nd', 'rd'];
  const v = n % 100;
  return `${n}${s[(v - 20) % 10] ?? s[v] ?? s[0]}`;
}
