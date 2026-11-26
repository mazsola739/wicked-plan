// Hungarian for Spot the Difference (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Keresd a különbséget',
  short: 'Két kép, egy maréknyi különbség: koppints rá mindre, mielőtt a riválisok tennék.',
  strings: {
    'Round {n} of {total}': '{n}. kör / {total}',
    'Left picture': 'Bal oldali kép',
    'Right picture': 'Jobb oldali kép',
    found: 'megvan',
    'missed {n}×': '{n}× mellé',
    'Next pictures': 'Következő képek',
    Finish: 'Befejezés',
    '{n} of {total}': '{n} / {total}',
    'Tap a difference in either picture: +30 each, −10 for a tap on nothing. Finding all first: +100.':
      'Koppints egy különbségre bármelyik képen: darabja +30, a semmibe koppintás −10. Aki elsőként megtalálja mindet: +100.',
    'Keys: Enter or N goes on after the reveal.': 'Billentyűk: a felfedés után az Enter vagy az N továbblép.',
  },
  // Replay moments written by the rules (crates/rules/src/games/find_the_differences.rs).
  patterns: [
    ['All (\\d+) found without a miss', '$1/$1 megvan, egyetlen tévedés nélkül'],
    ['All (\\d+) found with (\\d+) s left', '$1/$1 megvan, $2 mp-cel a vége előtt'],
  ],
};

export default hu;
