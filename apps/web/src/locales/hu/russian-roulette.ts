// Hungarian for Russian Roulette (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Orosz rulett',
  short: 'Költsd 16 kreditedet nyomokra, a dob pörgetésére és titkos babrálásra, hogy elkerüld a festéktöltetet.',
  strings: {
    '{name} spun the drum.': '{name} megpörgette a dobot.',
    '{name} pulled: click, safe.': '{name} meghúzta: katt, megúszta.',
    '{name} pulled: SPLAT, hit!': '{name} meghúzta: PACCS, találat!',
    'Both charges spent: the drum is reloaded.': 'Mindkét töltet elfogyott: a dobot újratöltik.',
    'The drum': 'A dob',
    'Your turn: act, then pull.': 'Te jössz: cselekedj, aztán húzd meg.',
    '{name} has the drum…': '{name} kezében a dob…',
    'The show is over.': 'Vége a műsornak.',
    'Pull {n} of {total}': 'Húzás: {n} / {total}',
    'Charges left': 'Maradék töltet',
    'Table odds: {n} in {total}': 'Esély az asztalnál: {n} / {total}',
    out: 'kiesett',
    'Your credits:': 'Kreditjeid:',
    '(secret)': '(titkos)',
    'Peek:': 'Bepillantás:',
    CHARGED: 'TÖLTÖTT',
    empty: 'üres',
    Peek: 'Bepillantás',
    Spin: 'Pörgetés',
    Tamper: 'Babrálás',
    Pull: 'Húzás',
    'P peek · S spin · T tamper · Space pull.': 'P bepillantás · S pörgetés · T babrálás · Szóköz húzás.',
  },
  // Replay moments written by the rules (crates/rules/src/games/russian_roulette.rs).
  patterns: [
    ['Last life, (\\d+) charges in (\\d+) chambers: click', 'Utolsó élet, $1 töltet $2 kamrában: katt'],
    ['Out: (\\{[a-z0-9-]+\\}) is the last one standing', 'Kiesés: $1 maradt egyedül talpon'],
  ],
};

export default hu;
