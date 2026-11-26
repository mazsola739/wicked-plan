// Hungarian for Cooperative Puzzle (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Közös kirakó',
  short: 'A kerék játékosról játékosra fordul: pár másodperc jut egy darab lerakására. Rakjátok ki együtt az alakzatokat.',
  strings: {
    You: 'Te',
    'Puzzle wheel': 'Kirakókerék',
    done: 'kész',
    'Your turn: quick!': 'Te jössz: gyorsan!',
    "{name}'s turn · {n} turns left": '{name} jön · még {n} kör',
    'Current puzzle': 'Az aktuális kirakó',
    'Place one piece on the shape (or take one back).': 'Tegyél le egy darabot az alakzatra (vagy vegyél vissza egyet).',
    '{name} is on puzzle {n}…': '{name} a(z) {n}. kirakón dolgozik…',
    '↻ Turn': '↻ Forgat',
    '⇋ Flip': '⇋ Tükröz',
    Pass: 'Passz',
    'Take back': 'Visszavesz',
    Place: 'Lerak',
    '100 per finished shape; all six: +10 per turn left.': 'Kész alakzatonként 100; mind a hat: +10 minden megmaradt körért.',
    'Keys: 1–4 pick, R turns, F flips, arrows move, Enter places, P passes.':
      'Billentyűk: 1–4 választ, R forgat, F tükröz, a nyilak mozgatnak, Enter lerak, P passzol.',
    // The last action (the rules' `last`), with "You" for your own.
    'You: placed a piece': 'Te: leraktál egy darabot',
    'You: took a piece back': 'Te: visszavettél egy darabot',
    'You: passed': 'Te: passzoltál',
    'You: ran out of time': 'Te: lejárt az időd',
  },
  // The last action and the replay moments written by the rules (crates/rules/src/games/spinning_puzzle_wheel.rs).
  patterns: [
    ['(.+): placed a piece', '$1: lerakott egy darabot'],
    ['(.+): took a piece back', '$1: visszavett egy darabot'],
    ['(.+): passed', '$1: passzolt'],
    ['(.+): ran out of time', '$1: lejárt az ideje'],
    ['Perfect wheel: all six in (\\d+) turns', 'Tökéletes kerék: mind a hat $1 kör alatt'],
    ['All six shapes done, (\\d+) turns to spare', 'Mind a hat alakzat kész, $1 kör maradt'],
  ],
};

export default hu;
