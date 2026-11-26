// Hungarian for Symmetrical Puzzle (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Tükörkép',
  short: 'Fesd ki a hiányzó felet, hogy az ábra tükrös legyen a tengelyre, gyorsabban a többieknél.',
  strings: {
    'Round {n} of {total}': '{n}. kör / {total}',
    'Cell {n}': '{n}. mező',
    'the vertical axis': 'a függőleges tengelyre',
    'the diagonal axis': 'az átlós tengelyre',
    'both axes': 'mindkét tengelyre',
    'Make the figure symmetric about {axis}. The cells with a dot are given; add only what the mirror needs.':
      'Tedd az ábrát {axis} szimmetrikussá. A pöttyös mezők adottak; csak azt fesd ki, amit a tükörkép kíván.',
    'Colour {n}': '{n}. szín',
    'Give up this round': 'Feladom ezt a kört',
    'Keys: 1–{n} pick a colour, 0 the eraser. Tap a cell to paint it.':
      'Billentyűk: 1–{n} színt választ, 0 a radír. Koppints egy mezőre, hogy kifesd.',
  },
  // Replay moments written by the rules (crates/rules/src/games/symmetry_puzzle.rs).
  patterns: [
    ['Photo finish: (\\{[a-z0-9-]+\\}) beaten by ([\\d.]+) s', 'Fotófiniss: $2 mp-cel $1 előtt'],
    ['Clean sweep: first in all three rounds', 'Tiszta söprés: mindhárom körben első'],
  ],
};

export default hu;
