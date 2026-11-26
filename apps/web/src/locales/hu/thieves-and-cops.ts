// Hungarian for Crooked Cops (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// The metro stations (crates/rules/src/games/thieves_and_cops.rs, STATIONS), also used to build the
// log lines and replay moments below, which name a station.
const STATIONS: Record<string, string> = {
  Harbour: 'Kikötő',
  Market: 'Piac',
  Central: 'Központ',
  Museum: 'Múzeum',
  Airport: 'Repülőtér',
  'North Gate': 'Északi kapu',
  Library: 'Könyvtár',
  Casino: 'Kaszinó',
  Stadium: 'Stadion',
  'South Pier': 'Déli móló',
  'Old Town': 'Óváros',
  Park: 'Park',
  Tower: 'Torony',
  Factory: 'Gyár',
  Docks: 'Dokkok',
  Bank: 'Bank',
  Zoo: 'Állatkert',
};

/** "a Kikötő állomáson", "az Óváros állomáson". */
const at = (hu: string) => `${/^[AÁEÉIÍOÓÖŐUÚÜŰ]/.test(hu) ? 'az' : 'a'} ${hu} állomáson`;
const cap = (s: string) => s[0]?.toUpperCase() + s.slice(1);
const ID = '(\\S+)';
const SEAT = '(\\{[a-z0-9-]+\\})';

const stationPatterns = Object.entries(STATIONS).flatMap(([en, hu]): [string, string][] => [
  [`${ID} catches ${ID} at ${en} \\((\\d+) pieces lost\\)!`, `${cap(at(hu))} elfogták: $2 (elfogó: $1, $3 kincs elveszett)!`],
  [`${ID} catches ${ID} at ${en}!`, `${cap(at(hu))} elfogták: $2 (elfogó: $1)!`],
  [`A piece was taken at ${en}\\.`, `Kincset vittek el ${at(hu)}.`],
  [`Sighting: ${ID} at ${en}\\.`, `Észlelés: $1, ${at(hu)}.`],
  // Replay moments.
  [`${SEAT} caught at ${en}: 1 piece lost`, `$1 elfogva ${at(hu)}, 1 kincs elveszett`],
  [`${SEAT} caught at ${en}: (\\d+) pieces lost`, `$1 elfogva ${at(hu)}, $2 kincs elveszett`],
  [`${SEAT} caught at ${en}`, `$1 elfogva ${at(hu)}`],
  [`Winning piece taken at ${en} in round (\\d+)`, `A döntő kincs ${at(hu)}, a $1. körben`],
]);

const hu: Locale = {
  name: 'Korrupt zsaruk',
  short: 'A tolvajok rejtett kincseket gyűjtenek egy metrótérképen, a rendőrök vadásznak rájuk; az egyik rendőr titokban korrupt.',
  strings: {
    ...STATIONS,
    // The screen.
    'Thief: collect pieces with the other thieves and stay away from the police.':
      'Tolvaj: gyűjts kincseket a többi tolvajjal, és kerüld el a rendőröket.',
    'Police: catch the thieves. One of your colleagues is corrupt.': 'Rendőr: kapd el a tolvajokat. Az egyik kollégád korrupt.',
    'Corrupt officer: act like police, but help the thieves win. You never catch anyone.':
      'Korrupt rendőr: játszd a rendőrt, de segítsd győzelemre a tolvajokat. Te soha senkit nem kapsz el.',
    'Everything revealed': 'Minden kiderült',
    'Unmask the corrupt officer': 'Leplezd le a korrupt rendőrt',
    'Round {n} of {total}': '{n}. kör / {total}',
    'Metro map': 'Metrótérkép',
    'R{n}': 'K{n}',
    police: 'rendőr',
    thief: 'tolvaj',
    corrupt: 'korrupt',
    'Pieces kept by the thieves:': 'A tolvajok kincsei:',
    'of {n} needed': '/ {n} kell',
    'You were caught.': 'Elkaptak.',
    'Tap a lit station to move one stop, or stay.': 'Koppints egy kivilágított állomásra, hogy egy megállót lépj, vagy maradj.',
    'Stay at {station}': 'Maradok: {station}',
    'S stays.': 'S: maradsz.',
    'Who is the corrupt officer?': 'Ki a korrupt rendőr?',
    'The police are voting…': 'A rendőrök szavaznak…',
    'Vote sent. Waiting…': 'Szavazat elküldve. Várakozás…',
    'The thieves win.': 'A tolvajok nyertek.',
    'The police win.': 'A rendőrök nyertek.',
    'The corrupt officer was unmasked.': 'A korrupt rendőr lelepleződött.',
    'Your score: {n}': 'A pontszámod: {n}',
    'See results': 'Eredmények',
    caught: 'elkapva',
    '{n} piece': '{n} kincs',
    '{n} pieces': '{n} kincs',
    '{n} catch': '{n} elfogás',
    '{n} catches': '{n} elfogás',

    // The log (the rules).
    'Round 1: everyone moves at once.': '1. kör: mindenki egyszerre lép.',
    'The thieves have 5 pieces!': 'A tolvajoknál 5 kincs van!',
    'The chase is over.': 'Vége a hajszának.',
    'Unmask: name the officer you think is corrupt.': 'Leleplezés: nevezd meg a rendőrt, akit korruptnak tartasz.',
  },
  patterns: [
    ...stationPatterns,
    [`${ID} is unmasked as the corrupt officer!`, 'Lelepleződött a korrupt rendőr: $1!'],
    [`The corrupt officer, ${ID}, got away with it\\.`, 'A korrupt rendőr megúszta: $1.'],
  ],
};

export default hu;
