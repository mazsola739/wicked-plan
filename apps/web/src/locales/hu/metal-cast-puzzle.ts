// Hungarian for Metal Cast Puzzle (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Fémgyűrű-rejtvény',
  short: 'Csúsztasd le az összes gyűrűt a fémhurokról; egy gyűrű csak akkor mozdul, ha az alatta lévő engedi.',
  strings: {
    'Level {n} of {total} · {rings} rings': '{n}. szint / {total} · {rings} gyűrű',
    'Rings on a metal loop': 'Gyűrűk egy fémhurkon',
    'Ring {n} on': '{n}. gyűrű fent',
    'Ring {n} off': '{n}. gyűrű lent',
    'Ring 1 always slides. Any other ring slides only when the ring just below it is':
      'Az 1. gyűrű mindig mozdul. Bármely másik gyűrű csak akkor, ha a közvetlenül alatta lévő',
    on: 'fent',
    'and every ring below that one is': 'van, a többi alatta lévő pedig',
    off: 'lent',
    'Take every ring off.': 'Vedd le az összes gyűrűt.',
    '{n} ring on': '{n} gyűrű fent',
    '{n} rings on': '{n} gyűrű fent',
    '{n} move': '{n} lépés',
    '{n} moves': '{n} lépés',
    'best possible {n}': 'a legjobb lehetséges {n}',
    'Give up': 'Feladom',
    'Reset level': 'Szint újrakezdése',
    'Keys: 1–{n} slide that ring, Backspace resets.': 'Billentyűk: az 1–{n} azt a gyűrűt mozdítja, a Backspace újrakezd.',
  },
  // Replay moments written by the rules (crates/rules/src/games/metal_cast_puzzle.rs).
  patterns: [['(\\d+) rings off in (\\d+) moves, the fewest possible', '$1 gyűrű lent $2 lépésből, a lehető legkevesebből']],
};

export default hu;
