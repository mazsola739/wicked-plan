// Hungarian for Nine Men's Morris (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Malom',
  short: 'A malomjáték: tedd le, csúsztasd és röptesd kilenc bábudat; három egy vonalban levesz egy ellenséges bábut.',
  strings: {
    'Place a piece on an empty point.': 'Tegyél egy bábut egy üres pontra.',
    'Pick a piece, then a neighbouring empty point.': 'Válassz egy bábut, aztán egy szomszédos üres pontot.',
    'Three pieces left: a piece may fly to any empty point.': 'Három bábud maradt: bármelyik üres pontra repülhetsz.',
    'Mill! Remove one of the glowing pieces.': 'Malom! Vegyél le egyet a világító bábuk közül.',
    'Game over.': 'Vége a játéknak.',
    '{name} is thinking…': '{name} gondolkodik…',
    'Turn {n}': '{n}. kör',
    'your move': 'te lépsz',
    'Morris board': 'Malomtábla',
    '{n} on board': '{n} a táblán',
    'Click a piece, then where it goes. Esc cancels.': 'Kattints egy bábura, aztán oda, ahová lép. Az Esc visszavon.',
  },
  // Replay moments written by the rules (crates/rules/src/games/nine_mens_morris.rs).
  patterns: [
    ['Mill closed: (\\{[a-z0-9-]+\\}) is down to 2 pieces', 'Malom: $1 2 bábura fogyott'],
    ['(\\{[a-z0-9-]+\\}) blocked in, not a move left', '$1 beszorult, nincs több lépése'],
    ['Two mills closed with one piece', 'Két malom egyetlen bábuval'],
  ],
};

export default hu;
