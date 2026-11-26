// Hungarian for Secret Code (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Titkos kód',
  short: 'Négy kocka a pohár alatt: fejtsd meg az értéküket a pontos és a közeli jelzésekből, gyorsabban a riválisoknál.',
  strings: {
    empty: 'üres',
    '{exact} exact, {near} near': '{exact} pontos, {near} közeli',
    'Code {n} of {total}': '{n}. kód / {total}',
    Total: 'Összesen',
    pips: 'pötty',
    'Remove die {n}': '{n}. kocka levétele',
    '{n} left': 'még {n}',
    'right face, right place': 'jó érték, jó helyen',
    'right face, wrong place': 'jó érték, rossz helyen',
    'not solved': 'nem fejtette meg',
    'Next code': 'Következő kód',
    Finish: 'Befejezés',
    Guess: 'Tipp',
    'cracked it in {n}': '{n} tippből megfejtette',
    '{n} guess': '{n} tipp',
    '{n} guesses': '{n} tipp',
    'Keys: 1–6 add a die, Backspace removes, Enter guesses.': 'Billentyűk: 1–6 kockát tesz be, a Backspace kivesz, az Enter tippel.',
  },
  // Replay moments written by the rules (crates/rules/src/games/secret_code.rs).
  patterns: [
    ['Code cracked in one guess', 'Kód megfejtve egyetlen tippből'],
    ['Cracked in 2 guesses, fastest at the table', 'Megfejtve 2 tippből, a leggyorsabban az asztalnál'],
  ],
};

export default hu;
