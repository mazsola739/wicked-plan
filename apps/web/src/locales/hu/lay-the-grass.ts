// Hungarian for Lay the Grass (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Fűszőnyeg',
  short: 'Minden körben mindenki ugyanazt a fűdarabot rakja le: növeszd a legnagyobb foltot és a legnagyobb négyzetet.',
  strings: {
    'Grass piece': 'Fűdarab',
    'Round {n} of {total}': '{n}. kör / {total}',
    'Square {x}, {y}': 'Mező: {x}, {y}',
    'The field is full.': 'Megtelt a mező.',
    'Your turn: tap where the piece goes, tap again to lay it.':
      'Te lépsz: koppints oda, ahová a darab kerül, és koppints újra a lerakáshoz.',
    '{name} is laying grass…': '{name} füvet rak le…',
    'this round': 'ebben a körben',
    next: 'következő',
    Turn: 'Forgat',
    Flip: 'Tükröz',
    Lay: 'Lerak',
    laying: 'rak',
    'patch {n} · square {side}×{side}': 'folt {n} · négyzet {side}×{side}',
    'Score: 10 per square of your largest patch + 20 × side² of your largest grass square.':
      'Pontszám: a legnagyobb foltod minden mezője 10 + a legnagyobb fűnégyzeted oldala² × 20.',
    'Keys: arrows move, R turns, F flips, Enter lays.': 'Billentyűk: a nyilak mozgatnak, R forgat, F tükröz, Enter lerak.',
    // Replay moment written by the rules (crates/rules/src/games/lay_the_grass.rs).
    'First 3×3 square of grass on the field': 'Az első 3×3-as fűnégyzet a mezőn',
  },
  patterns: [['A (\\d+)×(\\d+) square of grass: (\\d+) bonus', 'Egy $1×$2-es fűnégyzet: $3 bónusz']],
};

export default hu;
