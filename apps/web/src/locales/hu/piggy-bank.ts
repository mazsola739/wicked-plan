// Hungarian for Piggy Bank (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Malacpersely',
  short: 'Dobj érméket a perselybe, és add tovább; aki megkapja, megmondja, hány van benne egy színből, a többiek kételkedhetnek.',
  strings: {
    white: 'fehér',
    black: 'fekete',
    '{colour} coin': '{colour} érme',
    'Piggy bank, {n} coins': 'Malacpersely, {n} érme',
    'Drop 1 or 2 coins, then pass the bank.': 'Dobj be 1 vagy 2 érmét, aztán add tovább a perselyt.',
    '{name} is dropping coins…': '{name} érméket dob be…',
    'How many {colour} coins are inside?': 'Hány {colour} érme van benne?',
    '{name} is peeking…': '{name} belekukucskál…',
    '{name} says {n} {colour}. Doubt it?': '{name} szerint {n} {colour}. Kételkedsz?',
    'You doubt it. Waiting…': 'Kételkedsz. Várakozás…',
    'You trust it. Waiting…': 'Elhiszed. Várakozás…',
    'Waiting for the others to judge your claim…': 'Várakozás, hogy a többiek megítéljék az állításodat…',
    right: 'igaz',
    wrong: 'hamis',
    '{names} loses a point.': 'Mínusz egy pont: {names}.',
    '{names} lose a point.': 'Mínusz egy pont: {names}.',
    'Nobody loses.': 'Senki sem veszít.',
    'Full bank!': 'Tele a persely!',
    '{name} said {claim}, there were {truth}: {verdict}.': '{name} szerint {claim}, valójában {truth} volt: {verdict}.',
    'Round {n} of {total}': 'Kör: {n} / {total}',
    'This round counts': 'Ebben a körben számít',
    'Inside:': 'Benne:',
    'Last accepted claim: {n}': 'Utolsó elfogadott állítás: {n}',
    'Your coins: {n} {colour}, {other} other': 'Érméid: {n} {colour}, {other} másik',
    claimer: 'állító',
    decided: 'döntött',
    'thinking…': 'gondolkodik…',
    doubted: 'kételkedett',
    dropped: 'bedobva',
    'Claims:': 'Állítások:',
    '{name} {n} of {total}': '{name} {n}/{total}',
    'No coins yet': 'Még nincs érme',
    '+ White': '+ Fehér',
    '+ Black': '+ Fekete',
    Clear: 'Törlés',
    'Drop and pass': 'Bedob és továbbad',
    'You see the last drop:': 'Látod az utolsó bedobást:',
    'Claim {n} {colour}': 'Állítás: {n} {colour}',
    Trust: 'Elhiszem',
    'Doubt!': 'Kételkedem!',
    'W / B add a coin, Enter drops · digits and Enter claim · D doubt, T trust.':
      'W / B érmét tesz hozzá, Enter bedob · számjegyek és Enter: állítás · D kétely, T elhiszem.',
  },
  // Replay moments written by the rules (crates/rules/src/games/piggy_bank.rs).
  patterns: [
    ['Claim of (\\d+) white holds: (\\d+) doubters pay', 'Igaz állítás, $1 fehér: $2 kételkedő fizet'],
    ['Claim of (\\d+) black holds: (\\d+) doubters pay', 'Igaz állítás, $1 fekete: $2 kételkedő fizet'],
    ['Lone doubt: (\\{[a-z0-9-]+\\}) claimed (\\d+) white, there were (\\d+)', 'Magányos kétely: $1 szerint $2 fehér volt, valójában $3'],
    ['Lone doubt: (\\{[a-z0-9-]+\\}) claimed (\\d+) black, there were (\\d+)', 'Magányos kétely: $1 szerint $2 fekete volt, valójában $3'],
  ],
};

export default hu;
