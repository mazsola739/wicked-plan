// Hungarian for Balance Mancala (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Egyensúly-mankala',
  short: 'Kör alakú mankala háromszínű kövekkel; csak a leggyengébb színed számít.',
  strings: {
    'Mancala ring': 'Mankalagyűrű',
    'Game over': 'Vége a játéknak',
    'Your turn': 'Te lépsz',
    '{n} turns left': 'Még {n} kör',
    '{name} to move': '{name} lép',
    'You sowed {n}': '{n} követ vetettél',
    '{name} sowed {n}': '{name} {n} követ vetett',
    ', captured {n}': ', zsákmány: {n}',
    ', extra turn': ', még egy lépés',
    'Sow first:': 'Ezzel kezdd:',
    red: 'piros',
    green: 'zöld',
    blue: 'kék',
    'Points = 50 × your weakest colour in the store + all its stones.': 'Pont = 50 × a tárolód leggyengébb színe + az összes köve.',
    'Keys: 1–{n} sow from a pit, R / G / B pick the first colour.':
      'Billentyűk: 1–{n} vet egy gödörből, R / G / B választja az első színt.',
  },
  // Replay moments written by the rules (crates/rules/src/games/colour_mancala.rs).
  patterns: [['(\\d+) stones captured from (\\{[a-z0-9-]+\\})', '$1 kő zsákmányolva $2 kárára']],
};

export default hu;
