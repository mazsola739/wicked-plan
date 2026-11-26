// Hungarian for The Knight's Tour (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Huszárvándorlás',
  short: 'Járd be a huszárral az összes mezőt, mindegyiket csak egyszer. Három tábla, 15 perc.',
  strings: {
    knight: 'huszár',
    'Board {n} of {total}': '{n}. tábla / {total}',
    'Put the knight on any square.': 'Tedd le a huszárt bármelyik mezőre.',
    '{n} / {total} squares': '{n} / {total} mező',
    'Stuck: no free jump.': 'Elakadtál: nincs szabad ugrás.',
    'Restart this board': 'Tábla újrakezdése',
  },
  // Replay moments written by the rules (crates/rules/src/games/knights_tour.rs).
  patterns: [
    ['Three tours, not one restart, (\\d+) s left', 'Három bejárás egyetlen újrakezdés nélkül, $1 mp maradt'],
    ['All three tours done, (\\d+) s to spare', 'Mind a három bejárás kész, $1 mp tartalékkal'],
  ],
};

export default hu;
