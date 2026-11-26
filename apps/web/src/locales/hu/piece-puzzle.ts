// Hungarian for Piece Puzzle (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Kirakós',
  short: 'Töltsd ki pontosan az alakzatot a megadott darabokkal, gyorsabban a többieknél. Három kör.',
  strings: {
    'Round {n} of {total}': '{n}. kör / {total}',
    'Target shape': 'A kirakandó alakzat',
    'Piece {n}': '{n}. darab',
    'Tap the shape where the dotted cell should go.': 'Koppints az alakzatra oda, ahová a pöttyös mező kerüljön.',
    'Pick a piece, turn or flip it, then tap the shape. Tap a placed piece to pick it up.':
      'Válassz egy darabot, forgasd vagy tükrözd, aztán koppints az alakzatra. A lerakott darabra koppintva felveszed.',
    '⟳ Turn': '⟳ Forgatás',
    '⇋ Flip': '⇋ Tükrözés',
    'Give up this round': 'Feladom ezt a kört',
    'Keys: 1–{n} pick a piece, R turns, F flips, Esc drops it.': 'Billentyűk: 1–{n} darabot választ, R forgat, F tükröz, Esc elengedi.',
  },
  // Replay moments written by the rules (crates/rules/src/games/piece_puzzle.rs).
  patterns: [
    ['Beat (\\{[a-z0-9-]+\\}) to the finish by ([\\d.]+) s', 'Célba érés $2 mp-cel $1 előtt'],
    ['First to finish in all three rounds', 'Mindhárom körben első a célban'],
  ],
};

export default hu;
