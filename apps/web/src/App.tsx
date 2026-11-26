import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { airedOf, seasonOf, stageOf } from './aired';
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

/** The five difficulty levels (docs/games.md); a game's level n of 5 is the n-th key. */
const LEVELS = { easy: 'Easy', normal: 'Normal', medium: 'Medium', hard: 'Hard', expert: 'Expert' };

/** Players, as a board game box says it: 1 for a solo game, otherwise the fewest players and a plus. */
function playersText(g: GameInfo): string {
  return g.maxPlayers === 1 ? '1' : `${g.minPlayers}+`;
}

const STEPS: [string, string][] = [
  ['Pick a game', 'Fifty-one puzzles, bluffs and duels, from a two-minute count to a long game of alliances.'],
  ['Read the brief', 'Every game opens with a one-minute How to play. The clock waits while you read.'],
  ['Outsmart the table', 'Solo puzzles test your head, a few against the clock. Everything else is you against bots that bluff back.'],
];

/** The categories (docs/games.md, Categories), in the order the filter shows them. */
const CATEGORIES: [string, string][] = [
  ['logic-puzzles', 'Logic & Puzzles'],
  ['strategy-territory', 'Strategy & Territory'],
  ['bluff-deduction', 'Bluff & Deduction'],
  ['social-alliances', 'Social & Alliances'],
  ['auction-economy', 'Auction & Economy'],
  ['memory-observation', 'Memory & Observation'],
  ['speed-risk', 'Speed & Risk'],
];
const CATEGORY_NAME = Object.fromEntries(CATEGORIES);

type Filter = { tag: string; level: string; seats: string; season: string; stage: string; query: string };
const ANY: Filter = { tag: 'all', level: 'all', seats: 'all', season: 'all', stage: 'all', query: '' };

function matches(g: GameInfo, f: Filter): boolean {
  const solo = g.maxPlayers === 1;
  const words = f.query.toLowerCase().split(/\s+/).filter(Boolean);
  const text = `${g.name} ${g.short} ${g.tags.map((t) => CATEGORY_NAME[t]).join(' ')}`.toLowerCase();
  return (
    (f.tag === 'all' || g.tags.includes(f.tag)) &&
    (f.level === 'all' || g.difficulty === f.level) &&
    (f.seats === 'all' || (f.seats === 'solo') === solo) &&
    (f.season === 'all' || seasonOf(g.id) === f.season) &&
    (f.stage === 'all' || stageOf(g.id) === f.stage) &&
    words.every((w) => text.includes(w))
  );
}

function Picker({ games }: { games: GameInfo[] }) {
  const [filter, setFilter] = useState<Filter>(ANY);
  // Below the sidebar width the filters fold away behind a button that counts the active ones.
  const [filtersOpen, setFiltersOpen] = useState(false);
  const active = [...FILTERS.map((f) => f.key), ...SHOW_PICKS.map((p) => p.key)].filter((k) => filter[k] !== 'all').length;
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
        <p className="games-count" aria-live="polite">
          {shown.length === games.length ? `${games.length} games` : `${shown.length} of ${games.length} games`}
        </p>
        {/* Jump to a type's group of cards; a type with no game left after the filters is greyed out. */}
        <nav className="kind-nav" aria-label="Jump to a type of game">
          {KINDS.map(([kind, label]) => (
            <button
              key={kind}
              type="button"
              className={`chip kind-${kind}`}
              disabled={!shown.some((g) => g.kind === kind)}
              onClick={() => document.getElementById(`group-${kind}`)?.scrollIntoView({ behavior: 'smooth', block: 'start' })}
            >
              {label.replace(/ games$/, '')}
            </button>
          ))}
        </nav>
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
            {active > 0 && (
              <button
                type="button"
                className="btn ghost small filter-clear"
                onClick={() => setFilter({ ...ANY, query: filter.query, seats: filter.seats, level: filter.level })}
              >
                Clear filters
              </button>
            )}
            {/* Desktop only: the games found, small icon and name, A to Z. */}
            <ul className="side-list">
              {[...shown]
                .sort((a, b) => a.name.localeCompare(b.name, 'en'))
                .map((g) => (
                  <li key={g.id}>
                    <button type="button" className={`kind-${g.kind}`} onClick={() => start(g.id)}>
                      <GameIcon id={g.id} />
                      <span>{g.name}</span>
                    </button>
                  </li>
                ))}
            </ul>
          </aside>
          <div className="catalog-list">
            {/* Over the cards: solo or with bots, and under it the difficulty (the other filters are
                in the sidebar). */}
            <div className="card-tabs">
              <fieldset className="seat-tabs">
                <legend className="sr-only">Players</legend>
                {SEAT_TABS.map(([value, label]) => (
                  <button
                    key={value}
                    type="button"
                    className="seat-tab"
                    aria-pressed={filter.seats === value}
                    onClick={() => setFilter({ ...filter, seats: value })}
                  >
                    {label}
                  </button>
                ))}
              </fieldset>
              <fieldset className="level-tabs">
                <legend className="sr-only">Difficulty</legend>
                {LEVEL_TABS.map(([value, label]) => (
                  <button
                    key={value}
                    type="button"
                    className={`level-tab ${value}`}
                    aria-pressed={filter.level === value}
                    onClick={() => setFilter({ ...filter, level: value })}
                  >
                    {label}
                  </button>
                ))}
              </fieldset>
            </div>
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
                <section key={kind} id={`group-${kind}`} className="game-group">
                  <h3 className={`group-title kind-${kind}`}>{label}</h3>
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
                          {/* The card's foot: players on the left, where it was on the show on the right. */}
                          <span className="card-foot">
                            <span className="players">
                              <svg viewBox="0 0 16 16" aria-hidden="true">
                                <circle cx="8" cy="5" r="3" />
                                <path d="M2.5 15 C2.5 10.5 13.5 10.5 13.5 15 Z" />
                              </svg>
                              <span className="sr-only">Players: </span>
                              {playersText(g)}
                            </span>
                            <Aired id={g.id} />
                          </span>
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

const SEAT_TABS: [string, string][] = [
  ['all', 'All games'],
  ['solo', 'Solo'],
  ['bots', 'With bots'],
];

const LEVEL_TABS: [string, string][] = [['all', 'Any level'], ...Object.entries(LEVELS)];

// Where the game was on the show (aired.ts): two dropdowns side by side. Final, Exile and Hidden
// are types (the jump buttons under the title), so Day lists only the days.
const SHOW_PICKS: { key: 'season' | 'stage'; label: string; options: [string, string][] }[] = [
  {
    key: 'season',
    label: 'Season',
    options: [
      ['all', 'Any season'],
      ['1', 'Season 1'],
      ['2', 'Season 2'],
      ['3', 'Season 3'],
    ],
  },
  {
    key: 'stage',
    label: 'Day',
    options: [['all', 'Any day'], ...[1, 2, 3, 4, 5, 6].map((d): [string, string] => [`d${d}`, `Day ${d}`])],
  },
];

/** The tint class of a season (season-1 …) or a day (day-1 … day-6); none for "any". */
const tint = (key: 'season' | 'stage', value: string) =>
  value === 'all' ? '' : key === 'season' ? `season-${value}` : `day-${value.slice(1)}`;

// No "All" chip: a filter is off until a chip is picked, and picking the pressed chip again turns
// it off.
const FILTERS: { key: keyof Filter; label: string; options: [string, string][] }[] = [
  { key: 'tag', label: 'Category', options: CATEGORIES },
];

/** Where the game was on the show, on its card: the season in its colour, the day in its shade. */
function Aired({ id }: { id: string }) {
  const mark = airedOf(id);
  if (!mark) return null;
  const [season, stage = ''] = mark.split(' · ');
  const day = stageOf(id).startsWith('d') ? ` day-${stageOf(id).slice(1)}` : '';
  return (
    <span className={`aired season-${seasonOf(id)}`}>
      <span className="aired-season">{season}</span> · <span className={`aired-stage${day}`}>{stage}</span>
    </span>
  );
}

function Filters({ open, filter, onChange }: { open: boolean; filter: Filter; onChange: (f: Filter) => void }) {
  return (
    <div id="filters" className={`filters${open ? ' open' : ''}`}>
      <div className="show-picks">
        {SHOW_PICKS.map((p) => (
          <label key={p.key} className="filter">
            <span className="label">{p.label}</span>
            <span className={`pick ${tint(p.key, filter[p.key])}`}>
              <select value={filter[p.key]} onChange={(e) => onChange({ ...filter, [p.key]: e.target.value })}>
                {p.options.map(([value, label]) => (
                  <option key={value} value={value} className={tint(p.key, value)}>
                    {label}
                  </option>
                ))}
              </select>
            </span>
          </label>
        ))}
      </div>
      {FILTERS.map((f) => (
        <fieldset key={f.key} className={`filter filter-${f.key}`}>
          <legend className="label">{f.label}</legend>
          <div className="chips">
            {f.options.map(([value, label]) => (
              <button
                key={value}
                type="button"
                className="chip"
                aria-pressed={filter[f.key] === value}
                onClick={() => onChange({ ...filter, [f.key]: filter[f.key] === value ? 'all' : value })}
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
