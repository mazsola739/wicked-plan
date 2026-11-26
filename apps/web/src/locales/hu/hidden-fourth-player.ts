// Hungarian for Four Player Three in a Row (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Négyszemélyes amőba',
  short: 'Hárman raktok ki hármas sorokat egy negyedik ellen, aki magától játszik. Együtt állítsátok meg, vagy mind veszítetek.',
  strings: {
    'Your stone. The Shadow threatens {n} squares!': 'Te raksz. Az Árnyék {n} mezőt fenyeget!',
    'Your stone. The Shadow threatens {n} square!': 'Te raksz. Az Árnyék {n} mezőt fenyeget!',
    'Your stone: build lines, keep the Shadow down.': 'Te raksz: építs sorokat, és tartsd vissza az Árnyékot.',
    '{name} is placing…': '{name} rak…',
    'Round {n}': '{n}. kör',
    '{n} line': '{n} sor',
    '{n} lines': '{n} sor',
    'The Shadow': 'Az Árnyék',
    'plays after the three of you': 'hármatok után lép',
    'Lines score 100 each, and everyone gets +300 if the board fills up. If the Shadow makes three in a row, the game ends and lines are worth half.':
      'Minden sor 100 pontot ér, és mindenki +300-at kap, ha megtelik a tábla. Ha az Árnyék kirak hármat egy sorban, vége a játéknak, és a sorok csak a felét érik.',
    'Arrow keys move, Enter places.': 'A nyilak mozgatnak, az Enter lerak.',
  },
  // Replay moments written by the rules (crates/rules/src/games/hidden_fourth_player.rs).
  patterns: [
    ['(\\d+) lines with one stone', '$1 sor egyetlen kővel'],
    ['One stone: Shadow blocked, line made', 'Egy kő: az Árnyék megállítva, és kész egy sor'],
  ],
};

export default hu;
