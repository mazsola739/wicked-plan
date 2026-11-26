// Hungarian for Unknown (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// Directions (Dir::name in crates/rules/src/games/hidden_rules_race.rs).
const DIRS: [string, string][] = [
  ['up', 'fel'],
  ['right', 'jobbra'],
  ['down', 'le'],
  ['left', 'balra'],
];

const hu: Locale = {
  name: 'Ismeretlen',
  short: 'Vezesd a kockádat egy színes tábla közepére, miközben kitalálod, melyik szín milyen titkos mozgásszabályt rejt.',
  strings: {
    // Colours and rules (COLOURS, Effect::name).
    red: 'piros',
    blue: 'kék',
    green: 'zöld',
    yellow: 'sárga',
    purple: 'lila',
    'step one': 'egyet lép',
    'step two': 'kettőt lép',
    backwards: 'hátrafelé',
    'turn right': 'jobbra fordul',
    'turn left': 'balra fordul',
    goal: 'cél',
    'stayed (no choice)': 'helyben maradt (nem választott)',
    '{arrow} · did not move': '{arrow} · nem mozdult',
    '{arrow} · moved {n} {way}': '{arrow} · {n} mezőt lépett {way}',
    'Round {n} of {total}': '{n}. kör / {total}',
    'The rules revealed': 'A szabályok kiderültek',
    'You chose {arrow}. Waiting for the others…': 'Ezt választottad: {arrow}. Várakozás a többiekre…',
    'You stand on {colour}. Pick a direction.': 'Ezen a színen állsz: {colour}. Válassz irányt.',
    'Arrow keys choose a direction.': 'A nyilakkal választasz irányt.',
    'See results': 'Eredmények',
    Colours: 'Színek',
    '(tap to note your guess)': '(koppints, és jegyezd fel a tipped)',
    'Last round': 'Előző kör',
  },
  // Replay moments written by the rules.
  patterns: [
    ...DIRS.flatMap(([en, hu]): [string, string][] => [
      [`Pointed ${en}, backed onto the goal`, `Irány: ${hu}, és hátrafelé a célba`],
      [`Pointed ${en}, turned onto the goal`, `Irány: ${hu}, és fordulva a célba`],
    ]),
    ['Two squares onto the goal, past (\\{[a-z0-9-]+\\})', 'Két mezőt lépve a célba, elhúzva mellette: $1'],
  ],
};

export default hu;
