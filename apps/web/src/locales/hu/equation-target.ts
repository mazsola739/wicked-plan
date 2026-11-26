// Hungarian for Equation Pyramid (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Egyenletpiramis',
  short: 'Válassz tíz számkártyából hármat és két műveleti jelet, hogy kijöjjön a célszám.',
  strings: {
    'Round {n} of {total}': '{n}. kör / {total}',
    'Last round:': 'Előző kör:',
    'right, +{n}': 'helyes, +{n}',
    '{given} – the answer: {answer}': '{given} – a megoldás: {answer}',
    'no answer': 'nincs válasz',
    'Target:': 'Célszám:',
    'Tap three cards; tap an operator to change it.': 'Koppints három kártyára; a műveleti jelre koppintva váltod.',
    Submit: 'Beküldés',
    'Keys: 1–9, 0 pick the cards in order, + − * set the operators, Enter submits.':
      'Billentyűk: az 1–9 és a 0 sorban választja a kártyákat, a + − * a műveleti jeleket, az Enter beküldi.',
    // Replay moment written by the rules (crates/rules/src/games/equation_target.rs).
    'All five targets hit': 'Mind az öt célszámot eltaláltad',
  },
};

export default hu;
