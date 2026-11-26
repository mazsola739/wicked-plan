// Hungarian for Count the Rectangles (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Számold meg a téglalapokat',
  short: 'Hány téglalapot rajzolnak ki a vonalak? Öt kör, mindegyik 30 másodperc.',
  strings: {
    'Round {n} of {total}': '{n}. kör / {total}',
    'Last round:': 'Előző kör:',
    'right, +{n}': 'helyes, +{n}',
    '{given} – the answer: {answer}': '{given} – a megoldás: {answer}',
    'no answer': 'nincs válasz',
    Figure: 'Ábra',
    'How many rectangles are there? (Squares count too.)': 'Hány téglalap van? (A négyzetek is számítanak.)',
    // Replay moment written by the rules (crates/rules/src/games/count_the_rectangles.rs).
    'All five figures counted right': 'Mind az öt ábrát jól számoltad meg',
  },
  patterns: [['(\\d+) rectangles counted with (\\d+) s to spare', '$1 téglalap megszámolva, $2 mp tartalékkal']],
};

export default hu;
