// Hungarian for Loop Fence Puzzle (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Körbekerítés',
  short: 'Építs egyetlen zárt kerítést; minden szám megmondja, a parcellája hány oldala van bekerítve. Verseny.',
  strings: {
    'Round {n} of {total}': '{n}. kör / {total}',
    Field: 'Mező',
    'Edge {n}': '{n}. oldal',
    'Build one closed fence along the dotted lines. A number tells how many sides of its plot are fenced. Tap near a side: fence, then cross (a note), then empty.':
      'Építs egyetlen zárt kerítést a pontozott vonalak mentén. A szám megmondja, a parcellája hány oldala van bekerítve. Koppints egy oldal közelébe: kerítés, aztán kereszt (jegyzet), aztán üres.',
    'Give up this round': 'Feladom ezt a kört',
    'Clear all': 'Mindent töröl',
    'Click near a side of a plot to fence it; click again for a cross.':
      'Kattints egy parcella oldala közelébe, hogy bekerítsd; kattints újra egy keresztért.',
  },
  // Replay moments written by the rules (crates/rules/src/games/fence_building.rs).
  patterns: [
    ['Fence closed ([\\d.]+) s ahead of (\\{[a-z0-9-]+\\})', 'Kerítés lezárva $1 mp-cel $2 előtt'],
    ['Fence closed (\\d+) s ahead of the field', 'Kerítés lezárva $1 mp-cel a mezőny előtt'],
    ['First in all three rounds', 'Mindhárom körben első'],
  ],
};

export default hu;
