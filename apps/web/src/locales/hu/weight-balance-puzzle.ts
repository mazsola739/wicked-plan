// Hungarian for Scale Game (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Mérlegjáték',
  short: 'Osszátok be a 12 mérést egyetlen mérlegen, és találjátok ki öt kocka súlyát.',
  strings: {
    You: 'Te',
    nothing: 'semmi',
    'left heavier': 'a bal nehezebb',
    'right heavier': 'a jobb nehezebb',
    balanced: 'egyensúlyban',
    'Balance scale': 'Kétkarú mérleg',
    'No weighings yet.': 'Még nem volt mérés.',
    vs: 'vs',
    Left: 'Bal',
    Right: 'Jobb',
    left: 'bal',
    right: 'jobb',
    off: 'lent',
    'Your weighing: tap blocks to put them on a pan.': 'Te mérsz: koppints a kockákra, hogy a serpenyőkbe tedd őket.',
    '{name} is weighing…': '{name} mér…',
    'Answer now': 'Válaszolok',
    Weigh: 'Mérés',
    'Keys: A–E move a block (off → left → right), Enter weighs.':
      'Billentyűk: az A–E áthelyez egy kockát (lent → bal → jobb), az Enter mér.',
    'Sent. Waiting for your teammate…': 'Elküldve. Várakozás a csapattársadra…',
    'Write down every block’s weight (1–9, all different).': 'Írd be minden kocka súlyát (1–9, mind különböző).',
    'Send answer': 'Válasz elküldése',
    'Keys: A–E pick a block, 1–9 write its weight, Enter sends.':
      'Billentyűk: az A–E kiválaszt egy kockát, az 1–9 beírja a súlyát, az Enter elküldi.',
    'Weighing · {n} of {total} left': 'Mérés · még {n} / {total}',
    'Answer in secret': 'Titkos válasz',
    'Five blocks, different whole weights 1–9 g. 150 points per block everyone gets right; all five right: +50 per unused weighing.':
      'Öt kocka, különböző egész súlyok 1–9 g között. 150 pont minden kockáért, amit mindenki eltalál; mind az öt jó: +50 minden fel nem használt mérésért.',
  },
  // Replay moment written by the rules (crates/rules/src/games/weight_balance_puzzle.rs).
  patterns: [['All five weights right, (\\d+) weighings to spare', 'Mind az öt súly jó, $1 mérés maradt']],
};

export default hu;
