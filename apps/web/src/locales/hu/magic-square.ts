// Hungarian for Divine Test (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Isteni próba',
  short: 'Töltsd ki a rácsot úgy, hogy minden sor, oszlop és átló összege egyenlő legyen. 3×3, 4×4, 5×5.',
  strings: {
    'Board {n} of {total}': '{n}. tábla / {total}',
    'Use 1–{max} once each; every row, column and diagonal adds up to':
      'Az 1–{max} számokat egyszer-egyszer használd; minden sor, oszlop és átló összege',
    'Not right yet ({n}×).': 'Még nem jó ({n}×).',
    'Give up': 'Feladom',
    Check: 'Ellenőrzés',
    'Keys: type a number, arrows move, Backspace clears, Enter checks.':
      'Billentyűk: írj be egy számot, a nyilakkal lépsz, a Backspace töröl, az Enter ellenőriz.',
  },
  // Replay moment written by the rules (crates/rules/src/games/magic_square.rs).
  patterns: [['5×5 square solved, (\\d+:\\d\\d) still on the clock', '5×5-ös négyzet megoldva, még $1 volt hátra']],
};

export default hu;
