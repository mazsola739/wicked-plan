// Hungarian for Time Auction (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Időárverés',
  short: 'Tartsd nyomva a gombot, és licitálj időt a véges idődből; a leghosszabb nyomás nyeri a kört.',
  strings: {
    'Your bank is empty: you sit this one out.': 'Elfogyott az időd: ezt a kört kihagyod.',
    'Keep holding…': 'Tartsd nyomva…',
    'Press and hold the button.': 'Nyomd meg, és tartsd nyomva a gombot.',
    'Starting in {n}… let go now to pass.': 'Indul: {n}… ha most elengeded, passzolsz.',
    'You pass this round.': 'Ebben a körben passzolsz.',
    'Bidding: {time}': 'Licit: {time}',
    'You let go at {time}. Waiting…': '{time} után engedted el. Várakozás…',
    'You sit this one out. Waiting…': 'Ezt a kört kihagyod. Várakozás…',
    'You win the token.': 'Te nyered a zsetont.',
    '{name} wins the token.': '{name} nyeri a zsetont.',
    'Nobody wins the token.': 'Senki sem nyeri a zsetont.',
    'Final round': 'Utolsó kör',
    'Round {n} of {total}': '{n}. kör / {total}',
    'Clock running': 'Megy az óra',
    'out of time': 'elfogyott az ideje',
    holding: 'nyomja',
    'not yet': 'még nem',
    tokens: 'zsetonok',
    'no tokens': 'nincs zsetonja',
    'Round {n}: You won ({b} bid)': '{n}. kör: te nyertél ({b} licitált)',
    'Round {n}: {name} won ({b} bid)': '{n}. kör: {name} nyert ({b} licitált)',
    'Round {n}: no winner ({b} bid)': '{n}. kör: nincs nyertes ({b} licitált)',
    'Your bank': 'Az időd',
    'You held {time}.': '{time} ideig tartottad.',
    HOLDING: 'NYOMOD',
    HOLD: 'NYOMD',
    'Or hold Space.': 'Vagy tartsd nyomva a szóközt.',
  },
  // Replay moments written by the rules (crates/rules/src/games/time_auction.rs).
  patterns: [
    ['Third token in a row', 'Harmadik zseton zsinórban'],
    ['Last token of the game takes the sole lead', 'A játék utolsó zsetonja egyedüli vezetést hoz'],
    ['Sole lead taken in round (\\d+) of (\\d+)', 'Egyedüli vezetés: $1. kör / $2'],
    ['Token won with a hold of just ([\\d.]+) s', 'Zseton mindössze $1 s nyomással'],
  ],
};

export default hu;
