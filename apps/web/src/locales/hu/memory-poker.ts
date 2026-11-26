// Hungarian for Memory Poker (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// Hand names as the rules write them (crates/rules/src/games/memory_poker.rs, HAND_NAMES).
const HANDS: [string, string][] = [
  ['High card', 'Magas lap'],
  ['Pair', 'Pár'],
  ['Two pair', 'Két pár'],
  ['Three of a kind', 'Drill'],
  ['Straight', 'Sor'],
  ['Flush', 'Szín'],
  ['Full house', 'Full'],
  ['Four of a kind', 'Póker'],
  ['Straight flush', 'Színsor'],
];

const hu: Locale = {
  name: 'Memóriapóker',
  short: 'Jegyezz meg 16 kártyát, aztán vegyél el kettőt vakon: a három közös lappal a legjobb kéz nyer.',
  strings: {
    ...Object.fromEntries(HANDS),
    spades: 'pikk',
    hearts: 'kőr',
    diamonds: 'káró',
    clubs: 'treff',
    'Memorise the grid!': 'Jegyezd meg a rácsot!',
    'Take two cards': 'Vegyél el két kártyát',
    Showdown: 'Leleplezés',
    'Tap Ready when you have them memorised.': 'Koppints a Kész gombra, ha megjegyezted őket.',
    'Your pick: tap a face-down card.': 'Te választasz: koppints egy lefordított kártyára.',
    '{name} is picking…': '{name} választ…',
    'Two cards + the shared three make each hand.': 'Két kártya + a három közös adja ki mindenki kezét.',
    'Round {n} of {total}': '{n}. kör / {total}',
    Shared: 'Közös',
    'Square {letter}': '{letter} mező',
    Ready: 'Kész',
    'Next round': 'Következő kör',
    Finish: 'Befejezés',
    picking: 'választ',
    'Pair 50 · two pair 90 · three 130 · straight 180 · flush 200 · full house 250 · four 320 · straight flush 400 · best hand +60':
      'Pár 50 · két pár 90 · drill 130 · sor 180 · szín 200 · full 250 · póker 320 · színsor 400 · legjobb kéz +60',
    'Keys: A–P pick a square, Enter is Ready / Next.': 'Billentyűk: A–P választ egy mezőt, az Enter a Kész / Tovább.',
  },
  // Replay moments: "<hand> from memory (+n)".
  patterns: HANDS.map(([en, hu]): [string, string] => [`${en} from memory \\(\\+(\\d+)\\)`, `${hu} fejből (+$1)`]),
};

export default hu;
