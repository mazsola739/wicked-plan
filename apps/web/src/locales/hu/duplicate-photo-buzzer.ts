// Hungarian for Montage (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Képsor',
  short: 'Képek villannak fel egymás után; nyomd meg a gombot, amint az egyik megismétlődik, mielőtt a riválisok tennék.',
  strings: {
    Picture: 'Kép',
    'Round {n} of {total}': '{n}. kör / {total}',
    'Picture {first} came back as picture {second}.': 'A(z) {first}. kép tért vissza {second}. képként.',
    'Picture {n}': '{n}. kép',
    'You buzzed first!': 'Te nyomtad meg elsőként!',
    '{name} buzzed first.': '{name} nyomta meg elsőként.',
    'Nobody caught the repeat.': 'Senki nem vette észre az ismétlést.',
    'Next round': 'Következő kör',
    Finish: 'Befejezés',
    'Out this round': 'Ebben a körben kiestél',
    BUZZ: 'MOST!',
    'buzzed first': 'elsőként nyomta',
    'wrong buzz': 'rossz nyomás',
    'Buzz while the repeat or the picture after it is up: 100 + up to 60 for speed. A wrong buzz: −30.':
      'Nyomd meg, amíg az ismétlés vagy az utána jövő kép látszik: 100 + legfeljebb 60 a gyorsaságért. Rossz nyomás: −30.',
    'Keys: Space or B buzzes, Enter goes on.': 'Billentyűk: a Szóköz vagy a B nyom, az Enter továbblép.',
  },
  // Replay moments written by the rules (crates/rules/src/games/duplicate_photo_buzzer.rs).
  patterns: [
    ['Repeat buzzed in (\\d+)\\.(\\d) s', 'Ismétlés elkapva $1,$2 mp alatt'],
    ['Lead taken in the last round', 'Vezetés átvéve az utolsó körben'],
  ],
};

export default hu;
