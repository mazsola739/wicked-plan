import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { Crown, Emblem, Peeker } from './Emblem';
import { GameIcon } from './GameIcon';
import { GameIdContext, nameOf, useKeys } from './games/common';
import { SCREENS } from './games/registry';
import { HowToPlay, rulesHidden } from './HowTo';
import { setPaused, start, stop, usePlay } from './play';
import { Replay } from './Replay';
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
        <>
          <GameView info={info} view={current.view} results={current.results} />
          {play.replay && <Replay moment={play.replay} />}
        </>
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

const STEPS: [string, string][] = [
  ['Pick a game', 'Fifty-one puzzles, bluffs and duels, from a two-minute count to a long game of alliances.'],
  ['Read the brief', 'Every game opens with a one-minute How to play. The clock waits while you read.'],
  ['Outsmart the table', 'Solo puzzles test your head, a few against the clock. Everything else is you against bots that bluff back.'],
];

/** The categories (docs/games.md, Categories), in the order the filter shows them. */
const CATEGORIES: [string, string][] = [
  ['cards', 'Cards'],
  ['bluff', 'Bluff'],
  ['secret-role', 'Secret role'],
  ['bidding', 'Bidding'],
  ['memory', 'Memory'],
  ['logic', 'Logic'],
  ['numbers', 'Numbers'],
  ['board', 'Board'],
  ['dice', 'Dice'],
  ['words', 'Words'],
  ['speed', 'Speed'],
  ['alliance', 'Alliance'],
];
const CATEGORY_NAME = Object.fromEntries(CATEGORIES);

type Filter = { kind: string; tag: string; level: string; seats: string; query: string };
const ANY: Filter = { kind: 'all', tag: 'all', level: 'all', seats: 'all', query: '' };

function matches(g: GameInfo, f: Filter): boolean {
  const solo = g.maxPlayers === 1;
  const words = f.query.toLowerCase().split(/\s+/).filter(Boolean);
  const text = `${g.name} ${g.short} ${g.tags.map((t) => CATEGORY_NAME[t]).join(' ')}`.toLowerCase();
  return (
    (f.kind === 'all' || g.kind === f.kind) &&
    (f.tag === 'all' || g.tags.includes(f.tag)) &&
    (f.level === 'all' || g.difficulty === f.level) &&
    (f.seats === 'all' || (f.seats === 'solo') === solo) &&
    words.every((w) => text.includes(w))
  );
}

function Picker({ games }: { games: GameInfo[] }) {
  const [filter, setFilter] = useState<Filter>(ANY);
  // Below the sidebar width the filters fold away behind a button that counts the active ones.
  const [filtersOpen, setFiltersOpen] = useState(false);
  const active = FILTERS.filter((f) => filter[f.key] !== 'all').length;
  const shown = games.filter((g) => matches(g, filter));
  const random = () => {
    const pick = shown[Math.floor(Math.random() * shown.length)] ?? games[0];
    if (pick) start(pick.id);
  };
  return (
    <div className="screen picker">
      <nav className="topbar">
        <a className="brand" href="#top" aria-label="Wicked Plan, top of the page">
          <Crown className="brand-crown" />
          <span>Wicked Plan</span>
        </a>
        <span className="topnav">
          <a href="#how">How it works</a>
          <a href="#games">Games</a>
        </span>
      </nav>

      <header className="hero" id="top">
        <Emblem className="hero-emblem" watching />
        <div className="hero-text">
          <p className="wip">Under construction</p>
          <p className="label tagline">A digital arena of strategy</p>
          <h1 className="title">Wicked Plan</h1>
          <p className="lead">Puzzles, bluffs and duels. Think ahead, trust carefully: in the games for several players you face bots.</p>
          <div className="hero-actions">
            <a className="btn primary" href="#games">
              Enter the arena
            </a>
            <button type="button" className="btn violet" onClick={random}>
              Random game
            </button>
          </div>
          <p className="muted small credit">Inspired by the Netflix show The Devil's Plan. An unofficial fan project.</p>
        </div>
      </header>

      <Divider />

      <section className="how" id="how" aria-labelledby="how-title">
        <h2 className="section-title" id="how-title">
          How it works
        </h2>
        <ol className="steps">
          {STEPS.map(([title, text], i) => (
            <li key={title}>
              <span className="step-no">{['I', 'II', 'III'][i]}</span>
              <strong>{title}</strong>
              <span className="muted">{text}</span>
            </li>
          ))}
        </ol>
      </section>

      <Divider />

      <section className="catalog" id="games" aria-labelledby="games-title">
        <h2 className="section-title" id="games-title">
          Games
        </h2>
        <div className="catalog-body">
          <aside className="catalog-side" aria-label="Find a game">
            <Search value={filter.query} onChange={(query) => setFilter({ ...filter, query })} />
            <button
              type="button"
              className="btn small ghost filters-toggle"
              aria-expanded={filtersOpen}
              aria-controls="filters"
              onClick={() => setFiltersOpen(!filtersOpen)}
            >
              {active ? `Filters · ${active}` : 'Filters'}
            </button>
            <Filters open={filtersOpen} filter={filter} onChange={setFilter} />
            <p className="muted small" aria-live="polite">
              {shown.length === games.length ? `${games.length} games` : `${shown.length} of ${games.length} games`}
            </p>
            {/* Desktop only: the games found, small icon and name, in the order of the cards. */}
            <ul className="side-list">
              {KINDS.flatMap(([kind]) => shown.filter((g) => g.kind === kind)).map((g) => (
                <li key={g.id}>
                  <button type="button" onClick={() => start(g.id)}>
                    <GameIcon id={g.id} />
                    <span>{g.name}</span>
                  </button>
                </li>
              ))}
            </ul>
          </aside>
          <div className="catalog-list">
            {shown.length === 0 && (
              <div className="empty">
                <p className="muted">No game fits these filters.</p>
                <button type="button" className="btn ghost small" onClick={() => setFilter(ANY)}>
                  Clear filters
                </button>
              </div>
            )}
            {KINDS.map(([kind, label]) => {
              const list = shown.filter((g) => g.kind === kind);
              if (!list.length) return null;
              return (
                <section key={kind}>
                  <h3 className="group-title">{label}</h3>
                  <ul className="games">
                    {list.map((g) => (
                      <li key={g.id} className={`game-card ${g.difficulty}`}>
                        <Snapshot id={g.id} />
                        <button type="button" className="game-btn" onClick={() => start(g.id)}>
                          <span className="game-head">
                            <strong>
                              <GameIcon id={g.id} className="name-icon" />
                              {g.name}
                            </strong>
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
        </div>
      </section>

      <Divider />
      <footer className="footer muted small">
        <Crown className="footer-crown" />
        <span>Wicked Plan · an unofficial fan project · all games run in your browser</span>
        <nav className="footer-links" aria-label="About the maker">
          <a href="https://mazsola739.github.io/mazsola739/" target="_blank" rel="noreferrer">
            Portfolio
          </a>
          <a href="https://github.com/mazsola739/wicked-plan" target="_blank" rel="noreferrer">
            Source code
          </a>
        </nav>
      </footer>
    </div>
  );
}

const FILTERS: { key: keyof Filter; label: string; options: [string, string][] }[] = [
  { key: 'kind', label: 'Type', options: [['all', 'All'], ...KINDS.map(([k, l]): [string, string] => [k, l.replace(/ games$/, '')])] },
  { key: 'tag', label: 'Category', options: [['all', 'All'], ...CATEGORIES] },
  {
    key: 'level',
    label: 'Difficulty',
    options: [
      ['all', 'All'],
      ['easy', 'Easy'],
      ['medium', 'Medium'],
      ['hard', 'Hard'],
    ],
  },
  {
    key: 'seats',
    label: 'Players',
    options: [
      ['all', 'All'],
      ['solo', 'Solo'],
      ['bots', 'With bots'],
    ],
  },
];

function Filters({ open, filter, onChange }: { open: boolean; filter: Filter; onChange: (f: Filter) => void }) {
  return (
    <div id="filters" className={`filters${open ? ' open' : ''}`}>
      {FILTERS.map((f) => (
        <fieldset key={f.key} className="filter">
          <legend className="label">{f.label}</legend>
          <div className="chips">
            {f.options.map(([value, label]) => (
              <button
                key={value}
                type="button"
                className={`chip${value !== 'all' && f.key === 'level' ? ` ${value}` : ''}`}
                aria-pressed={filter[f.key] === value}
                onClick={() => onChange({ ...filter, [f.key]: value })}
              >
                {label}
              </button>
            ))}
          </div>
        </fieldset>
      ))}
    </div>
  );
}

/** Searches the names and short texts; "/" jumps to it on a keyboard. */
function Search({ value, onChange }: { value: string; onChange: (q: string) => void }) {
  const input = useRef<HTMLInputElement>(null);
  useKeys((key) => {
    if (key !== '/') return false;
    input.current?.focus();
    return true;
  });
  return (
    <label className="search">
      <svg className="search-icon" viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="10.5" cy="10.5" r="6.5" />
        <path d="M15.5 15.5 L21 21" />
      </svg>
      <input
        ref={input}
        type="search"
        value={value}
        placeholder="Search games"
        aria-label="Search games"
        onChange={(e) => onChange(e.target.value)}
      />
      <kbd className="key-hint search-key">/</kbd>
    </label>
  );
}

function Divider() {
  return (
    <div className="divider" aria-hidden="true">
      <span />
    </div>
  );
}

function GameView({ info, view, results }: { info: GameInfo; view: unknown; results: PlayerResult[] | null }) {
  const Screen = SCREENS[info.id];
  // The rules open by themselves when a game starts, unless "Don't show again" was ticked for it.
  const [rules, setRules] = useState(() => !rulesHidden(info.id));
  useEffect(() => {
    setPaused(rules);
    return () => setPaused(false);
  }, [rules]);
  // Each switch between the rules and the board starts at the top, with the game bar in view.
  // biome-ignore lint/correctness/useExhaustiveDependencies: runs on purpose whenever `rules` flips
  useLayoutEffect(() => {
    window.scrollTo(0, 0);
  }, [rules]);
  // Closing the rules puts focus back on the button that opens them.
  const howTo = useRef<HTMLButtonElement>(null);
  const wasOpen = useRef(rules);
  useEffect(() => {
    if (wasOpen.current && !rules) howTo.current?.focus({ preventScroll: true });
    wasOpen.current = rules;
  }, [rules]);
  return (
    <div className="screen game-screen tv-on">
      {/* While you play, the Observer's face peeks at you from behind the game (the transmission shows it otherwise). */}
      <Peeker active={!rules && !results} />
      <nav className="game-nav">
        <button type="button" className="btn small ghost" onClick={stop}>
          ← Games
        </button>
        {!rules && (
          <button ref={howTo} type="button" className="btn small ghost" onClick={() => setRules(true)}>
            How to play
          </button>
        )}
      </nav>
      {rules ? (
        <HowToPlay key={info.id} id={info.id} name={info.name} onClose={() => setRules(false)} />
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
          <GameIdContext.Provider value={info.id}>
            <Screen view={view} />
          </GameIdContext.Provider>
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
      <Emblem className="result-emblem" />
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
