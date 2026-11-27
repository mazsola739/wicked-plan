import { type CSSProperties, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { airedOf, seasonOf, stageOf } from './aired';
import { Dropdown } from './Dropdown';
import { Emblem, Grin, Peeker } from './Emblem';
import { GameIcon } from './GameIcon';
import { GameIdContext, nameOf, useKeys } from './games/common';
import { SCREENS } from './games/registry';
import { HowToPlay, rulesHidden } from './HowTo';
import { currentLang, gameShort, huName, type Lang, setGameScope, setLang, t, useLang } from './i18n';
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
  useLang();
  const current = play.current;
  const info = play.games.find((g) => g.id === current?.gameId);
  setGameScope(info?.id ?? null);
  return (
    <div className="app">
      {play.error && <div className="banner error">{t(play.error)}</div>}
      {!play.ready ? (
        <p className="muted center">{t('Loading…')}</p>
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

/** The categories (docs/games.md, Categories), in the order the filter shows them: id, full name
    (searched, and the chip's tooltip), one-word chip label. */
const CATEGORIES: [string, string, string][] = [
  ['logic-puzzles', 'Logic & Puzzles', 'Logic'],
  ['strategy-territory', 'Strategy & Territory', 'Strategy'],
  ['bluff-deduction', 'Bluff & Deduction', 'Bluff'],
  ['social-alliances', 'Social & Alliances', 'Social'],
  ['auction-economy', 'Auction & Economy', 'Auction'],
  ['memory-observation', 'Memory & Observation', 'Memory'],
  ['speed-risk', 'Speed & Risk', 'Speed'],
];
const CATEGORY_NAME: Record<string, string> = Object.fromEntries(CATEGORIES.map(([id, name]) => [id, name]));

type Filter = { tag: string; level: string; seats: string; season: string; stage: string; query: string };
const ANY: Filter = { tag: 'all', level: 'all', seats: 'all', season: 'all', stage: 'all', query: '' };

function matches(g: GameInfo, f: Filter): boolean {
  const solo = g.maxPlayers === 1;
  const words = f.query.toLowerCase().split(/\s+/).filter(Boolean);
  const text =
    `${g.name} ${huName(g.id) ?? ''} ${gameShort(g.id, g.short)} ${g.tags.map((c) => t(CATEGORY_NAME[c] ?? '')).join(' ')}`.toLowerCase();
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
  const lang = useLang();
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
        <a className="brand" href="#top" aria-label={t('Wicked Plan, top of the page')}>
          <Grin className="brand-grin" />
          <span>Wicked Plan</span>
        </a>
        <span className="topnav">
          <a href="#how">{t('How it works')}</a>
          <a href="#games">{t('Games')}</a>
          <LangSwitch />
        </span>
      </nav>

      <header className="hero" id="top">
        <Emblem className="hero-emblem" watching />
        <div className="hero-text">
          <p className="wip">{t('Under construction')}</p>
          <p className="label tagline">{t('A digital arena of strategy')}</p>
          <h1 className="title">
            Wicked Plan
            {lang === 'hu' && <span className="title-hu">(Gonosz terv)</span>}
          </h1>
          <p className="lead">
            {t('Puzzles, bluffs and duels. Think ahead, trust carefully: in the games for several players you face bots.')}
          </p>
          <div className="hero-actions">
            <a className="btn primary" href="#games">
              {t('Enter the arena')}
            </a>
            <button type="button" className="btn violet" onClick={random}>
              {t('Random game')}
            </button>
          </div>
          <p className="muted small credit">{t("Inspired by the Netflix show The Devil's Plan. An unofficial fan project.")}</p>
        </div>
      </header>

      <Divider />

      <section className="how" id="how" aria-labelledby="how-title">
        <h2 className="section-title" id="how-title">
          {t('How it works')}
        </h2>
        <ol className="steps">
          {STEPS.map(([title, text], i) => (
            <li key={title}>
              <span className="step-no">{['I', 'II', 'III'][i]}</span>
              <strong>{t(title)}</strong>
              <span className="muted">{t(text)}</span>
            </li>
          ))}
        </ol>
      </section>

      <Divider />

      <section className="catalog" id="games" aria-labelledby="games-title">
        <h2 className="section-title" id="games-title">
          {t('Games')}
        </h2>
        {/* The count, and while anything narrows the list (search, a filter, a tab) one way back to all. */}
        <div className="games-count">
          <p aria-live="polite">
            {shown.length === games.length
              ? t('{n} games', { n: games.length })
              : t('{n} of {total} games', { n: shown.length, total: games.length })}
          </p>
          {shown.length < games.length || JSON.stringify(filter) !== JSON.stringify(ANY) ? (
            <button type="button" className="clear-all" onClick={() => setFilter(ANY)}>
              <svg viewBox="0 0 16 16" aria-hidden="true">
                <path d="M4 4 L12 12 M12 4 L4 12" />
              </svg>
              {t('Clear all')}
            </button>
          ) : null}
        </div>
        {/* Jump to a type's group of cards; a type with no game left after the filters is greyed out. */}
        <nav className="kind-nav" aria-label={t('Jump to a type of game')}>
          {KINDS.map(([kind, label]) => (
            <button
              key={kind}
              type="button"
              className={`chip kind-${kind}`}
              disabled={!shown.some((g) => g.kind === kind)}
              onClick={() => document.getElementById(`group-${kind}`)?.scrollIntoView({ behavior: 'smooth', block: 'start' })}
            >
              <UiIcon className="chip-icon" name={`kind-${kind}`} />
              <span className="chip-label">{t(kind === 'final-duel' ? 'Finals' : label.replace(/ games$/, ''))}</span>
            </button>
          ))}
        </nav>
        <div className="catalog-body">
          <aside className="catalog-side" aria-label={t('Find a game')}>
            <Search value={filter.query} onChange={(query) => setFilter({ ...filter, query })} />
            <button
              type="button"
              className="btn small ghost filters-toggle"
              aria-expanded={filtersOpen}
              aria-controls="filters"
              onClick={() => setFiltersOpen(!filtersOpen)}
            >
              {active ? `${t('Filters')} · ${active}` : t('Filters')}
            </button>
            <Filters open={filtersOpen} filter={filter} onChange={setFilter} />
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
                <legend className="sr-only">{t('Players')}</legend>
                {SEAT_TABS.map(([value, label]) => (
                  <button
                    key={value}
                    type="button"
                    className={`chip tab-chip mode-${value}`}
                    aria-pressed={filter.seats === value}
                    onClick={() => setFilter({ ...filter, seats: value })}
                  >
                    {value !== 'all' && <UiIcon className="chip-icon" name={`mode-${value}`} />}
                    <span className="chip-label">{t(label)}</span>
                  </button>
                ))}
              </fieldset>
              <fieldset className="level-tabs">
                <legend className="sr-only">{t('Difficulty')}</legend>
                {LEVEL_TABS.map(([value, label]) => (
                  <button
                    key={value}
                    type="button"
                    className={`chip tab-chip lv-${value}`}
                    aria-pressed={filter.level === value}
                    onClick={() => setFilter({ ...filter, level: value })}
                  >
                    {value !== 'all' && <UiIcon className="chip-icon" name={`level-${value}`} />}
                    <span className="chip-label">{t(label)}</span>
                  </button>
                ))}
              </fieldset>
            </div>
            {shown.length === 0 && (
              <div className="empty">
                <p className="muted">{t('No game fits these filters.')}</p>
                <button type="button" className="btn ghost small" onClick={() => setFilter(ANY)}>
                  {t('Clear filters')}
                </button>
              </div>
            )}
            {KINDS.map(([kind, label]) => {
              const list = shown.filter((g) => g.kind === kind);
              if (!list.length) return null;
              return (
                <section key={kind} id={`group-${kind}`} className="game-group">
                  <h3 className={`group-title kind-${kind}`}>{t(label)}</h3>
                  <ul className="games">
                    {list.map((g) => (
                      <li key={g.id} className={`game-card ${g.difficulty}`}>
                        <Snapshot id={g.id} />
                        <CardBadges g={g} />
                        <button type="button" className="game-btn" onClick={() => start(g.id)}>
                          <span className="game-head">
                            <strong>
                              {g.name}
                              {huName(g.id) && <span className="name-hu">({huName(g.id)})</span>}
                            </strong>
                          </span>
                          <span className="game-short">{gameShort(g.id, g.short)}</span>
                          {/* The card's foot: players on the left, where it was on the show on the right. */}
                          <span className="card-foot">
                            <span className={`players mode-${g.maxPlayers === 1 ? 'solo' : 'multi'}`}>
                              <svg viewBox="0 0 16 16" aria-hidden="true">
                                <circle cx="8" cy="5" r="3" />
                                <path d="M2.5 15 C2.5 10.5 13.5 10.5 13.5 15 Z" />
                              </svg>
                              <span className="sr-only">{t('Players')}: </span>
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
        <Grin className="footer-grin" />
        <span>{t('Wicked Plan · an unofficial fan project · all games run in your browser')}</span>
        <nav className="footer-links" aria-label={t('About the maker')}>
          <a href="https://mazsola739.github.io/mazsola739/" target="_blank" rel="noreferrer">
            {t('Portfolio')}
          </a>
          <a href="https://github.com/mazsola739/wicked-plan" target="_blank" rel="noreferrer">
            {t('Source code')}
          </a>
        </nav>
      </footer>
    </div>
  );
}

const SEAT_TABS: [string, string][] = [
  ['all', 'All games'],
  ['solo', 'Solo'],
  ['multi', 'Multiplayer'],
];

const LEVEL_TABS: [string, string][] = [['all', 'Any level'], ...Object.entries(LEVELS)];

// Where the game was on the show (aired.ts): two dropdowns side by side. Final, Exile and Hidden
// are types (the jump buttons under the title), so Day lists only the days.
const SHOW_PICKS: { key: 'season' | 'stage'; label: string; options: [string, string][] }[] = [
  {
    key: 'season',
    label: 'Season',
    options: [
      ['all', 'Any'],
      ['1', 'Season 1'],
      ['2', 'Season 2'],
      ['3', 'Season 3'],
    ],
  },
  {
    key: 'stage',
    label: 'Day',
    options: [['all', 'Any'], ...[1, 2, 3, 4, 5, 6].map((d): [string, string] => [`d${d}`, `Day ${d}`])],
  },
];

/** The tint class of a season (season-1 …) or a day (day-1 … day-6); none for "any". */
const tint = (key: 'season' | 'stage', value: string) =>
  value === 'all' ? '' : key === 'season' ? `season-${value}` : `day-${value.slice(1)}`;

// No "All" chip: a filter is off until a chip is picked, and picking the pressed chip again turns
// it off.
const FILTERS: { key: keyof Filter; label: string; options: [string, string][] }[] = [
  { key: 'tag', label: 'Category', options: CATEGORIES.map(([id, , short]): [string, string] => [id, short]) },
];

/** The owner's neon icons for the types, levels and categories (assets/ui-icons). */
const UI_ICONS = import.meta.glob<string>('./assets/ui-icons/*.webp', { eager: true, import: 'default' });
const uiIcon = (name: string) => UI_ICONS[`./assets/ui-icons/${name}.webp`];

/** A UI icon drawn in its button's colour: the art is the mask, the colour comes from CSS. */
function UiIcon({ name, className }: { name: string; className?: string }) {
  return <span className={className} aria-hidden="true" style={{ '--icon': `url(${uiIcon(name)})` } as CSSProperties} />;
}

/**
 * A card's badges, on the line between its picture and its text, at the left: hexagons like icon
 * roundels, each in its own colour with its neon icon: solo or multiplayer, the difficulty, the type,
 * then the categories.
 */
function CardBadges({ g }: { g: GameInfo }) {
  const kind = KINDS.find(([k]) => k === g.kind)?.[1] ?? '';
  const mode = g.maxPlayers === 1 ? 'solo' : 'multi';
  const badges: [string, string, string][] = [
    [`mode-${mode}`, `mode-${mode}`, t(mode === 'solo' ? 'Solo' : 'Multiplayer')],
    [`lv`, `level-${g.difficulty}`, t(LEVELS[g.difficulty])],
    [`kind-${g.kind}`, `kind-${g.kind}`, t(kind)],
    ...g.tags.map((c): [string, string, string] => [`cat-${c}`, `category-${c}`, t(CATEGORY_NAME[c] ?? '')]),
  ];
  return (
    <span className="card-badges">
      {badges.map(([tint, icon, name]) => (
        <span key={icon} className={`hex-glow ${tint}`} role="img" aria-label={name} title={name}>
          <span className="hex-badge">
            <UiIcon className="badge-icon" name={icon} />
          </span>
        </span>
      ))}
    </span>
  );
}

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
        {SHOW_PICKS.map((p, i) => (
          <Dropdown
            key={p.key}
            label={t(p.label)}
            value={filter[p.key]}
            align={i === 0 ? 'left' : 'right'}
            options={p.options.map(([value, label]) => ({ value, label: t(label), tint: tint(p.key, value) }))}
            onChange={(value) => onChange({ ...filter, [p.key]: value })}
          />
        ))}
      </div>
      {FILTERS.map((f) => (
        <fieldset key={f.key} className={`filter filter-${f.key}`}>
          <legend className="label">{t(f.label)}</legend>
          <div className="chips">
            {f.options.map(([value, label]) => (
              <button
                key={value}
                type="button"
                className={`chip cat-${value}`}
                title={f.key === 'tag' ? t(CATEGORY_NAME[value] ?? '') : undefined}
                aria-pressed={filter[f.key] === value}
                onClick={() => onChange({ ...filter, [f.key]: filter[f.key] === value ? 'all' : value })}
              >
                {f.key === 'tag' && <UiIcon className="chip-icon" name={`category-${value}`} />}
                <span className="chip-label">{t(label)}</span>
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
        placeholder={t('Search games')}
        aria-label={t('Search games')}
        onChange={(e) => onChange(e.target.value)}
      />
      <kbd className="key-hint search-key">/</kbd>
    </label>
  );
}

/** EN / HU in the top bar. */
function LangSwitch() {
  const lang = useLang();
  const pick = (l: Lang) => (
    <button type="button" className="lang-btn" aria-pressed={lang === l} onClick={() => setLang(l)}>
      {l.toUpperCase()}
    </button>
  );
  return (
    <fieldset className="lang-switch">
      <legend className="sr-only">{t('Language')}</legend>
      {pick('en')}
      {pick('hu')}
    </fieldset>
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
          ← {t('Games')}
        </button>
        {!rules && (
          <button ref={howTo} type="button" className="btn small ghost" onClick={() => setRules(true)}>
            {t('How to play')}
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
        !results && <p className="muted center">{t('Loading…')}</p>
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
      <div className="score big">{t('{n} points', { n: mine?.points ?? 0 })}</div>
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
          {t('Back to games')}
        </button>
        <button type="button" className="btn primary" onClick={() => start(info.id)}>
          {t('Play again')}
        </button>
      </div>
    </section>
  );
}

function heading(results: PlayerResult[], mine: PlayerResult | undefined): string {
  if (results.length === 1) return t('Finished');
  // Team games give every seat the same rank and points.
  if (results.every((r) => r.rank === 1 && r.points === results[0]?.points)) return t('Team result');
  if (mine?.rank === 1) return t('You won!');
  return currentLang() === 'hu' ? `${mine?.rank ?? 0}. lettél` : `You came ${ordinal(mine?.rank ?? 0)}`;
}

function ordinal(n: number): string {
  const s = ['th', 'st', 'nd', 'rd'];
  const v = n % 100;
  return `${n}${s[(v - 20) % 10] ?? s[v] ?? s[0]}`;
}
