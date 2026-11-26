// Hungarian for Wall Go (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Fal-gó',
  short: 'Lépj egy bábuval, és építs falat: keríts el magadnak minél nagyobb részt a 7×7-es táblából.',
  strings: {
    'Tap one of your pieces.': 'Koppints az egyik bábudra.',
    'Tap where it goes (or keep it), then pick a side for the wall.':
      'Koppints oda, ahová lép (vagy maradjon), aztán válaszd ki a fal oldalát.',
    'Game over.': 'Vége a játéknak.',
    '{name} is building…': '{name} épít…',
    'Turn {n}': '{n}. kör',
    'your move': 'te lépsz',
    'sealed in': 'bezárva',
    'Wall side': 'A fal oldala',
    Top: 'Fent',
    Left: 'Bal',
    Right: 'Jobb',
    Bottom: 'Lent',
    'Arrow keys build the wall on that side; Esc cancels.': 'A nyilakkal arra az oldalra építesz falat; az Esc visszavon.',
  },
  // Replay moments written by the rules (crates/rules/src/games/wall_territory.rs).
  patterns: [
    ['Wall seals off (\\d+) squares', 'Egy fal $1 mezőt zár le'],
    ['(\\{[a-z0-9-]+\\}) shut in with only 1 square', '$1 mindössze 1 mezővel zárult be'],
    ['(\\{[a-z0-9-]+\\}) shut in with only (\\d+) squares', '$1 mindössze $2 mezővel zárult be'],
  ],
};

export default hu;
