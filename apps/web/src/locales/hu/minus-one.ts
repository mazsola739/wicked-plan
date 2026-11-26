// Hungarian for Remove One (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Mínusz egy',
  short: 'Mutass két kártyát, tarts meg egyet; a legkisebb egyedi kártya nyeri a kört.',
  strings: {
    'Waiting for the others to show…': 'Várakozás, hogy a többiek is mutassanak…',
    'Pick two cards to show.': 'Válassz két kártyát, amit megmutatsz.',
    'You keep {card}. Waiting…': 'A(z) {card} marad nálad. Várakozás…',
    'Keep one of your two cards.': 'Tartsd meg a két kártyád egyikét.',
    'You win {pot} with the {card}.': 'A(z) {card} kártyával {pot} pontot nyersz.',
    '{name} wins {pot} with the {card}.': '{name} a(z) {card} kártyával {pot} pontot nyer.',
    'No unique card: nobody scores.': 'Nincs egyedi kártya: senki sem kap pontot.',
    'Round {n} of {total}': '{n}. kör / {total}',
    ready: 'kész',
    'thinking…': 'gondolkodik…',
    'Round {n}: kept {cards} → {name} +{pot}': '{n}. kör: megtartva {cards} → {name} +{pot}',
    'Round {n}: kept {cards} → no winner': '{n}. kör: megtartva {cards} → nincs nyertes',
    keep: 'megtart',
    'Show {a} and {b}': '{a} és {b} megmutatása',
    'Show two cards': 'Mutass két kártyát',
    'Keys 1–9 and 0 (for 10) pick cards, Enter shows.': 'Az 1–9 és a 0 (a 10-hez) gomb választ kártyát, az Enter megmutatja.',
  },
  // Replay moments written by the rules (crates/rules/src/games/minus_one.rs).
  patterns: [['A lone (\\d+) takes the pot of (\\d+)', 'Egy magányos $1 viszi a $2 pontos kasszát']],
};

export default hu;
