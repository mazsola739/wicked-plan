// English / Hungarian (docs/design.md, Language). The English text is the key: t('Search games')
// gives the Hungarian when the page is in Hungarian and a translation exists, else the English.
// Translations live in locales/hu/: common.ts for the page around the games, and <game-id>.ts for
// each game (its Hungarian name and short text, its screen's strings, and patterns for the lines
// the rules write, such as logs). A game's Hungarian How to play is locales/hu/howto/<game-id>.md.

import { useSyncExternalStore } from 'react';

export type Lang = 'en' | 'hu';

/** One locale file: plain strings, patterns for built lines, and (for a game) its name and short. */
export type Locale = {
  name?: string;
  short?: string;
  strings?: Record<string, string>;
  /** [English pattern, Hungarian replacement]: the whole line must match; $1, $2 … are the groups. */
  patterns?: [string, string][];
};

const FILES = import.meta.glob<Locale>('./locales/hu/*.ts', { eager: true, import: 'default' });
const HOWTO = import.meta.glob<string>('./locales/hu/howto/*.md', { query: '?raw', import: 'default', eager: true });

// Each file keeps its own strings and patterns: the game being played is asked first, so the same
// English word can read differently in two games; then the page's common words. Only on the picker
// (the cards' still previews, where no game is open) are all games' words tried.
type Table = { strings: Record<string, string>; patterns: [RegExp, string][] };
const table = (l: Locale): Table => ({
  strings: l.strings ?? {},
  patterns: (l.patterns ?? []).map(([en, hu]): [RegExp, string] => [new RegExp(`^(?:${en})$`), hu]),
});
const GAMES: Record<string, Locale> = {};
const TABLES: Record<string, Table> = {};
for (const [path, locale] of Object.entries(FILES)) {
  const id = path.slice(path.lastIndexOf('/') + 1, -3);
  TABLES[id] = table(locale);
  if (id !== 'common') GAMES[id] = locale;
}
const ORDER = Object.keys(TABLES).filter((id) => id !== 'common');

let scope: string | null = null;
/** The game being played (its translations come first); null on the picker. */
export function setGameScope(id: string | null) {
  scope = id;
}

const KEY = 'wp-lang';

function initial(): Lang {
  try {
    const saved = localStorage.getItem(KEY);
    if (saved === 'en' || saved === 'hu') return saved;
  } catch {}
  return navigator.language?.toLowerCase().startsWith('hu') ? 'hu' : 'en';
}

let lang: Lang = initial();
document.documentElement.lang = lang;
const listeners = new Set<() => void>();

export function setLang(next: Lang) {
  lang = next;
  try {
    localStorage.setItem(KEY, next);
  } catch {}
  document.documentElement.lang = next;
  for (const f of listeners) f();
}

/** The page's language now, for code outside a component's render. */
export const currentLang = (): Lang => lang;

/** The page's language; the component re-renders when it changes. */
export function useLang(): Lang {
  return useSyncExternalStore(
    (f) => {
      listeners.add(f);
      return () => listeners.delete(f);
    },
    () => lang,
  );
}

/** Fills {name} slots from vars. */
function fill(text: string, vars?: Record<string, string | number>): string {
  return vars ? text.replace(/\{(\w+)\}/g, (all, k: string) => (k in vars ? String(vars[k]) : all)) : text;
}

/** The text in the page's language: an exact translation, else a matching pattern, else the English. */
export function t(en: string, vars?: Record<string, string | number>): string {
  if (lang === 'en') return fill(en, vars);
  const order = scope ? [scope, 'common'] : ['common', ...ORDER];
  for (const id of order) {
    const hu = id ? TABLES[id]?.strings[en] : undefined;
    if (hu !== undefined) return fill(hu, vars);
  }
  for (const id of order) {
    for (const [re, rep] of (id && TABLES[id]?.patterns) || []) if (re.test(en)) return en.replace(re, rep);
  }
  return fill(en, vars);
}

/** A game's Hungarian name, shown under the English one in Hungarian; null in English. */
export function huName(id: string): string | null {
  return lang === 'hu' ? (GAMES[id]?.name ?? null) : null;
}

/** A game's short text in the page's language. */
export function gameShort(id: string, en: string): string {
  return (lang === 'hu' && GAMES[id]?.short) || en;
}

/** A game's How to play page in Hungarian, if it has one and the page is in Hungarian. */
export function howToHu(id: string): string | null {
  return lang === 'hu' ? (HOWTO[`./locales/hu/howto/${id}.md`] ?? null) : null;
}
