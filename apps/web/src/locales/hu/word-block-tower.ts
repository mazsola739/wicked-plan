// Hungarian for Word Tower (i18n.ts). The English text is the key.
// The topics and words stay English: the rules check every word against an English list.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Szótorony',
  short: 'Használd el az összes betűkockát: mindenki egy szót rak ki a témából, sorban egymás után.',
  strings: {
    You: 'Te',
    'Round {n} of {total}': '{n}. kör / {total}',
    Topic: 'Téma',
    'building…': 'épít…',
    waiting: 'vár',
    'No blocks left.': 'Nem maradt kocka.',
    'Your word: use every block left ({n}).': 'A te szavad: használd el az összes maradék kockát ({n}).',
    'Your word: one {topic} word from the blocks.': 'A te szavad: egy angol szó a kockákból, téma: {topic}.',
    '{name} is building…': '{name} épít…',
    'tap blocks': 'koppints a kockákra',
    Clear: 'Törlés',
    Build: 'Építés',
    'Take back the word before (−30)': 'Az előző szó visszavétele (−30)',
    'Keys: type letters, Backspace undoes, Enter builds.': 'Billentyűk: gépeld a betűket, a Backspace visszavon, az Enter épít.',
  },
  // Replay moment written by the rules (crates/rules/src/games/word_block_tower.rs).
  patterns: [['Seven letters in one word: ([A-Z]+)', 'Hét betű egyetlen szóban: $1']],
};

export default hu;
