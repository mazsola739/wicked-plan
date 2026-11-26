// Hungarian for Halloween Monster (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// The monsters (crates/rules/src/games/monster_alliance.rs, NAMES), also used to build the log
// lines and replay moments below, which name the monster.
const MONSTERS: Record<string, string> = {
  Gloomtusk: 'Komoragyar',
  Rustjaw: 'Rozsdapofa',
  Mirebeast: 'Lápszörny',
  Thornback: 'Tüskehát',
  'Hollow King': 'Odvas Király',
  Ashwing: 'Hamuszárny',
  Sludgemaw: 'Iszaptorok',
  Ironhide: 'Vasbőr',
};

const SEAT = '(\\{[a-z0-9-]+\\})';

const monsterPatterns = Object.entries(MONSTERS).flatMap(([en, hu]): [string, string][] => [
  [`${en} \\(strength (\\d+)\\) is beaten by (.+) and (.+) with power (\\d+)\\.`, `${hu} (erő: $1) legyőzve: $2 és $3, $4 erővel.`],
  [`${en} \\(strength (\\d+)\\) is beaten by (.+) with power (\\d+)\\.`, `${hu} (erő: $1) legyőzve: $2, $3 erővel.`],
  [`Nobody beats ${en} \\(strength (\\d+)\\)\\.`, `Senki sem győzte le: ${hu} (erő: $1).`],
  // Replay moments.
  [`Beat ${en} alone: all (\\d+) points`, `${hu} legyőzve egyedül: mind a $1 pont`],
  [`Beat ${en}, strength (\\d+), with ${SEAT} and ${SEAT}`, `${hu} legyőzve (erő: $1), társak: $2 és $3`],
  [`Beat ${en}, strength (\\d+), with ${SEAT}`, `${hu} legyőzve (erő: $1), társ: $2`],
]);

const hu: Locale = {
  name: 'Halloweeni szörny',
  short: 'Licitálj fegyverekre, köss legfeljebb háromfős szövetségeket, és győzzétek le együtt a szörnyeket, hogy megosszátok a jutalmat.',
  strings: {
    ...MONSTERS,
    // The screen.
    'Bid for a weapon': 'Licit egy fegyverre',
    'Form alliances': 'Szövetségkötés',
    'Attack!': 'Támadás!',
    'Monster {n} of {total}': '{n}. szörny / {total}',
    Monster: 'Szörny',
    Strength: 'Erő',
    Reward: 'Jutalom',
    'Extra weapon for auction:': 'Elárverezett extra fegyver:',
    with: 'szövetségben:',
    '{points} pts · {cards} cards': '{points} pont · {cards} kártya',
    'Last attack': 'Legutóbbi támadás',
    'power {n}': 'erő {n}',
    'Bid points for the extra weapon (you have {n}).': 'Licitálj pontokat az extra fegyverre ({n} pontod van).',
    'Pick up to two partners. An alliance forms only if they pick you too.':
      'Válassz legfeljebb két társat. Szövetség csak akkor jön létre, ha ők is téged választanak.',
    '{n} cards': '{n} kártya',
    'Ask them': 'Megkérem őket',
    'Fight alone': 'Egyedül harcolok',
    'Keys: 1–7 pick players, Enter confirms.': 'Billentyűk: 1–7 játékost választ, Enter megerősít.',
    'Your alliance: {names}.': 'A szövetséged: {names}.',
    'You fight alone.': 'Egyedül harcolsz.',
    "The monster's strength is {n}.": 'A szörny ereje {n}.',
    'Attack with {n}': 'Támadás {n} erővel',
    'Hold back': 'Kivárok',
    'Keys: 1–9 pick cards, Enter attacks.': 'Billentyűk: 1–9 kártyát választ, Enter támad.',
    'Your weapons:': 'A fegyvereid:',

    // The log (the rules).
    'A monster approaches. Bid for the extra weapon.': 'Közeleg egy szörny. Licitálj az extra fegyverre.',
    'Nobody wins the extra weapon.': 'Senki sem kapja meg az extra fegyvert.',
    'No alliances: everyone fights alone.': 'Nincs szövetség: mindenki egyedül harcol.',
  },
  patterns: [
    ...monsterPatterns,
    ['Bids: (.+)\\.', 'Licitek: $1.'],
    ['(\\S+) wins the (\\d+)-strength weapon for (\\d+) points\\.', 'Extra fegyver ($2 erő) $3 pontért: $1.'],
    ['Alliances: (.+)\\.', 'Szövetségek: $1.'],
  ],
};

export default hu;
