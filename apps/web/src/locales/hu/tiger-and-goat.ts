// Hungarian for Bagh Chal (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Tigris és kecskék',
  short: 'A vadászjáték két táblán egyszerre: az egyiken tigrisek vagytok, a másikon kecskék.',
  strings: {
    tigers: 'tigrisek',
    goats: 'kecskék',
    'You won ({side})': 'Nyertél ({side})',
    '{name} won ({side})': '{name} nyert ({side})',
    'Ended: turn limit': 'Vége: elfogytak a körök',
    'Your move ({side})': 'Te lépsz ({side})',
    'Waiting for the next step': 'Várakozás a következő lépésre',
    '{name} is moving': '{name} lép',
    'Step {n} · {name} {score}': '{n}. lépés · {name} {score}',
    'Board {name}': '{name} tábla',
    'Board {name} · you are {side}': '{name} tábla · te: {side}',
    'Captured {captured}/5 · goats in hand {hand} · turn {turns}/{max} · {points} pts':
      'Elfogva {captured}/5 · kecske kézben {hand} · kör {turns}/{max} · {points} pont',
    'Tigers: slide along a line, or jump over a goat to capture it. Five captures win.':
      'Tigrisek: csússz egy vonal mentén, vagy ugord át a kecskét, hogy elfogd. Öt elfogás nyer.',
    'Goats: place all 20, then slide. Trap every tiger to win; never leave a goat open to a jump.':
      'Kecskék: tedd le mind a 20-at, aztán csúsztasd őket. Zárj be minden tigrist a győzelemhez; soha ne hagyj kecskét átugorhatóan.',
    'Click a piece, then where it goes. V switches boards, Esc cancels.':
      'Kattints egy bábura, aztán oda, ahová lép. A V táblát vált, az Esc visszavon.',
  },
  // Replay moments written by the rules (crates/rules/src/games/tiger_and_goat.rs).
  patterns: [
    ['Every tiger trapped: board ([AB]) won', 'Minden tigris csapdában: $1 tábla megnyerve'],
    [
      'Board ([AB]) won: fifth goat taken after all 20 placed',
      '$1 tábla megnyerve: az ötödik kecske is elfogva, miután mind a 20 felkerült',
    ],
    ['Board ([AB]) held for all (\\d+) turns', '$1 tábla kitartott mind a $2 körön át'],
  ],
};

export default hu;
