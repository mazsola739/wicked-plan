// Hungarian for Rising Water Well (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Emelkedő víz kútja',
  short: 'Fejts meg négy képes rejtvényt a kút falán, mielőtt a víz felér a tetejéig.',
  strings: {
    red: 'piros',
    blue: 'kék',
    gold: 'arany',
    circle: 'kör',
    triangle: 'háromszög',
    square: 'négyzet',
    star: 'csillag',
    'How many {shape}s of any colour?': 'Hány {shape} van, bármilyen színben?',
    'How many {colour} {shape}s?': 'Hány {colour} {shape} van?',
    'The scratches follow a rule. How many marks come next?': 'A karcolások egy szabályt követnek. Hány vonás jön a következő mezőbe?',
    'Both scales balance. How many circles weigh as much as one triangle?':
      'Mindkét mérleg egyensúlyban van. Hány kör nyom annyit, mint egy háromszög?',
    'Each symbol stands for a number. What is the last sum?': 'Minden jel egy számot jelent. Mennyi az utolsó összeg?',
    'This clock is seen in the water below it. What hour does the real clock show?':
      'Ezt az órát az alatta lévő víz tükrében látod. Hány órát mutat a valódi óra?',
    'Riddle {n} of {total}': '{n}. rejtvény / {total}',
    'The riddle on the well wall': 'A rejtvény a kút falán',
    '{n} of {total} locks open': '{n} / {total} lakat nyitva',
    '{n} is wrong: the water rises!': '{n}: rossz válasz, emelkedik a víz!',
    'Give up': 'Feladom',
    'The grate opens: you climb out!': 'Kinyílik a rács: kimászol!',
    'The well is full.': 'Megtelt a kút.',
  },
  // Replay moments written by the rules (crates/rules/src/games/rising_water_well.rs).
  patterns: [
    ['Flawless escape, (\\d+:\\d\\d) to spare', 'Hibátlan menekülés, $1 tartalékkal'],
    ['Out of the well with (\\d+:\\d\\d) left', 'Kint a kútból, $1 maradt'],
  ],
};

export default hu;
