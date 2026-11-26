// Hungarian for Doubt and Bet (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// COLOURS in crates/rules/src/games/colour_count_bluff.rs.
const COLOURS: [string, string][] = [
  ['red', 'piros'],
  ['blue', 'kék'],
  ['green', 'zöld'],
  ['yellow', 'sárga'],
];
const SEAT = '([a-z0-9-]+)';
const SLOT = '(\\{[a-z0-9-]+\\})';

const hu: Locale = {
  name: 'Kétség és licit',
  short: 'Licitálj arra, hány kártya van egy színből az asztalon, miközben csak a sajátjaidat látod; emelj, vagy leplezd le a blöfföt.',
  strings: {
    ...Object.fromEntries(COLOURS),
    life: 'élet',
    'Game over.': 'Vége a játéknak.',
    'Your turn: raise the bet or call it.': 'Te jössz: emeld a licitet, vagy kételkedj benne.',
    'You open: make the first bet.': 'Te kezdesz: tedd meg az első licitet.',
    '{name} is thinking…': '{name} gondolkodik…',
    'Round {n} of {total}': '{n}. kör / {total}',
    '{n} lives': '{n} élet',
    '{n} cards': '{n} kártya',
    '{name} bets at least': '{name} licitje: legalább',
    'No bet yet this round': 'Ebben a körben még nincs licit',
    '{n} cards on the table': '{n} kártya az asztalon',
    'Your cards': 'A kártyáid',
    'Bet {n} {colour}': 'Licit: {n} {colour}',
    Call: 'Kétlem',
    'Round {n}: {count} {colour} bet, {actual} there': '{n}. kör: a licit {count} {colour}, valójában {actual}',
    'Keys: R/B/G/Y colour, + / − count, Enter bets, C calls.':
      'Billentyűk: R/B/G/Y szín (piros/kék/zöld/sárga), + / − darabszám, Enter licitál, C kétkedik.',
  },
  // Log lines and replay moments written by the rules. Seat ids stay (the screen shows names).
  patterns: [
    ['Round (\\d+): ([a-z0-9-]+) starts\\.', '$1. kör, kezdő: $2.'],
    ...COLOURS.flatMap(([en, hu]): [string, string][] => [
      [`${SEAT} bets (\\d+) ${en}\\.`, `$1 – licit: $2 ${hu}.`],
      [
        `${SEAT} calls: (\\d+) ${en} on the table\\. ${SEAT} loses a life\\.`,
        `Kétkedés ($1): $2 ${hu} van az asztalon. Életet veszít: $3.`,
      ],
      [`Exactly (\\d+) ${en}: ${SLOT}'s call fails`, `Pontosan $1 ${hu}: hibás kétkedés ($2)`],
      [`${SLOT}'s (\\d+) ${en} caught: only (\\d+) there`, `Lebukott blöff ($1): $2 ${hu}, de csak $3 volt`],
    ]),
  ],
};

export default hu;
