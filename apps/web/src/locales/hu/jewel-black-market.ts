// Hungarian for Gem Black Market (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// The rules name a card "Diamond 5"; one pattern per kind gives its Hungarian name.
const KINDS: [string, string][] = [
  ['Diamond', 'Gyémánt'],
  ['Gold', 'Arany'],
  ['Silver', 'Ezüst'],
  ['Bronze', 'Bronz'],
  ['Bomb', 'Bomba'],
];
const SEAT = '(you|bot-\\d+)';

const hu: Locale = {
  name: 'Ékszer-feketepiac',
  short:
    'Mindenkinek egy rejtett kártyája van; cserélj a játékosokkal, a vevővel vagy az eldobott kártyával. Gyűjtsd a vevő ékszereit, kerüld a bombákat.',
  strings: {
    diamond: 'gyémánt',
    gold: 'arany',
    silver: 'ezüst',
    bronze: 'bronz',
    bomb: 'bomba',
    Discard: 'Eldobott',
    'Game over.': 'Vége a játéknak.',
    'You hold a bomb: pass it to a player.': 'Bomba van nálad: add tovább egy játékosnak.',
    'Your turn: keep, or swap with a player, the buyer or the discard.':
      'Te lépsz: tartsd meg, vagy cserélj egy játékossal, a vevővel vagy az eldobott kártyával.',
    '{name} is choosing…': '{name} választ…',
    'Round {n} of {total} · swap {swap} of 2': '{n}. kör / {total} · {swap}. csere / 2',
    "Buyer's space": 'A vevő helye',
    'Your card': 'A kártyád',
    'Keep my card': 'Megtartom a kártyám',
    "A jewel scores its value × the same-type jewels in the buyer's space. A bomb costs its value.":
      'Egy ékszer az értéke × a vevő helyén lévő azonos fajta ékszerek számát éri. A bomba az értékét veszi el.',
    'Round {n} revealed': 'A(z) {n}. kör felfedve',
    Buyer: 'Vevő',
    'Keys: K keeps, 1–3 buyer card, D discard, A–H a player.':
      'Billentyűk: K megtartja, 1–3 a vevő kártyái, D az eldobott, A–H egy játékos.',
    // Replay moment written by the rules (crates/rules/src/games/jewel_black_market.rs).
    'Lead stolen in the very last round': 'Az utolsó körben elhappolt vezetés',
  },
  // Log lines and moments written by the rules. Seat ids stay: the screen turns them into names.
  patterns: [
    ['Round (\\d+) dealt\\.', 'A(z) $1. kör kiosztva.'],
    ['you kept their card\\.', 'Megtartottad a kártyádat.'],
    [`${SEAT} kept their card\\.`, '$1 megtartotta a kártyáját.'],
    [`you swapped with ${SEAT}\\.`, 'Kártyát cseréltél vele: $1.'],
    [`${SEAT} swapped with you\\.`, '$1 kártyát cserélt veled.'],
    [`${SEAT} swapped with ${SEAT}\\.`, '$1 kártyát cserélt vele: $2.'],
    ['you swapped with buyer card (\\d+)\\.', 'Kicserélted a vevő $1. kártyáját.'],
    [`${SEAT} swapped with buyer card (\\d+)\\.`, '$1 kicserélte a vevő $2. kártyáját.'],
    ['you swapped with the discard\\.', 'Kicserélted az eldobott kártyát.'],
    [`${SEAT} swapped with the discard\\.`, '$1 kicserélte az eldobott kártyát.'],
    ...KINDS.flatMap(([en, hu]): [string, string][] => [
      [`You got ${en} (-?\\d+) from the buyer\\.`, `A vevőtől ezt kaptad: ${hu} $1.`],
      [`You got ${en} (-?\\d+) from the discard\\.`, `Az eldobott helyéről ezt kaptad: ${hu} $1.`],
      [`You got ${en} (-?\\d+) from ${SEAT}\\.`, `$2 átadta neked: ${hu} $1.`],
      [`${en} (-?\\d+) × (\\d+) in the buyer's space \\(\\+(\\d+)\\)`, `${hu} $1 × $2 a vevő helyén (+$3)`],
    ]),
  ],
};

export default hu;
