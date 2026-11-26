// Hungarian for Count the Cubes (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Számold meg a kockákat',
  short: 'Hány kocka van a rajzon? Öt kör, mindegyik 20 másodperc.',
  strings: {
    'Round {n} of {total}': '{n}. kör / {total}',
    'Last round:': 'Előző kör:',
    'right, +{n}': 'helyes, +{n}',
    '{given} – the answer: {answer}': '{given} – a megoldás: {answer}',
    'no answer': 'nincs válasz',
    Cubes: 'Kockák',
    'How many cubes are there?': 'Hány kocka van?',
    // Replay moment written by the rules (crates/rules/src/games/count_the_cubes.rs).
    'All five stacks counted right': 'Mind az öt halmot jól számoltad meg',
  },
  patterns: [['(\\d+) cubes counted with (\\d+) s to spare', '$1 kocka megszámolva, $2 mp tartalékkal']],
};

export default hu;
