// Hungarian for Secret Number (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// The clues (crates/rules/src/games/secret_number.rs, Clue::text): [English pattern, groups in it,
// Hungarian with the group written as {g}]. A trade note holds two clues of the same kind.
const CLUES: [string, number, string][] = [
  ['even', 0, 'páros'],
  ['odd', 0, 'páratlan'],
  ['higher than (\\d+)', 1, 'nagyobb, mint {g}'],
  ['(\\d+) or lower', 1, 'legfeljebb {g}'],
  ['tens digit (\\d)', 1, 'tízes helyiérték: {g}'],
  ['last digit (\\d)', 1, 'utolsó számjegy: {g}'],
  ['divisible by 3', 0, 'osztható 3-mal'],
  ['not divisible by 3', 0, 'nem osztható 3-mal'],
];
const KINDS = [
  [0, 1],
  [2, 3],
  [4, 4],
  [5, 5],
  [6, 7],
];

/** "{name}'s number: <clue>. {name} learned yours: <clue>." for every pair of clues of one kind. */
const notes = KINDS.flatMap((kind) =>
  kind.flatMap((a) =>
    kind.map((b): [string, string] => {
      const [ea, na, ha] = CLUES[a] ?? ['', 0, ''];
      const [eb, , hb] = CLUES[b] ?? ['', 0, ''];
      return [
        `(\\S+)'s number: ${ea}\\. \\S+ learned yours: ${eb}\\.`,
        `$1 száma: ${ha.replace('{g}', '$2')}. $1 ezt tudta meg a tiédről: ${hb.replace('{g}', `$${2 + na}`)}.`,
      ];
    }),
  ),
);

const hu: Locale = {
  name: 'Titkos szám',
  short: 'Mindenkinek van egy titkos száma 1 és 100 között; cserélj nyomokat, tippeld meg a többiekét, és védd a sajátodat.',
  strings: {
    // The screen.
    'Game over: all numbers are shown.': 'Vége a játéknak: minden szám látható.',
    'Waiting for {names}…': 'Várakozás: {names}…',
    'Trade clues: tap a player (they learn about you too), or skip.':
      'Nyomcsere: koppints egy játékosra (ő is megtud rólad valamit), vagy hagyd ki.',
    "Guess {name}'s number.": 'Tippeld meg {name} számát.',
    'Guess: tap a player to guess their number, or pass.': 'Tipp: koppints egy játékosra, és tippeld meg a számát, vagy passzolj.',
    'Game over': 'Vége a játéknak',
    'Round {n} of {total}': '{n}. kör / {total}',
    trade: 'csere',
    guess: 'tipp',
    'Your secret number': 'A titkos számod',
    Exposed: 'Lelepleződött',
    '{n} possible ({low}–{high})': '{n} lehetséges ({low}–{high})',
    'Skip the trade': 'Kihagyom a cserét',
    Pass: 'Passz',
    'What you learned': 'Amit megtudtál',
    'Keys: A–H pick a player, S skips, P passes, digits + Enter guess.':
      'Billentyűk: A–H játékost választ, S kihagy, P passzol, számjegyek + Enter tippel.',

    // The log (the rules).
    'Round 1 begins.': 'Kezdődik az 1. kör.',
    'Round 2 begins.': 'Kezdődik a 2. kör.',
    'Round 3 begins.': 'Kezdődik a 3. kör.',
    'Round 4 begins.': 'Kezdődik a 4. kör.',
    'Round 5 begins.': 'Kezdődik az 5. kör.',
    'Game over: every hidden number scores +40.': 'Vége a játéknak: minden rejtve maradt szám +40 pontot ér.',
  },
  patterns: [
    ...CLUES.map(([en, , hu]): [string, string] => [en, hu.replace('{g}', '$1')]),
    ...notes,
    ["(\\S+)'s number is higher than (\\d+)\\.", '$1 száma nagyobb, mint $2.'],
    ["(\\S+)'s number is lower than (\\d+)\\.", '$1 száma kisebb, mint $2.'],
    ['(\\S+) traded clues with (\\S+)\\.', 'Nyomcsere: $1 és $2.'],
    ["(\\S+) guessed (\\S+)'s number: right, (\\d+)!", 'Tipp: $1 → $2 száma. Talált, $3!'],
    ["(\\S+) guessed (\\S+)'s number: wrong\\.", 'Tipp: $1 → $2 száma. Nem talált.'],
    // Replay moments.
    ["(\\{[a-z0-9-]+\\})'s number (\\d+) cracked in round (\\d)", '$1 száma ($2) feltörve, $3. kör'],
    ["Long shot: (\\{[a-z0-9-]+\\})'s (\\d+) out of (\\d+) possible", 'Telitalálat: $1 száma, $2, $3 lehetséges közül'],
  ],
};

export default hu;
