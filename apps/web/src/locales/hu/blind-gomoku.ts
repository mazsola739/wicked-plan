// Hungarian for Blind Gomoku (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Vak amőba',
  short: 'Öt egy sorban a bot ellen, de a régebbi lapkák lefordulnak: jegyezd meg, melyik a tiéd.',
  strings: {
    'Five in a row: you won!': 'Öt egy sorban: nyertél!',
    'The bot made five in a row.': 'A bot kirakta az ötöt egy sorban.',
    'The board is full: a draw.': 'Megtelt a tábla: döntetlen.',
    'Tap again (or Place) to put your tile here.': 'Koppints rá újra (vagy a Lerakásra), hogy ide tedd a lapkádat.',
    'Your turn: tap an empty square.': 'Te lépsz: koppints egy üres mezőre.',
    'The bot is thinking…': 'A bot gondolkodik…',
    '{n} tiles on the board': '{n} lapka a táblán',
    'Square {n}: {state}': '{n}. mező: {state}',
    empty: 'üres',
    down: 'lefordítva',
    dark: 'sötét',
    light: 'világos',
    yours: 'tiéd',
    bot: 'bot',
    'face down': 'lefordítva',
    'Peek ({n})': 'Kukucskálás ({n})',
    Place: 'Lerakás',
    'Only the newest tile shows its colour. A peek shows every tile for 3 seconds but costs 100 points of a win.':
      'Csak a legújabb lapka színe látszik. A kukucskálás 3 másodpercre minden lapkát megmutat, de 100 pontba kerül a győzelemből.',
    'Keys: arrows move, Enter places, P peeks.': 'Billentyűk: a nyilak mozgatnak, az Enter lerak, a P kukucskál.',
    // Replay moments written by the rules (crates/rules/src/games/blind_gomoku.rs).
    'Five in a row without a peek': 'Öt egy sorban, kukucskálás nélkül',
    'Six in a row without a peek': 'Hat egy sorban, kukucskálás nélkül',
    'Seven in a row without a peek': 'Hét egy sorban, kukucskálás nélkül',
    'Eight in a row without a peek': 'Nyolc egy sorban, kukucskálás nélkül',
    'Nine in a row without a peek': 'Kilenc egy sorban, kukucskálás nélkül',
    'Five in a row on a blind board': 'Öt egy sorban, vak táblán',
    'Six in a row on a blind board': 'Hat egy sorban, vak táblán',
    'Seven in a row on a blind board': 'Hét egy sorban, vak táblán',
    'Eight in a row on a blind board': 'Nyolc egy sorban, vak táblán',
    'Nine in a row on a blind board': 'Kilenc egy sorban, vak táblán',
  },
};

export default hu;
