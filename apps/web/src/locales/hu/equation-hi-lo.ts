// Hungarian for Equation High-Low (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const SEAT = '([a-z0-9-]+)';

const hu: Locale = {
  name: 'Magas-alacsony egyenlet',
  short: 'Építs egyenletet négy számkártyából és három jelből, aztán fogadj arra, hogy az 1-hez vagy a 20-hoz kerülsz legközelebb.',
  strings: {
    invalid: 'érvénytelen',
    'Tap a card': 'Koppints egy kártyára',
    'Tap a symbol': 'Koppints egy jelre',
    'aim for {n}': 'cél: {n}',
    'Low · 1': 'Alacsony · 1',
    'High · 20': 'Magas · 20',
    low: 'alacsony',
    high: 'magas',
    '⌫ Undo': '⌫ Vissza',
    Submit: 'Beküldés',
    'betting 1': '1. tétkör',
    build: 'építés',
    'betting 2': '2. tétkör',
    'Game over': 'Vége a játéknak',
    'Hand {n} of {total}': '{n}. leosztás / {total}',
    folded: 'kiszállt',
    'in {n}': 'bent: {n}',
    'built ✓': 'kész ✓',
    out: 'kint',
    Pot: 'Kassza',
    hidden: 'rejtett',
    'face up': 'nyílt',
    'Your equation:': 'Az egyenleted:',
    'Call {n}': 'Megadás: {n}',
    Check: 'Passz',
    'Raise 2': 'Emelés 2',
    Fold: 'Kiszállás',
    'Waiting for the others to build…': 'Várakozás, amíg a többiek megépítik az egyenletüket…',
    '{name} is betting…': '{name} tesz…',
    'Hand {n}: pot {pot}': '{n}. leosztás: kassza {pot}',
    'Keys: C call, R raise, F fold · build: 1–4 cards, + − * / symbols, L / H side, Enter.':
      'Billentyűk: C megadás, R emelés, F kiszállás · építés: 1–4 kártyák, + − * / jelek, L / H oldal, Enter.',
    'Game over.': 'Vége a játéknak.',
  },
  // Log lines and replay moments written by the rules. Seat ids stay (the screen shows names).
  patterns: [
    ['Hand (\\d+): ante (\\d+) each\\.', '$1. leosztás: mindenki $2 zsetont tesz be.'],
    [`${SEAT} checks\\.`, '$1: passz.'],
    [`${SEAT} calls (\\d+)\\.`, '$1: megadás, $2.'],
    [`${SEAT} raises to (\\d+)\\.`, '$1: emelés, a tét most $2.'],
    [`${SEAT} folds\\.`, '$1: kiszállás.'],
    [`${SEAT} takes the pot of (\\d+)\\.`, 'A kassza ($2) → $1.'],
    ['Showdown: pot (\\d+) shared\\.', 'Lapfelfedés: a kassza ($1) szétosztva.'],
    ['Pot of (\\d+) taken without a showdown', 'Kassza ($1) lapfelfedés nélkül'],
    ['Exactly (\\d+) sweeps the pot of (\\d+): (.+)', 'Pontosan $1, a teljes kassza ($2): $3'],
    ['Exactly (\\d+) takes (\\d+) chips: (.+)', 'Pontosan $1, $2 zseton: $3'],
  ],
};

export default hu;
