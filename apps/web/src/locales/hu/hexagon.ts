// Hungarian for Hexagon (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Hatszög',
  short: 'Jegyezz meg 19 számot, aztán vakon keress hármas vonalakat, amelyek összege a célszám.',
  strings: {
    'Memorise!': 'Jegyezd meg!',
    'Round {n} of {total}': '{n}. kör / {total}',
    'Hexagon board': 'Hatszögtábla',
    'Target:': 'Célszám:',
    Ready: 'Kész',
    Submit: 'Beküldés',
    'Three neighbouring cells in a straight line that add up to the target.':
      'Három szomszédos mező egy egyenesben, amelyek összege a célszám.',
    'Last round:': 'Előző kör:',
    'right, +{n}': 'helyes, +{n}',
    'the answer: {answer}': 'a megoldás: {answer}',
    'Keys: letters A–S pick cells, Enter submits.': 'Billentyűk: az A–S betűkkel választasz mezőt, az Enter beküldi.',
    // Replay moment written by the rules (crates/rules/src/games/hexagon.rs).
    'All five lines from memory': 'Mind az öt vonal emlékezetből',
  },
};

export default hu;
