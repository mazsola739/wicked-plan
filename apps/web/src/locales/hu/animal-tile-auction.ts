// Hungarian for Zoo (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// The animals (ANIMALS in crates/rules/src/games/animal_tile_auction.rs): English, Hungarian,
// and "on the …".
const ANIMALS: [string, string, string][] = [
  ['fox', 'róka', 'rókán'],
  ['owl', 'bagoly', 'baglyon'],
  ['frog', 'béka', 'békán'],
  ['bear', 'medve', 'medvén'],
];
const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
const SEAT = '([a-z0-9-]+)';

const hu: Locale = {
  name: 'Állatkert',
  short: 'Licitálj zsetonokkal állatlapkákra, és tedd le őket a közös 5×5-ös táblára, hogy kirakd a titkos mintádat.',
  strings: {
    ...Object.fromEntries(ANIMALS.map(([en, name]) => [en, name])),
    'Bid {n} chip': 'Licit: {n} zseton',
    'Bid {n} chips': 'Licit: {n} zseton',
    'Game over: every pattern is scored.': 'Vége a játéknak: minden minta pontot kapott.',
    'You won the tile: tap an empty cell to place it.': 'Tiéd a lapka: koppints egy üres mezőre, hogy letedd.',
    '{name} is placing the tile…': '{name} lerakja a lapkát…',
    'Bid for this tile.': 'Licitálj erre a lapkára.',
    'Waiting for {names}…': 'Várakozás: {names}…',
    'the others': 'a többiek',
    'Game over': 'Vége a játéknak',
    'Auction {n} of {total}': '{n}. licit / {total}',
    'On offer': 'Most licitálható',
    'Your pattern': 'A mintád',
    'any rotation': 'bármilyen elforgatás',
    Patterns: 'Minták',
    'Keys: digits + Enter bid; arrows + Enter place.': 'Billentyűk: számok + Enter a licit; nyilak + Enter a lerakás.',
    'Game over: patterns are scored.': 'Vége a játéknak: a minták pontot kapnak.',
    'Secret pattern completed (+25)': 'Titkos minta kész (+25)',
  },
  // Log lines and replay moments written by the rules. Seat ids stay (the screen shows names).
  patterns: [
    ...ANIMALS.flatMap(([en, name, on]): [string, string][] => [
      [`${SEAT} wins the ${en} for (\\d+)\\.`, `${cap(name)} → $1 ($2 zseton).`],
      [`Tie at (\\d+): the ${en} is lost\\.`, `Döntetlen $1 zsetonnál: a ${name} elveszett.`],
      [`${SEAT} places the ${en} at ([A-E]\\d)\\.`, `${cap(name)} lerakva ide: $2 ($1).`],
      [`All (\\d+) chips on the ${en}`, `Minden zseton ($1) a ${on}`],
    ]),
    ['Secret pattern completed (\\d+) times \\(\\+(\\d+)\\)', 'Titkos minta kész $1× (+$2)'],
  ],
};

export default hu;
