// Hungarian for Laying Grass (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Fűtelepítés',
  short: 'Növeld a területedet poliominó lapkákkal; a kulcsmezők ráadást és ugrást adnak.',
  strings: {
    'Encore: place again!': 'Ráadás: tegyél le még egyet!',
    'Your turn': 'Te jössz',
    '{name} to move': '{name} jön',
    'Territory board': 'Területtábla',
    'Pick a piece, tap the board to put it there, then Place.': 'Válassz lapkát, koppints a táblára, hogy odategyed, aztán: Lerakom.',
    '{name} is placing…': '{name} rak le…',
    Turn: 'Forgatás',
    Flip: 'Tükrözés',
    Leap: 'Ugrás',
    'Must touch your squares': 'A mezőidhez kell érnie',
    'Squares taken': 'Foglalt mezők',
    Place: 'Lerakom',
    '{n} sq': '{n} mező',
    '⟳ encore: place again · ↗ leap: one later piece anywhere. 10 points per square.':
      '⟳ ráadás: tegyél le még egyet · ↗ ugrás: egy későbbi lapka bárhová mehet. Mezőnként 10 pont.',
    'Keys: 1–3 pick, R turns, F flips, arrows move, L leap, Enter places.':
      'Billentyűk: 1–3 választ, R forgat, F tükröz, nyilak mozgatnak, L ugrás, Enter lerak.',

    // Replay moments (crates/rules/src/games/polyomino_territory.rs).
    'Leap grabs two key squares': 'Egy ugrás két kulcsmezőt is megszerez',
    'Leap grabs an encore square': 'Egy ugrás ráadásmezőt szerez',
    'Leap grabs another leap': 'Egy ugrás újabb ugrást szerez',
  },
};

export default hu;
