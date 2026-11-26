// Hungarian for Big Small (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Nagy-kicsi',
  short: 'Játssz ki titokban egy kártyát, és fogadj, hogy az ellenfeled kártyája üti-e az osztóét. Kilenc kör zsetonokért.',
  strings: {
    'Round {n} of {total}': 'Kör: {n} / {total}',
    Dealer: 'Osztó',
    "{name}'s cards left": '{name} maradék kártyái',
    'Last round (dealer {n}): {name} played': 'Előző kör (osztó: {n}): {name} kártyája',
    ', your bet': ', a tipped',
    'you played': 'a te kártyád',
    ', {name} bet': ', {name} tippje',
    'Waiting for {name}…': 'Várakozás: {name}…',
    'Your card': 'A kártyád',
    '{name} has chosen': '{name} már választott',
    "{name}'s card compared with {n}": '{name} kártyája az osztó {n} lapjához képest',
    Smaller: 'Kisebb',
    'Equal ×2': 'Egyenlő ×2',
    Bigger: 'Nagyobb',
    Stake: 'Tét',
    '{n} chip': '{n} zseton',
    '{n} chips': '{n} zseton',
    Play: 'Kijátszom',
    'Keys: 1–9 card, S / E / B bet, ↑ ↓ stake, Enter plays.': 'Billentyűk: 1–9 kártya, S / E / B tipp, ↑ ↓ tét, Enter kijátssza.',
  },
  // Replay moments written by the rules (crates/rules/src/games/big_or_small.rs).
  patterns: [
    ["Called (\\{[a-z0-9-]+\\})'s (\\d+) exactly \\(\\+(\\d+)\\)", 'Telitalálat: $1 kártyája pont $2 volt (+$3)'],
    ['Six-chip swing against (\\{[a-z0-9-]+\\})', 'Hatzsetonos fordulat $1 ellen'],
    ['Lead stolen in the last round \\((\\d+)–(\\d+)\\)', 'Az utolsó körben megszerzett vezetés ($1–$2)'],
  ],
};

export default hu;
